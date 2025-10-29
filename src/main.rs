pub mod elf_header;
pub mod error;
pub mod parse;

use error::{Error, Res};
use std::io::Write;

const CURSOR_SIZE_LIMIT: u64 = 32 * 1024 * 1024;

fn run() -> Res<()> {
    let file_paths = std::env::args_os().skip(1).map(std::path::PathBuf::from);
    let file_paths_len = file_paths.len();
    if file_paths_len == 0 {
        return Err(Error::Cli("No file paths provided".into()));
    }

    let mut stdout = std::io::stdout().lock();
    let mut add_newline = false;
    let mut write_path = |stdout: &mut std::io::StdoutLock, path: &std::path::Path| -> Res<()> {
        if file_paths_len > 1 {
            if add_newline {
                writeln!(stdout)?;
            }
            add_newline = true;
            writeln!(stdout, "{}:", path.canonicalize()?.display())?;
        }
        Ok(())
    };

    for file_path in file_paths {
        let Ok(meta) = std::fs::metadata(&file_path) else {
            eprintln!("Failed to stat {}", file_path.display());
            continue;
        };
        let table = if meta.len() > CURSOR_SIZE_LIMIT {
            println!("foo");
            let file = std::io::BufReader::new(std::fs::File::open(&file_path)?);
            write_path(&mut stdout, &file_path)?;
            parse::start(file)
        } else {
            let contents = std::io::Cursor::new(std::fs::read(&file_path)?);
            write_path(&mut stdout, &file_path)?;
            parse::start(contents)
        }
        .map_err(|err| Error::RunCtx(file_path, Box::new(err)))?;
        table.display(&mut stdout)?;
    }
    Ok(())
}

fn main() {
    _ = run().map_err(|e| eprintln!("{e}"));
}
