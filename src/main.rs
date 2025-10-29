pub mod elf_header;
pub mod error;
pub mod parse;

use error::{Error, Res};
use std::io::Write;

const CURSOR_SIZE_LIMIT: u64 = 32 * 1024 * 1024;

struct Args {
    help: bool,
    all: bool,
    file_paths: Box<[std::path::PathBuf]>,
}

impl TryFrom<std::env::ArgsOs> for Args {
    type Error = Error;

    fn try_from(mut args: std::env::ArgsOs) -> Res<Self> {
        args.next();
        let mut args = args.peekable();
        let mut help = false;
        let mut all = false;
        while let Some(arg) = args.peek() {
            let arg = arg.as_encoded_bytes();
            if arg.starts_with(b"--") {
                match arg {
                    b"--help" => help = true,
                    b"--all" => all = true,
                    _ => {
                        return Err(Error::Cli(format!(
                            "Unknown argument '{}'",
                            std::str::from_utf8(arg)
                                .map_err(|_| Error::Cli("Invalid argument".into()))?
                        )));
                    }
                }
            } else if arg.starts_with(b"-") {
                for &small_arg in &arg[1..] {
                    match small_arg {
                        b'h' => help = true,
                        b'a' => all = true,
                        _ => {
                            return Err(Error::Cli(format!(
                                "Unknown argument '{}'",
                                char::from_u32(small_arg as _)
                                    .ok_or_else(|| Error::Cli("Invalid argument".into()))?
                            )));
                        }
                    }
                }
                help |= arg.contains(&b'h');
                all |= arg.contains(&b'a');
            } else {
                break;
            }
            args.next();
        }
        help |= args.peek().is_none();
        Ok(Self {
            help,
            all,
            file_paths: args.map(Into::into).collect(),
        })
    }
}

fn run(args: Args) -> Res<()> {
    if args.help {
        println!(
            "
Usage: inspector [options] paths...

Options:
    -h, --help    Display help
    -a, --all     Show all file metadata
"
        );
        return Ok(());
    }

    let mut stdout = std::io::stdout().lock();
    let mut add_newline = false;
    let mut write_path = |stdout: &mut std::io::StdoutLock, path: &std::path::Path| -> Res<()> {
        if args.file_paths.len() > 1 {
            if add_newline {
                writeln!(stdout)?;
            }
            add_newline = true;
            writeln!(stdout, "{}:", path.canonicalize()?.display())?;
        }
        Ok(())
    };

    for file_path in &args.file_paths {
        let Ok(meta) = std::fs::metadata(&file_path) else {
            eprintln!("Failed to stat '{}'", file_path.display());
            continue;
        };
        let table = if meta.len() > CURSOR_SIZE_LIMIT {
            println!("foo");
            let file = std::io::BufReader::new(std::fs::File::open(&file_path)?);
            write_path(&mut stdout, &file_path)?;
            parse::start(file, args.all)
        } else {
            let contents = std::io::Cursor::new(std::fs::read(&file_path)?);
            write_path(&mut stdout, &file_path)?;
            parse::start(contents, args.all)
        }
        .map_err(|err| Error::RunCtx(file_path.into(), Box::new(err)))?;
        table.display(&mut stdout)?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = std::env::args_os().try_into().and_then(run) {
        eprintln!("{e}")
    }
}
