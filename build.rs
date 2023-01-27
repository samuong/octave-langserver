fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/main.cc")
        .file("src/tree_walker.cc")
        .flag("-std=c++11")
        .flag("-I/usr/include/octave-7.2.0")
        .compile("octave-langserver");

    println!("cargo:rerun-if-changed=src/main.cc");
    println!("cargo:rerun-if-changed=src/main.h");
    println!("cargo:rerun-if-changed=src/tree_walker.cc");
    println!("cargo:rerun-if-changed=src/tree_walker.h");

    println!("cargo:rustc-link-lib=octave");
    println!("cargo:rustc-link-lib=octinterp");
    println!("cargo:rustc-link-search=/usr/lib64/octave/7.2.0");
}
