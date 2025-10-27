pub mod elf;
pub mod error;
pub mod parse;

use error::{Error, Res};
use parse::Parser;
use std::io::{Read, Write};

fn run() -> Res<()> {
    let file_paths = std::env::args_os()
        .skip(1)
        .map(Into::<std::path::PathBuf>::into);
    let file_paths_len = file_paths.len();
    if file_paths_len == 0 {
        return Err(Error::Cli("No file paths provided".into()));
    }
    let mut stdout = std::io::stdout().lock();
    let mut add_newline = false;
    for file_path in file_paths {
        let file = std::io::BufReader::new(std::fs::File::open(&file_path)?);
        if file_paths_len > 1 {
            if add_newline {
                writeln!(&mut stdout)?;
            }
            add_newline = true;
            writeln!(&mut stdout, "{}:", file_path.canonicalize()?.display())?;
        }
        let data_stream = file.bytes().scan(Some(file_path), |maybe_path, res| {
            maybe_path.as_ref()?;
            Some(res.map_err(|e| Error::Io(maybe_path.take().unwrap(), e)))
        });
        let mut parser = Parser::new(data_stream);
        parser.start()?;
        parser.display(&mut stdout)?;
    }
    Ok(())
}

fn main() {
    _ = run().map_err(|e| eprintln!("{e}"));
}
