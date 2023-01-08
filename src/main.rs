#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("octave-language-server/src/main.h");
        fn cpp_main();
    }
}

fn main() {
    ffi::cpp_main();
}
