use std::{
    collections::HashMap,
    io::{self, Read},
};

use xml::reader::{EventReader, XmlEvent};

use crate::{tokenizer::Tokenizer, CaseInsensitiveString};

#[derive(Debug, Default)]
pub struct XmlTokenizer;

impl Tokenizer for XmlTokenizer {
    fn tokenize<R: Read>(
        &mut self,
        reader: R,
        term_frequency: &mut HashMap<CaseInsensitiveString<'static>, usize>,
    ) -> io::Result<usize> {
        let mut count = 0;
        for event in EventReader::new(reader).into_iter() {
            let event =
                event.map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
            if let XmlEvent::Characters(s) = event {
                let c = self.tokenize_string(&s, term_frequency);
                count += c;
            }
        }

        Ok(count)
    }
}
