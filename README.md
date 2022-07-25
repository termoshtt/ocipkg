ocipkg
=======

[![crate](https://img.shields.io/crates/v/ocipkg.svg)](https://crates.io/crates/ocipkg) 
[![docs.rs](https://docs.rs/ocipkg/badge.svg)](https://docs.rs/ocipkg)
[![master](https://img.shields.io/badge/docs-master-blue)](https://termoshtt.github.io/ocipkg/ocipkg/index.html)

OCI Registry for package distribution.

Features
---------

ocipkg is designed as a thin OCI registry client:

- Read and Write oci-archive format (tar archive of [OCI Image Layout](https://github.com/opencontainers/image-spec/blob/main/image-layout.md)).
- Push and Pull container images to OCI registry without external container runtime, e.g. docker or podman

In addition, ocipkg provides utilities for using OCI registry for package distribution:

- CLI tool for building containers from files, directory,
  and Rust project with `Cargo.toml` metadata.
- `build.rs` helper for getting and linking library file (`*.a` or `*.so`) as a container

Why ocipkg?
-------------
I have determined to start this project while writing FFI crate in Rust.
The problem is "how to get a share/static library linked to FFI crate".
This is the problem bothered me and prevent from creating portable C++ library.

We have three options:

1. Use library in the system
    - ✔ Library is prepared by the system administrator who would be most familiar with the system.
    - ❌ Developer have to know how the library is distributed in user's system,
         possibly Ubuntu 22.04, 20.04, 18.04, Debian sid, 11, 10, 9, RHEL9, 8, 7,
         ArchLinux, Gentoo Linux, FreeBSD,
         macOS with brew, Windows with winget, chocolatey, scoop, ...
    - ❌ Some system does not allows co-existence of multi-version libraries.
    - Most of `*-sys` crate support this option.
2. Get source code from the internet, and build and link them
    - ✔ Developer can control the library fully.
    - ❌ Development tool, e.g. `cmake`, is required in user system,
         and requires additional build resources.
    - Some crate support this option, and they are named with `*-src`.
3. Get compiled library from the internet on build time
    - ✔ Developer can control the library fully, too.
    - ❌ Requires HTTP(or other protocol) server to distribute the library
    - ❌ Developer have to ready binaries for every supported platforms,
         e.g. `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-unknown-linux-gnu`,...

ocipkg focuses on the option 3., i.e. helping distributing binary compiled
by the developer through OCI registry.

Examples
---------

| Library type | Create package in Rust | Use package from Rust |
|:-------------|:-----------------------|:----------------------|
| static       |[examples/static/rust/lib](./examples/static/rust/lib)   | [examples/static/rust/exe](./examples/static/rust/exe) |
| dynamic      |[examples/dynamic/rust/lib](./examples/dynamic/rust/lib) | [examples/dynamic/rust/exe](./examples/dynamic/rust/exe) |

CLI tools
----------

### Install

```bash
cargo install --features=cli ocipkg
```

### `ocipkg` command

TBW

### `cargo-ocipkg` command

A tool for creating and publishing container consists of
static or dynamic library built by `cargo build`:

```
$ cargo ocipkg build --release
    Finished release [optimized] target(s) in 0.00s
    Creating oci-archive (/home/teramura/github.com/termoshtt/ocipkg/examples/dynamic/rust/lib/target/release/ocipkg_dd0c7a812fd0fcbc.tar)
```

The filename is in form of `ocipkg_{{ hash }}.tar`,
and this hash is calculated from image name and `Cargo.toml`.

Container image name is determined using git commit hash
as `{{ registry }}:$(git rev-parse HEAD --short)`
where registry name is set by `Cargo.toml`:

```toml
[package.metadata.ocipkg]
registry = "ghcr.io/termoshtt/ocipkg/dynamic/rust"
```

This container can be published by `cargo-ocipkg publish`:

```
$ cargo ocipkg publish --release
     Publish container (ghcr.io/termoshtt/ocipkg/dynamic/rust:be7f108)
```

Links
------

[Open Container Initiative (OCI)](https://opencontainers.org/) is a project under [Linux Foundation](https://www.linuxfoundation.org/).

- [OCI Image Format Specification](https://github.com/opencontainers/image-spec)
- [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec)

This project does not depend on [OCI Runtime specification](https://github.com/opencontainers/runtime-spec)
since we never run a container.

The idea that distribute any files (not a system image) using OCI registry is based on [ORAS][oras].

- [OCI Registry As Storage][oras]

[oras]: https://oras.land/

Similar projects trying to distribute packages using OCI registries:

- [OCI transport plugin for apt-get](https://github.com/AkihiroSuda/apt-transport-oci)
- [Homebrew](https://github.com/orgs/Homebrew/packages)

License
--------

© 2020 Toshiki Teramura (@termoshtt)

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
