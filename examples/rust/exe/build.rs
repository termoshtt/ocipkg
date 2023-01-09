fn main() {
    ocipkg::link_package("ghcr.io/termoshtt/ocipkg/static/cpp:e52eae9").unwrap();
    println!("cargo:rustc-link-lib=stdc++")
}
