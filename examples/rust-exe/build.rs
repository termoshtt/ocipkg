fn main() {
    ocipkg::link_package("ghcr.io/termoshtt/ocipkg/rust-lib:latest")
        .expect("Failed to link rust-lib");
}
