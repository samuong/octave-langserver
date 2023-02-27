use std::collections::HashMap;

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type Index;
        fn add_symbol(&mut self, line: u32, character: u32, symbol: String);
        fn add_definition(&mut self, symbol: String, line: u32, character: u32);
    }

    unsafe extern "C++" {
        include!("octave-langserver/src/bridge.h");
        fn init(logger: fn(&str)) -> Result<()>;
        fn analyse(text: &str, index: &mut Index);
    }
}

pub struct Index {
    // line -> (character -> symbol)
    symbols: HashMap<u32, HashMap<u32, String>>,
    // symbol -> (line, character)
    definitions: HashMap<String, (u32, u32)>,
}

impl Index {
    pub fn new() -> Index {
        Index {
            symbols: HashMap::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, line: u32, character: u32, symbol: String) {
        eprintln!("adding symbol '{symbol}' to index at position ({line}, {character})");
        let Some(char_to_symbol) = self.symbols.get_mut(&line) else {
            self.symbols.insert(line, HashMap::from([(character, symbol)]));
            return;
        };
        char_to_symbol.insert(character, symbol);
    }

    pub fn find_symbol(&self, line: u32, character: u32) -> Option<&String> {
        let char_to_symbol = self.symbols.get(&line)?;
        for (&start, symbol) in char_to_symbol.iter() {
            let end = start + symbol.len() as u32;
            if start <= character && character < end {
                return Some(symbol);
            }
        }
        None
    }

    pub fn add_definition(&mut self, symbol: String, line: u32, character: u32) {
        eprintln!("adding definition '{symbol}' to index at position ({line}, {character})");
        self.definitions.insert(symbol, (line, character));
    }

    pub fn find_definition(&self, symbol: &str) -> Option<&(u32, u32)> {
        self.definitions.get(symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial]
    fn test_symbol_at() {
        ffi::init(|s| eprintln!("{s}")).unwrap();
        let mut index = Index::new();
        ffi::analyse("msg = 'Hello, world!'\ndisp(msg)\n", &mut index);
        assert_eq!(index.find_symbol(0, 0).unwrap(), "msg");
        assert_eq!(index.find_symbol(1, 3).unwrap(), "disp");
        assert!(index.find_symbol(1, 4).is_none());
        assert_eq!(index.find_symbol(1, 5).unwrap(), "msg");
        assert!(index.find_symbol(1, 10).is_none());
        assert!(index.find_symbol(2, 0).is_none());
    }

    #[test]
    #[serial]
    fn test_goto_def() {
        ffi::init(|s| eprintln!("{s}")).unwrap();
        let mut index = Index::new();
        ffi::analyse(
            r#"
function sum = add (augend, addend)
    sum = augend + addend;
endfunction
f = @add;
y = f (1, 2);
"#,
            &mut index,
        );
        let symbol = index.find_symbol(3, 5).unwrap();
        assert_eq!(symbol, "add");
        let &pos = index.find_definition(symbol.as_str()).unwrap();
        assert_eq!(pos, (0, 0));
    }
}
