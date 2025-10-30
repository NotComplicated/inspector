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

    fn forward(&mut self, count: usize) -> Res<()> {
        Ok(self.seek_relative(count.try_into().map_err(|_| Error::Seek(count))?)?)
    }

    fn backward(&mut self, count: usize) -> Res<()> {
        Ok(self.seek_relative(-count.try_into().map_err(|_| Error::Seek(count))?)?)
    }

    fn forward_sizeof<T>(&mut self) -> Res<()> {
        self.forward(std::mem::size_of::<T>())
    }

    fn backward_sizeof<T>(&mut self) -> Res<()> {
        self.backward(std::mem::size_of::<T>())
    }

    fn jump(&mut self, pos: u64) -> Res<()> {
        self.seek(std::io::SeekFrom::Start(pos))?;
        Ok(())
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

macro_rules! impl_pull_int {
    ($int:ty) => {
        impl Pull for $int {
            fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self> {
                Pull::pull(bytes).map(<$int>::from_le_bytes)
            }
        }
    };
}
impl_pull_int!(u8);
impl_pull_int!(u16);
impl_pull_int!(u32);
impl_pull_int!(u64);

impl Pull for std::ffi::CString {
    fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self> {
        let mut contents = vec![];
        while let byte = bytes.pull::<u8>()?
            && byte != 0
        {
            contents.push(byte);
        }
        Ok(Self::new(contents).expect("already checked bytes are not null"))
    }
}

#[macro_export]
macro_rules! unknown {
    () => {
        return Err(Error::UnknownFormat(
            std::backtrace::Backtrace::capture(),
            line!(),
        ))
    };
}

pub type Str = std::borrow::Cow<'static, str>;

pub struct Table {
    entries: Vec<(Str, Str)>,
    sections: Vec<Section>,
}

#[derive(Default)]
struct Section {
    name: Option<Str>,
    len: u16,
    width: u16,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            sections: vec![Default::default()],
            entries: vec![],
        }
    }
}

impl Table {
    pub fn display(&self, target: &mut impl std::io::Write) -> Res<()> {
        let mut entries_iter = self.entries.iter();
        let mut first_iter = true;
        for section in &self.sections {
            if first_iter {
                first_iter = false;
            } else if let Some(name) = &section.name {
                writeln!(target, "\n* {name} *")?;
            } else {
                writeln!(target)?;
            }
            for (key, value) in entries_iter.by_ref().take(section.len.into()) {
                write!(target, "{key}:")?;
                target.write_all(&[b' '; 100][..section.width as usize - key.len() + 1])?;
                writeln!(target, "{value}")?;
            }
        }
        Ok(())
    }

    pub fn add_entry(&mut self, key: impl Into<Str>, value: impl Into<Str>) {
        let key = key.into();
        let curr_section = self.sections.last_mut().expect("at least one section");
        curr_section.width = curr_section
            .width
            .max(key.len().try_into().expect("key is <= u16::MAX"));
        curr_section.len += 1;
        self.entries.push((key, value.into()));
    }

    pub fn new_named_section(&mut self, name: impl Into<Str>) {
        self.sections.push(Section {
            name: Some(name.into()),
            ..Default::default()
        });
    }

    pub fn new_unnamed_section(&mut self) {
        self.sections.push(Default::default());
    }
}

pub fn start<B: Bytes>(mut bytes: B, all: bool) -> Res<Table> {
    macro_rules! try_parse {
        ($mod:ident) => {
            if $mod::matching_magic(&mut bytes)? {
                bytes.rewind()?;
                return $mod::Parser::default().parse(bytes, all);
            }
            bytes.rewind()?;
        };
    }

    try_parse!(elf);
    // add parse modules here

    unknown!();
}
