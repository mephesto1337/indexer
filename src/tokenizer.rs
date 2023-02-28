use std::{
    collections::HashMap,
    io::{self, Read},
};

use crate::CaseInsensitiveString;

mod lexer;
mod text;
mod xml;

pub use self::text::TextTokenizer;
pub use self::xml::XmlTokenizer;
pub use lexer::Lexer;

pub trait Tokenizer {
    /// Returns the number of tokens encountered
    fn tokenize<R: Read>(
        &mut self,
        reader: R,
        term_frequency: &mut HashMap<CaseInsensitiveString<'static>, usize>,
    ) -> io::Result<usize>;

    /// Returns the number of tokens encountered as well as the string btreemap
    fn tokenize_string(
        &mut self,
        s: &str,
        term_frequency: &mut HashMap<CaseInsensitiveString<'static>, usize>,
    ) -> usize {
        let mut count = 0;
        for token in lexer::Lexer::new(s) {
            let token: CaseInsensitiveString<'static> = token.to_owned().into();
            if let Some(c) = term_frequency.get_mut(&token) {
                *c += 1;
            } else {
                term_frequency.insert(token, 1);
            }
            count += 1;
        }
        count
    }
}
