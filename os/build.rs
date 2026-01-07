use std::env;
use std::fs::{read_dir, File};
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    
    // Determine target architecture from environment
    let target = env::var("TARGET").unwrap_or_else(|_| "riscv64gc-unknown-none-elf".to_string());
    let is_rv32 = target.contains("riscv32");
    
    // Set the correct user target path based on architecture
    let target_path = if is_rv32 {
        "../user/target/riscv32imac-unknown-none-elf/release/"
    } else {
        "../user/target/riscv64gc-unknown-none-elf/release/"
    };
    
    println!("cargo:rerun-if-changed={}", target_path);
    insert_app_data(target_path, is_rv32).unwrap();
}

fn insert_app_data(target_path: &str, is_rv32: bool) -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    // Use .word for RV32 and .quad for RV64
    let ptr_directive = if is_rv32 { ".word" } else { ".quad" };

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    {} {}"#,
        ptr_directive,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    {} app_{}_start"#, ptr_directive, i)?;
    }
    writeln!(f, r#"    {} app_{}_end"#, ptr_directive, apps.len() - 1)?;

    writeln!(
        f,
        r#"
    .global _app_names
_app_names:"#
    )?;
    for app in apps.iter() {
        writeln!(f, r#"    .string "{}""#, app)?;
    }

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, target_path
        )?;
    }
    Ok(())
}
