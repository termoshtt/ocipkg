ocipkg
=======

WIP: OCI Registry for binary distribution.

**This is design document for pre-implementation. These features are not implmeneted yet.**

Features
---------
- You can distribute your binary including static or shared library
  through OCI registry, e.g. GitHub Container Registry, by your own authority.
  - Optionally, support container signing in [sigstore/cosign][cosign] way.
- Users can download your binary without container runtime (e.g. docker or podman).
- Binaries are stored in local file system (typically under `$XDG_DATA_HOME/ocipkg`)
  with image name and tags, and safely shared by several local projects
  since they are immutable due to container immutablility.
- Integration to linking libraries. Users can link same library specified by image name and tag everywhere.

How to use
-----------

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
