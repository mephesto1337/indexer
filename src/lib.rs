use std::{
    collections::{BTreeSet, HashMap},
    fs::{read_dir, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

mod case_insensitive_string;
pub mod tokenizer;

pub use crate::case_insensitive_string::CaseInsensitiveString;
use crate::tokenizer::{TextTokenizer, Tokenizer, XmlTokenizer};

fn traverse_tree(p: impl AsRef<Path>, mut callback: impl FnMut(PathBuf)) {
    let mut inodes = BTreeSet::new();
    let mut to_visit = Vec::new();
    to_visit.push(p.as_ref().to_path_buf());

    while let Some(p) = to_visit.pop() {
        let rd = match read_dir(&p) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("ERROR: cannot read {path}: {e}", path = p.display());
                continue;
            }
        };
        for entry in rd {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("ERROR: cannot process entry: {e}");
                    continue;
                }
            };
            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    eprintln!(
                        "ERROR: cannot get filetype for {path}: {e}",
                        path = entry.path().display()
                    );
                    continue;
                }
            };

            if ft.is_dir() {
                let path = entry.path();
                let was_not_present = inodes.insert(path.clone());
                if was_not_present {
                    to_visit.push(path);
                }
            } else if ft.is_file() {
                callback(entry.path());
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    term_frequency: HashMap<CaseInsensitiveString<'static>, usize>,
    count: usize,
}

impl Document {
    pub fn build<P: AsRef<Path>>(filename: P, mut tokenizer: impl Tokenizer) -> io::Result<Self> {
        let mut file = BufReader::new(File::open(filename)?);
        let mut term_frequency = HashMap::new();

        let count = tokenizer.tokenize(&mut file, &mut term_frequency)?;

        Ok(Self {
            term_frequency,
            count,
        })
    }

    pub fn term_frequency(&self, term: &str) -> f64 {
        match self.term_frequency.get(&term.into()) {
            Some(c) => *c as f64 / self.count as f64,
            None => 0f64,
        }
    }

    pub fn contains(&self, term: &str) -> bool {
        self.term_frequency.contains_key(&term.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    documents: HashMap<PathBuf, Document>,
    term_frequency: HashMap<CaseInsensitiveString<'static>, usize>,
}

macro_rules! apply_tokenizer {
    ($tokenizer:expr, $path:ident, $index:ident) => {{
        let tokenizer = $tokenizer;
        let p = $path;
        match Document::build(&p, tokenizer) {
            Ok(d) => {
                eprintln!("INFO: processed {path}", path = p.display());
                for term in d.term_frequency.keys() {
                    if let Some(count) = $index.term_frequency.get_mut(term) {
                        *count += 1;
                    } else {
                        $index.term_frequency.insert(term.clone(), 1);
                    }
                }
                $index.documents.insert(p, d);
            }
            Err(e) => {
                eprintln!("ERROR: processing {path}: {e}", path = p.display());
            }
        }
    }};
}

impl Index {
    pub fn new(p: impl AsRef<Path>) -> Self {
        let mut index = Self {
            documents: HashMap::new(),
            term_frequency: HashMap::new(),
        };
        traverse_tree(p, |p| match p.extension().and_then(|e| e.to_str()) {
            Some("xhtml") | Some("xml") => apply_tokenizer!(XmlTokenizer::default(), p, index),
            Some("text") | Some("txt") => apply_tokenizer!(TextTokenizer::default(), p, index),
            Some("rs") => apply_tokenizer!(TextTokenizer::default(), p, index),
            Some(ext) => {
                eprintln!("No handler for {ext:?} documents");
            }
            None => {
                eprintln!("Unknown document type {path}", path = p.display());
            }
        });
        index
    }

    fn idf(&self, term: &str) -> f64 {
        let n = self.documents.len() as f64;
        let d = self.documents.values().filter(|d| d.contains(term)).count() as f64;
        assert!(n >= d);
        (n / (d + 1f64)).log2()
    }

    pub fn search<'a>(&'a self, terms: &'_ str) -> Vec<(&'a Path, f64)> {
        let terms = tokenizer::Lexer::new(terms)
            .map(|t| (t, self.idf(t)))
            .collect::<Vec<_>>();
        let mut results: Vec<_> = self
            .documents
            .iter()
            .map(move |(filename, d)| {
                (
                    filename.as_path(),
                    terms
                        .iter()
                        .map(|(t, idf)| {
                            let tf = d.term_frequency(t);
                            tf * *idf
                        })
                        .sum::<f64>(),
                )
            })
            .filter(|(_, score)| score != &0f64)
            .collect();
        results.sort_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap());
        results.reverse();
        results
    }

    pub fn load<R: io::Read>(reader: R) -> io::Result<Self> {
        serde_json::from_reader(reader)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }

    pub fn save<W: io::Write>(&self, writer: W) -> io::Result<()> {
        serde_json::to_writer(writer, self)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }
}
