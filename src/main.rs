use std::process;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("octave-language-server/src/main.h");
        fn init() -> Result<()>;
        fn eval(eval_str: &str) -> Result<()>;
    }
}

fn main() {
    if let Err(err) = ffi::init() {
        eprintln!("error: {}", err);
        process::exit(1);
    }
    if let Err(err) = ffi::eval("disp('Hello, world!')\n") {
        eprintln!("error: {}", err);
        process::exit(1);
    }
}
