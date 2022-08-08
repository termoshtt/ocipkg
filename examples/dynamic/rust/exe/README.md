Rust example using dynamic library package
-------------------------------------------

This crate contains an executable,
which links `ghcr.io/termoshtt/ocipkg/dynamic/rust` package
generated by [examples/dynamic/rust/lib](../lib).

```rust
fn main() {
    ocipkg::link_package("ghcr.io/termoshtt/ocipkg/dynamic/rust:67c8634").unwrap()
}
```

About RPATH
------------
`ocipkg::link_package` appends a linker flag `-Wl,-rpath={{ where ocipkg saves package }}`,
i.e. the executable of this crate will contains a path where the library is stored:

```
$ readelf -a target/debug/ocipkg-example-dynamic-rust | grep RPATH
 0x000000000000000f (RPATH)              Library rpath: [/home/username/.local/share/ocipkg/ghcr.io/termoshtt/ocipkg/dynamic/rust/__67c8634]
```