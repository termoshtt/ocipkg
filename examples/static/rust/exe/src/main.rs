extern "C" {
    fn ocipkg_hello_world();
}

fn main() {
    unsafe { ocipkg_hello_world() };
}
