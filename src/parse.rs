mod elf;

use crate::error::{Error, Res};

trait Pull: Sized {
    fn pull(bytes: &mut impl Iterator<Item = Res<u8>>) -> Res<Self>;
}

impl<const N: usize> Pull for [u8; N] {
    fn pull(bytes: &mut impl Iterator<Item = Res<u8>>) -> Res<Self> {
        let mut pulled_bytes = [0; _];
        for i in 0..N {
            pulled_bytes[i] = bytes
                .next()
                .ok_or(Error::Eof(std::backtrace::Backtrace::capture()))
                .flatten()?;
        }
        Ok(pulled_bytes)
    }
}

macro_rules! impl_fromstream {
    ($int:ty) => {
        impl Pull for $int {
            fn pull(bytes: &mut impl Iterator<Item = Res<u8>>) -> Res<Self> {
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

pub struct GenericParser<I> {
    bytes: I,
    keys: Vec<&'static str>,
    values: Vec<std::borrow::Cow<'static, str>>,
    width: usize,
}

pub trait Parser {
    fn parse(&mut self) -> Res<()>;
}

impl<I: Iterator<Item = Res<u8>>> GenericParser<I> {
    pub fn new(bytes: I) -> Self {
        Self {
            bytes,
            keys: vec![],
            values: vec![],
            width: 0,
        }
    }

    pub fn display(&self, target: &mut impl std::io::Write) -> Res<()> {
        for (key, value) in self.keys.iter().zip(&self.values) {
            write!(target, "{key}:")?;
            target.write_all(&[b' '; 100][..self.width - key.len() + 1])?;
            writeln!(target, "{value}")?;
        }
        Ok(())
    }

    fn pull<P: Pull>(&mut self) -> Res<P> {
        P::pull(&mut self.bytes)
    }

    fn add_entry<P: Pull, V: Into<std::borrow::Cow<'static, str>>>(
        &mut self,
        key: &'static str,
        get_value: impl FnOnce(P) -> Res<V>,
    ) -> Res<()> {
        self.keys.push(key);
        let p = self.pull()?;
        self.values.push(get_value(p)?.into());
        self.width = self.width.max(key.len());
        Ok(())
    }

    pub fn start(&mut self, magic: [u8; 8]) -> Res<()> {
        (&mut match magic {
            _ => elf::ElfParser::new(self),
        } as &mut dyn Parser)
            .parse()?;
        Ok(())
    }
}
