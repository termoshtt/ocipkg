ocipkg
=======

WIP: OCI Registry for binary distribution.

**This is design document for pre-implementation. These features are not implmeneted yet.**

Features
---------
- You can distribute your binary including static or shared library
  through OCI registry, e.g. GitHub Container Registry, by your own authority.
  - Similar to [OCI Registry As Storage(ORAS)][oras] and [OCI Artifacts][oci-artifacts]
- Your users can download your binary without container runtime
  and link them in `pkg-config` compatible manner.
  - This project is based on [OCI image spec][image-spec] as package layout
    and [OCI distribution spec][distribution-spec] for OCI registry API.
    Do not use [OCI runtime spec][runtime-spec].
  - Optionally, support container signing in [sigstore/cosign][cosign] way.

How to use
-----------
TBW

[image-spec]: https://github.com/opencontainers/image-spec
[runtime-spec]: https://github.com/opencontainers/runtime-spec
[distribution-spec]: https://github.com/opencontainers/distribution-spec

[oras]: https://github.com/oras-project/oras
[oci-artifacts]: https://github.com/opencontainers/artifacts
[cosign]: https://github.com/sigstore/cosign
