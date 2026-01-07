use std::env;

fn main() {
    // Get target architecture from environment
    let target = env::var("TARGET").unwrap_or_else(|_| String::from("riscv64gc-unknown-none-elf"));
    
    // Set cfg based on architecture
    if target.contains("riscv32") {
        println!("cargo:rustc-cfg=riscv32");
    } else if target.contains("riscv64") {
        println!("cargo:rustc-cfg=riscv64");
    }
    
    // Re-run if TARGET changes
    println!("cargo:rerun-if-env-changed=TARGET");
}
