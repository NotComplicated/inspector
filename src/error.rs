pub enum Error {
    Cli(String),
    Io(std::io::Error, std::backtrace::Backtrace),
    Seek(usize),
    UnknownFormat(std::backtrace::Backtrace, u32),
    RunCtx(std::path::PathBuf, Box<Error>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err, std::backtrace::Backtrace::capture())
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
            Self::Cli(msg) => write!(f, "CLI error: {msg}"),
            Self::Io(err, bt) => {
                write!(f, "IO error: {err}")?;
                print_bt(f, bt)
            }
            Self::Seek(count) => write!(f, "Seeked too many bytes ({count})"),
            Self::UnknownFormat(bt, line) => {
                write!(f, "Unknown format")?;
                #[cfg(debug_assertions)]
                {
                    write!(f, "\n(at line {line})")?;
                }
                print_bt(f, bt)
            }
            Self::RunCtx(path, err) => write!(f, "{err}\n(while parsing {})", path.display()),
        }
    }
}

pub type Res<T> = Result<T, Error>;
