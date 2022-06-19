ocipkg
=======

WIP: OCI Registry for binary distribution.

Features
---------
- You can distribute your binary including static or shared library
  through OCI registry, e.g. GitHub Container Registry, by your own authority.
  - Similar to [OCI Registry As Storage(ORAS)][oras] and [OCI Artifacts][oci-artifacts]
- Your users can download your binary without container runtime
  and link them in `pkg-config` compatible manner.

Links
------

- [OCI Image Format](https://github.com/opencontainers/image-spec)
  - This describes how container consists of
- [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec)
  - This describes how the containers are distributed (pushed and pulled) over HTTP
  - This is based on [Docker Registry HTTP API V2 protocol](https://github.com/docker/distribution/blob/master/docs/spec/manifest-v2-2.md)
- [OCI Registry As Storage (ORAS)][oras]
  - CLI and Go module to realize ORAS

[oras]: https://github.com/oras-project/oras
[oci-artifacts]: https://github.com/opencontainers/artifacts
