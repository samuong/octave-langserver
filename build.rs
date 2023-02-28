use std::io::BufRead;
use std::process::Command;

fn main() {
    let output = Command::new("mkoctfile")
        .args(["--dry-run", "--link-stand-alone", "src/tree_walker.cc"])
        .output()
        .unwrap();
    let mut stdout = output.stdout.lines();
    let compile_cmd = stdout.next().unwrap().unwrap();
    let include_dirs = compile_cmd
        .split_whitespace()
        .filter_map(|s| s.strip_prefix("-I"));

    cxx_build::bridge("src/bridge.rs")
        .file("src/bridge.cc")
        .file("src/tree_walker.cc")
        .flag("-std=c++11")
        .includes(include_dirs)
        .compile("octave-langserver");

    println!("cargo:rerun-if-changed=src/bridge.cc");
    println!("cargo:rerun-if-changed=src/bridge.h");
    println!("cargo:rerun-if-changed=src/tree_walker.cc");
    println!("cargo:rerun-if-changed=src/tree_walker.h");

    let link_cmd = stdout.next().unwrap().unwrap();
    for link_arg in link_cmd.split_whitespace() {
        if let Some(lib) = link_arg.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else if let Some(lib_dir) = link_arg.strip_prefix("-L") {
            println!("cargo:rustc-link-search={lib_dir}");
        }
    }
    assert!(stdout.next().is_none());
}
