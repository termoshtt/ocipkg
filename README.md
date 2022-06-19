ocipkg
=======

WIP: OCI Registry for binary distribution.

**This is design document for pre-implementation. These features are not implmeneted yet.**

Features
---------
- You can distribute your binary including static or shared library
  through OCI registry (e.g. GitHub Container Registry) by your own authority.
  - Optionally, support container signing in [sigstore/cosign][cosign] way.
- Users can download your binary without container runtime (e.g. docker or podman).
- Binaries are stored in local file system (typically under `$XDG_DATA_HOME/ocipkg`)
  with image name and tags, and safely shared by several local projects.
- Integration to linking libraries. Users can link same library specified by image name and tag everywhere.

Why ocipkg?
-------------
I have determined to start this project while writing FFI crate in Rust.
The problem is "how to get a share/static library linked to FFI crate".
This is the problem bothered me and prevent from creating portable C++ library.

We have three options:

1. Use library in the system
    - ❤ Library is prepared by the system administrator who would be most familiar with the system.
    - 💔 Developer have to know how the library is distributed in user's system,
         possibly Ubuntu 22.04, 20.04, 18.04, Debian sid, 11, 10, 9, RHEL9, 8, 7,
         ArchLinux, Gentoo Linux, NixOS, FreeBSD,
         macOS with brew, Windows with winget, chocolatey, scoop, ...
    - 💔 Some system does not allows co-existence of multi-version libraries.
2. Get source code from the internet, and build and link them
    - ❤ Developer can control the library fully.
    - 💔 Development tool, e.g. `cmake`, is required in user system,
         and requires additional build resources.
3. Get compiled library from the internet on build time
    - ❤ Developer can control the library fully, too.
    - 💔 Requires HTTP(or other protocol) server to distribute the library
    - 💔 Developer have to ready binaries for every supported platforms,
         e.g. `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-unknown-linux-gnu`,...

ocipkg focuses on 3., i.e. helping distributing binary compiled by the developer.

How to use ocipkg
------------------

### ocipkg crate for Rust
TBW

### pkg-config compatible CLI for C, C++, and other ld-based languages
TBW

[image-spec]: https://github.com/opencontainers/image-spec
[runtime-spec]: https://github.com/opencontainers/runtime-spec
[distribution-spec]: https://github.com/opencontainers/distribution-spec

[oras]: https://github.com/oras-project/oras
[oci-artifacts]: https://github.com/opencontainers/artifacts
[cosign]: https://github.com/sigstore/cosign
