use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("linux") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,librbackend.abi3.so")
    }
    if target.contains("apple") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-install_name,librbackend.abi3.so")
    }
}
