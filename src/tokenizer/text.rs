use std::{
    collections::HashMap,
    io::{self, Read},
};

use crate::{tokenizer::Tokenizer, CaseInsensitiveString};

#[derive(Debug, Default)]
pub struct TextTokenizer;

impl Tokenizer for TextTokenizer {
    fn tokenize<R: Read>(
        &mut self,
        mut reader: R,
        term_frequency: &mut HashMap<CaseInsensitiveString<'static>, usize>,
    ) -> io::Result<usize> {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;
        Ok(self.tokenize_string(&s, term_frequency))
    }
}
