fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/main.cc")
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-I/usr/include/octave-7.2.0")
        .compile("octave-language-server");

    println!("cargo:rerun-if-changed=src/main.cc");
    println!("cargo:rerun-if-changed=src/main.h");

    println!("cargo:rustc-link-lib=octinterp");
    println!("cargo:rustc-link-search=/usr/lib64/octave/7.2.0");
}
