fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/main.cc")
        .flag_if_supported("-std=c++14")
        .compile("octave-language-server");
}
