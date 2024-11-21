pub struct Scanner {
    source: Vec<u8>,
}

impl Scanner {
    pub fn new(source: &[u8]) -> Self {
        Scanner {
            source: source.to_vec(),
        }
    }

    pub fn scan_tokens(&self) -> Vec<Token> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Token;
