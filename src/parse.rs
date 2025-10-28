mod elf;

use crate::error::{Error, Res};

pub trait Bytes: std::io::BufRead + std::io::Seek {
    fn pull<P: Pull>(&mut self) -> Res<P> {
        P::pull(self)
    }

    fn pull_arr<T, const N: usize>(&mut self) -> Res<[T; N]>
    where
        [T; N]: Pull,
    {
        self.pull()
    }

    fn skip(&mut self, count: impl Into<i64>) -> Res<()> {
        self.seek_relative(count.into()).map_err(Into::into)
    }
}

impl<T: std::io::BufRead + std::io::Seek> Bytes for T {}

pub trait Pull: Sized {
    fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self>;
}

impl<const N: usize> Pull for [u8; N] {
    fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self> {
        let mut pulled = [0; _];
        bytes.read_exact(&mut pulled)?;
        Ok(pulled)
    }
}

macro_rules! impl_pull {
    ($int:ty) => {
        impl Pull for $int {
            fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self> {
                Pull::pull(bytes).map(<$int>::from_le_bytes)
            }
        }
    };
}

impl_pull!(u8);
impl_pull!(u16);
impl_pull!(u32);
impl_pull!(u64);

#[macro_export]
macro_rules! unknown {
    () => {
        return Err(Error::UnknownFormat(
            std::backtrace::Backtrace::capture(),
            line!(),
        ))
    };
}

#[derive(Default)]
pub struct Table {
    keys: Vec<&'static str>,
    values: Vec<std::borrow::Cow<'static, str>>,
    width: usize,
}

impl Table {
    pub fn display(&self, target: &mut impl std::io::Write) -> Res<()> {
        for (key, value) in self.keys.iter().zip(&self.values) {
            write!(target, "{key}:")?;
            target.write_all(&[b' '; 100][..self.width - key.len() + 1])?;
            writeln!(target, "{value}")?;
        }
        Ok(())
    }

    pub fn add_entry(
        &mut self,
        key: &'static str,
        value: impl Into<std::borrow::Cow<'static, str>>,
    ) {
        self.keys.push(key);
        self.values.push(value.into());
        self.width = self.width.max(key.len());
    }
}

pub fn start<B: Bytes>(mut bytes: B) -> Res<Table> {
    macro_rules! try_parse {
        ($mod:tt) => {
            if $mod::matching_magic(&mut bytes)? {
                bytes.rewind()?;
                return $mod::Parser::default().parse(bytes);
            }
            bytes.rewind()?;
        };
    }
    try_parse!(elf);
    unknown!();
}
