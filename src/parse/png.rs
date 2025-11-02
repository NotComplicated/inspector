use crate::{
    error::{Error, Res},
    parse::{Bytes, Endianness, Pull, Table},
    unknown,
};

const MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

pub fn matching_magic(bytes: &mut impl Bytes) -> Res<bool> {
    Ok(bytes.pull::<[_; _]>()? == MAGIC)
}

#[repr(u8)]
enum BitDepth {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
}

impl Pull for BitDepth {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        Ok(match bytes.pull()? {
            1u8 => Self::One,
            2 => Self::Two,
            4 => Self::Four,
            8 => Self::Eight,
            16 => Self::Sixteen,
            _ => unknown!(),
        })
    }
}

#[repr(u8)]
enum ColorType {
    Grayscale = 0,
    Rgb = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RgbAlpha = 6,
}

impl Pull for ColorType {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        Ok(match bytes.pull()? {
            0u8 => Self::Grayscale,
            2 => Self::Rgb,
            3 => Self::Palette,
            4 => Self::GrayscaleAlpha,
            6 => Self::RgbAlpha,
            _ => unknown!(),
        })
    }
}

#[repr(u8)]
enum Interlace {
    None = 0,
    Adam7 = 1,
}

impl Pull for Interlace {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        Ok(match bytes.pull()? {
            0u8 => Self::None,
            1 => Self::Adam7,
            _ => unknown!(),
        })
    }
}

type Color = [u8; 3];

enum Chunk {
    Ihdr {
        width: u32,
        height: u32,
        bit_depth: BitDepth,
        color_type: ColorType,
        interlace: Interlace,
    },
    Plte(Vec<Color>),
    Idat(u32),
    Iend,
    Gama(f32),
    Unknown,
}

impl Pull for Chunk {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        let len: u32 = bytes.pull_via(Endianness::Big)?;
        let r#type = {
            let mut r#type = bytes.pull::<[u8; 4]>()?;
            r#type.make_ascii_uppercase();
            r#type
        };

        let chunk = match &r#type {
            b"IHDR" => {
                let width = bytes.pull_via(Endianness::Big)?;
                let height = bytes.pull_via(Endianness::Big)?;
                let bit_depth = bytes.pull()?;
                let color_type = bytes.pull()?;
                // compression
                if bytes.pull::<u8>()? != 0 {
                    unknown!();
                }
                // filter
                if bytes.pull::<u8>()? != 0 {
                    unknown!();
                }
                let interlace = bytes.pull()?;
                Self::Ihdr {
                    width,
                    height,
                    bit_depth,
                    color_type,
                    interlace,
                }
            }
            b"PLTE" => Self::Plte((0..len / 3).map(|_| bytes.pull()).collect::<Res<_>>()?),
            b"IDAT" => {
                bytes.forward(len.try_into().expect("u32 -> usize"))?;
                Self::Idat(len)
            }
            b"IEND" => Self::Iend,
            b"GAMA" => {
                let gamma = bytes.pull_via::<u32>(Endianness::Big)?;
                Self::Gama(gamma as f32 / 100_000.0)
            }
            _ => Self::Unknown,
        };
        bytes.forward(4)?; // crc
        Ok(chunk)
    }
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn parse(self, mut bytes: impl Bytes, all: bool) -> Res<Table> {
        let mut table = Default::default();
        bytes.forward(std::mem::size_of_val(&MAGIC))?;
        println!("foo");
        Ok(table)
    }
}
