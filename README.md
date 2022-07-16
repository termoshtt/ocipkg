ocipkg
=======

[![cargo-doc](https://img.shields.io/badge/master-ocipkg-green)](https://termoshtt.github.io/ocipkg/ocipkg/index.html)

OCI Registry for package distribution.

Features
---------
- You can distribute your binary including static or shared library
  through OCI registry (e.g. GitHub Container Registry) by your own authority.
  - [WIP:](https://github.com/termoshtt/ocipkg/issues/46) Optionally, support container signing in [sigstore/cosign](https://github.com/sigstore/cosign) way.
- Users can download your binary without container runtime (e.g. docker or podman).
- Binaries are stored in local file system (typically under `$XDG_DATA_HOME/ocipkg`)
  with image name and tags, and safely shared by several local projects.
- Integration to linking libraries. Users can link same library specified by image name and tag everywhere.

Examples
---------

- [Use in build.rs](./examples/rust-exe)
- [Create package in Rust](./examples/rust-exe)
- [WIP:](https://github.com/termoshtt/ocipkg/issues/23) Use in cmake
- [WIP:](https://github.com/termoshtt/ocipkg/issues/23) Create package in cmake

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
         ArchLinux, Gentoo Linux, NixOS, FreeBSD,
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

ocipkg focuses on 3., i.e. helping distributing binary compiled
by the developer through OCI registry.

Links
------

[Open Container Initiative (OCI)](https://opencontainers.org/) is a project under [Linux Foundation](https://www.linuxfoundation.org/).

- [OCI Image Format Specification](https://github.com/opencontainers/image-spec)
- [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec)

The idea that distribute binary files (not a system image) using OCI registry is based on [ORAS][oras].

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
