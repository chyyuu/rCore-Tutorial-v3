use std::env;
use std::fs::{File, read_dir};
use std::io::{Result, Write};

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    
    // Detect target architecture from environment
    let target = env::var("TARGET").unwrap_or_else(|_| "riscv64gc-unknown-none-elf".to_string());
    let target_path = format!("../user/target/{}/release/", target);
    
    println!("cargo:rerun-if-changed={}", target_path);
    insert_app_data(&target_path, &target).unwrap();
}

fn insert_app_data(target_path: &str, target: &str) -> Result<()> {
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

    // Use .word for RV32, .quad for RV64
    let is_rv32 = target.contains("riscv32");
    let data_directive = if is_rv32 { ".word" } else { ".quad" };
    let align = if is_rv32 { 2 } else { 3 };

    writeln!(
        f,
        r#"
    .align {}
    .section .data
    .global _num_app
_num_app:
    {} {}"#,
        align, data_directive, apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    {} app_{}_start"#, data_directive, i)?;
    }
    writeln!(f, r#"    {} app_{}_end"#, data_directive, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align {3}
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, target_path, align
        )?;
    }
    Ok(())
}
