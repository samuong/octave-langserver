use anyhow::Result;

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("octave-langserver/src/bridge.h");
        fn init() -> Result<()>;
        fn analyse(text: &str);
        fn symbol_at(line: u32, character: u32) -> Result<String>;
        fn definition(symbol: &str) -> Result<[u32; 2]>;
        fn clear_indexes();
    }
}

// TODO: figure out how to serialise tests (in code). For now, tests  need to be invoked using
// `cargo test -- --test-threads=1`, otherwise the Octave interpreter will crash.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_at() -> Result<()> {
        ffi::init()?;
        ffi::analyse("msg = 'Hello, world!'\ndisp(msg)\n");
        assert_eq!(ffi::symbol_at(0, 0)?, "msg");
        assert_eq!(ffi::symbol_at(1, 3)?, "disp");
        assert!(ffi::symbol_at(1, 4).is_err());
        assert_eq!(ffi::symbol_at(1, 5)?, "msg");
        assert!(ffi::symbol_at(1, 10).is_err());
        assert!(ffi::symbol_at(2, 0).is_err());
        ffi::clear_indexes(); // TODO: always run this, even if we finish the test early
        Ok(())
    }

    #[test]
    fn test_goto_def() -> Result<()> {
        ffi::init()?;
        ffi::analyse(
            r#"
function sum = add (augend, addend)
    sum = augend + addend;
endfunction
f = @add;
y = f (1, 2);
"#,
        );
        let symbol = ffi::symbol_at(3, 5)?;
        assert_eq!(symbol, "add");
        let pos = ffi::definition(symbol.as_str())?;
        assert_eq!(pos, [0, 0]);
        ffi::clear_indexes(); // TODO: always run this, even if we finish the test early
        Ok(())
    }
}
