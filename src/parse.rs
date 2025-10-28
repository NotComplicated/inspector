mod elf;

use crate::error::{Error, Res};

pub trait ByteStream {
    fn next_byte(&mut self) -> Res<u8>;

    fn pull<P: Pull>(&mut self) -> Res<P> {
        P::pull(self)
    }
}

impl<T: Iterator<Item = Res<u8>>> ByteStream for T {
    fn next_byte(&mut self) -> Res<u8> {
        self.next()
            .ok_or_else(|| Error::Eof(std::backtrace::Backtrace::capture()))
            .flatten()
    }
}

pub trait Pull: Sized {
    fn pull<B: ByteStream + ?Sized>(bytes: &mut B) -> Res<Self>;
}

impl<const N: usize> Pull for [u8; N] {
    fn pull<B: ByteStream + ?Sized>(bytes: &mut B) -> Res<Self> {
        let mut pulled_bytes = [0; _];
        for i in 0..N {
            pulled_bytes[i] = bytes.next_byte()?;
        }
        Ok(pulled_bytes)
    }
}

macro_rules! impl_fromstream {
    ($int:ty) => {
        impl Pull for $int {
            fn pull<B: ByteStream + ?Sized>(bytes: &mut B) -> Res<Self> {
                Pull::pull(bytes).map(<$int>::from_le_bytes)
            }
        }
    };
}

impl_fromstream!(u8);
impl_fromstream!(u16);
impl_fromstream!(u32);
impl_fromstream!(u64);

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

pub trait Parser<B: ByteStream> {
    fn parse(&mut self, bytes: B) -> Res<Table>;
}

pub fn start<B: ByteStream>(bytes: B, magic: [u8; 8]) -> Res<Table> {
    Ok((&mut match magic {
        [0x7F, b'E', b'L', b'F', ..] => elf::ElfParser::default(),
        _ => unknown!(),
    } as &mut dyn Parser<B>)
        .parse(bytes)?)
}
