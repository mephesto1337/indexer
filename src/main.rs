use std::{
    fs::{metadata, File},
    io::{self, BufReader, BufWriter},
    path::Path,
    time::SystemTime,
};

use clap::{Parser, Subcommand};

use indexer::Index;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Options {
    /// Index file to use
    #[arg(
        short = 'i',
        long = "index",
        value_name = "FILE",
        default_value = "index.json"
    )]
    index_file: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build the index file if not present
    Build {
        /// Force rebuild
        #[arg(short, long, default_value_t = false)]
        force: bool,

        /// Directory to index
        #[arg(default_value = ".")]
        directory: String,
    },

    /// search for terms
    Search {
        /// Maximum number of results to display
        #[arg(short, long, default_value_t = 10)]
        count: usize,

        /// Query
        query: String,
    },

    /// Checks that files references in index file are up to date
    Check,
}

fn file_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    let path = path.as_ref();
    match metadata(path) {
        Ok(m) => {
            if m.is_file() {
                Ok(true)
            } else if m.is_dir() {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("{p} points to a directory", p = path.display()),
                ))
            } else if m.is_symlink() {
                unreachable!()
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("{p} points to an unknown type", p = path.display()),
                ))
            }
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

fn get_last_modified_time(path: impl AsRef<Path>) -> io::Result<SystemTime> {
    let m = metadata(path)?;
    m.modified()
}

fn main() -> io::Result<()> {
    env_logger::init();
    let options = Options::parse();

    match options.command {
        Command::Build {
            ref directory,
            force,
        } => {
            if force || !file_exists(&options.index_file)? {
                log::info!("Computing index for {directory}...");
                let index = Index::new(directory);
                let f = File::create(&options.index_file)?;
                index.save(BufWriter::new(f))?;
                log::info!("Saved index at {path}", path = &options.index_file);
            } else {
                log::warn!("Index already exists");
            }
        }
        Command::Search { count, ref query } => {
            let index = Index::load(BufReader::new(File::open(&options.index_file)?))?;
            let results = index.search(query);
            if results.is_empty() {
                println!("No match for query {query:?}");
            }
            for (p, s) in results.into_iter().take(count) {
                println!("{path}: {s}", path = p.display());
            }
        }
        Command::Check => {
            let index_time = get_last_modified_time(&options.index_file)?;
            let index = Index::load(BufReader::new(File::open(&options.index_file)?))?;
            let (filename, mtime) = index.last_modified_file()?;
            if index_time >= mtime {
                println!("Index file {f} is up to date", f = &options.index_file);
            } else {
                println!(
                    "{filename} is newer than index file ({f})",
                    f = &options.index_file,
                    filename = filename.display()
                );
            }
        }
    }

    Ok(())
}
