pub enum Error {
    Cli(String),
    Io(std::path::PathBuf, std::io::Error),
    Eof(std::backtrace::Backtrace),
    UnknownFormat(std::backtrace::Backtrace, u32),
    Other(Box<dyn std::error::Error>),
}

impl<E: std::error::Error + 'static> From<E> for Error {
    fn from(err: E) -> Self {
        Error::Other(Box::new(err))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let print_bt = |f: &mut std::fmt::Formatter, bt: &std::backtrace::Backtrace| {
            if bt.status() == std::backtrace::BacktraceStatus::Captured {
                write!(f, "\n{bt}")?;
            }
            Ok(())
        };
        match self {
            Error::Cli(msg) => write!(f, "CLI error: {msg}"),
            Error::Io(path, err) => write!(f, "IO error while reading {}: {err}", path.display()),
            Error::Eof(bt) => {
                write!(f, "Encountered EOF prematurely")?;
                print_bt(f, bt)
            }
            Error::UnknownFormat(bt, line) => {
                write!(f, "Unknown format")?;
                #[cfg(debug_assertions)]
                {
                    write!(f, "\n(at line {line})")?;
                }
                print_bt(f, bt)
            }
            Error::Other(err) => write!(f, "Error: {err}"),
        }
    }
}

pub type Res<T> = Result<T, Error>;
