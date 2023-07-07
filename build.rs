fn main() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,librbackend.abi3.so")
}