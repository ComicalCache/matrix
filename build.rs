use std::process::Command;

fn main() {
    let libs = "libs";
    let objs = "objs";
    let lib = format!("{libs}/libconsts.a");
    let obj = format!("{libs}/{objs}/consts.o");

    let c = "src/c";

    Command::new("/usr/bin/gcc")
        .args([&format!("{c}/consts.c"), "-c", "-fPIC", "-o", &obj])
        .status()
        .unwrap();
    Command::new("/usr/bin/ar")
        .args(["curs", &lib, &obj])
        .status()
        .unwrap();

    println!("cargo::rustc-link-search=native={}", libs);
    println!("cargo::rustc-link-lib=static=consts");
    println!("cargo::rerun-if-changed={c}/consts.c");
}
