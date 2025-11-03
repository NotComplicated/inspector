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
#[derive(Debug)]
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
#[derive(Debug)]
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

type Color = [u8; 3];

#[derive(Debug)]
enum Chunk {
    Ihdr {
        width: u32,
        height: u32,
        bit_depth: BitDepth,
        color_type: ColorType,
    },
    Plte(Vec<Color>),
    Idat(usize),
    Gama(f32),
    Iend,
    Unknown,
}

impl Pull for Chunk {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        let len: usize = bytes
            .pull_via::<u32>(Endianness::Big)?
            .try_into()
            .expect("u32 -> usize");
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
                bytes.forward(1)?; // interlace
                Self::Ihdr {
                    width,
                    height,
                    bit_depth,
                    color_type,
                }
            }
            b"PLTE" => Self::Plte((0..len / 3).map(|_| bytes.pull()).collect::<Res<_>>()?),
            b"IDAT" => {
                bytes.forward(len)?;
                Self::Idat(len)
            }
            b"GAMA" => {
                let gamma: u32 = bytes.pull_via(Endianness::Big)?;
                Self::Gama(gamma as f32 / 100_000.0)
            }
            b"IEND" => Self::Iend,
            _ => {
                bytes.forward(len)?;
                Self::Unknown
            }
        };

        bytes.forward(4)?; // crc
        Ok(chunk)
    }
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn parse(self, mut bytes: impl Bytes, all: bool) -> Res<Table> {
        let mut table = Table::default();
        bytes.forward(std::mem::size_of_val(&MAGIC))?;
        let mut total_len = 0;
        let mut img_gamma = None;
        loop {
            match bytes.pull()? {
                Chunk::Ihdr {
                    width,
                    height,
                    bit_depth,
                    color_type,
                } => {
                    table.add_entry("Width", format!("{width} px"));
                    table.add_entry("Height", format!("{height} px"));
                    table.add_entry(
                        "Bit Depth",
                        match bit_depth {
                            BitDepth::One => "1",
                            BitDepth::Two => "2",
                            BitDepth::Four => "4",
                            BitDepth::Eight => "8",
                            BitDepth::Sixteen => "16",
                        },
                    );
                    table.add_entry(
                        "Color Type",
                        match color_type {
                            ColorType::Grayscale => "Grayscale",
                            ColorType::Rgb => "RGB",
                            ColorType::Palette => "Palette",
                            ColorType::GrayscaleAlpha => "Grayscale Alpha",
                            ColorType::RgbAlpha => "RGBA",
                        },
                    );
                    if !all {
                        break;
                    }
                }
                Chunk::Plte(palette) => {
                    table.new_named_section("Palette");
                    for (i, [r, g, b]) in palette.iter().enumerate() {
                        table.add_entry(
                            format!("Color {}", i + 1),
                            format!("0x{r:02X}{g:02X}{b:02X}"),
                        );
                    }
                }
                Chunk::Idat(len) => total_len += len,
                Chunk::Gama(gamma) => img_gamma = Some(gamma),
                Chunk::Iend => break,
                Chunk::Unknown => {}
            }
        }
        if all {
            table.new_unnamed_section();
            table.add_entry("Total IDAT Size", format!("{total_len} bytes"));
            if let Some(gamma) = img_gamma {
                table.add_entry("Gamma", format!("{gamma}"));
            }
        }

        Ok(table)
    }
}
