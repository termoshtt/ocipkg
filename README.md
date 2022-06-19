ocipkg
=======

WIP: OCI Registry for binary distribution.

**This is design document for pre-implementation. These features are not implmeneted yet.**

Features
---------
- You can distribute your binary including static or shared library
  through OCI registry, e.g. GitHub Container Registry, by your own authority.
  - Similar to [OCI Registry As Storage(ORAS)][oras] and [OCI Artifacts][oci-artifacts]
  - Optionally, support container signing in [sigstore/cosign][cosign] way.
  - Roughly, you can use OCI registry like AWS S3

- Users can download your binary without container runtime
  - This project is based on [OCI image spec][image-spec] as package layout
    and [OCI distribution spec][distribution-spec] for OCI registry API.
  - Do not use [OCI runtime spec][runtime-spec] since this never runs container.

- Integration to linking libraries. Users can link same library specified by image name and tag.
  - Rust crate for using in `build.rs`
  - `pkg-config` compatible CLI tool for C, C++, and other ld-based languages.

How to use
-----------
TBW

[image-spec]: https://github.com/opencontainers/image-spec
[runtime-spec]: https://github.com/opencontainers/runtime-spec
[distribution-spec]: https://github.com/opencontainers/distribution-spec

[oras]: https://github.com/oras-project/oras
[oci-artifacts]: https://github.com/opencontainers/artifacts
[cosign]: https://github.com/sigstore/cosign
