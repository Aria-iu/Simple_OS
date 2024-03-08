use std::fs::{read_dir,File};
use std::io::{Result,Write};

static TARGET_PATH: &str = "../apps/target/riscv64gc-unknown-none-elf/release/";
fn main(){
    println!("cargo:rerun-if-changed=../apps/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()>{
    let mut f = File::create("src/link_app.S").unwrap();
    // app's name without .rs ext 
    let mut apps:Vec<_> = read_dir("../apps/src/bin").unwrap()
        .into_iter()
        .map(|dir_entry|{
            let mut name_withext = dir_entry.unwrap().file_name().into_string().unwrap();
            // drop ext str
            name_withext.drain(name_withext.find('.').unwrap()..name_withext.len());
            name_withext
        })
        .collect();
    apps.sort();

    // generate linker file
    writeln!(
        f,
        r#"    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#
        ,apps.len()
    )?;
    for i in 0..apps.len(){
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;
    writeln!(f, r#"    .gloabl _app_names
_app_names:"#)?;
    for i in 0..apps.len(){
        writeln!(f, r#"    .string "{}""#, apps[i])?;
    }

    for (idx,app) in apps.iter().enumerate(){
        println!("app_{}:{}",idx,app);
        writeln!(
            f,
            r#"    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}.bin"
app_{0}_end:
            "#,
            idx,app,TARGET_PATH
        )?;
    }
    Ok(())
}