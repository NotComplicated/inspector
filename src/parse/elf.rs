use crate::elf_header::*;
use crate::error::{Error, Res};
use crate::parse::{GenericParser, Parser};
use crate::unknown;

#[derive(Copy, Clone)]
enum WordSize {
    Four,
    Eight,
}

pub struct ElfParser<'p, I> {
    parser: &'p mut GenericParser<I>,
    word_size: Option<WordSize>,
}

impl<'p, I> std::ops::Deref for ElfParser<'p, I> {
    type Target = GenericParser<I>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<'p, I> std::ops::DerefMut for ElfParser<'p, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<'p, I: Iterator<Item = Res<u8>>> Parser for ElfParser<'p, I> {
    fn parse(&mut self) -> Res<()> {
        self.header()
    }
}

impl<'p, I: Iterator<Item = Res<u8>>> ElfParser<'p, I> {
    pub fn new(parser: &'p mut GenericParser<I>) -> Self {
        Self {
            parser,
            word_size: None,
        }
    }

    fn add_word_entry<
        V32: Into<std::borrow::Cow<'static, str>>,
        V64: Into<std::borrow::Cow<'static, str>>,
    >(
        &mut self,
        key: &'static str,
        get_value_32: impl FnOnce(u32) -> Res<V32>,
        get_value_64: impl FnOnce(u64) -> Res<V64>,
    ) -> Res<()> {
        match self.word_size.expect("word size must be set") {
            WordSize::Four => self.add_entry(key, get_value_32),
            WordSize::Eight => self.add_entry(key, get_value_64),
        }
    }

    fn header(&mut self) -> Res<()> {
        if self.pull::<[_; _]>()? != [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3] {
            unknown!();
        }
        let mut word_size = None;
        self.add_entry("Word Size", |size| match size {
            ELFCLASS32 => {
                word_size = Some(WordSize::Four);
                Ok("32 bit")
            }
            ELFCLASS64 => {
                word_size = Some(WordSize::Eight);
                Ok("64 bit")
            }
            _ => unknown!(),
        })?;
        self.word_size = word_size;
        self.add_entry("Endianness", |endianness| match endianness {
            ELFDATA2LSB => Ok("Little"),
            ELFDATA2MSB => Ok("Big"),
            _ => unknown!(),
        })?;
        if self.pull::<u8>()? != EV_CURRENT {
            unknown!();
        }
        self.add_entry("OS ABI", |abi| match abi {
            ELFOSABI_SYSV => Ok("System V"),
            ELFOSABI_HPUX => Ok("HPUX"),
            ELFOSABI_NETBSD => Ok("NetBsd"),
            ELFOSABI_LINUX => Ok("Linux"),
            ELFOSABI_SOLARIS => Ok("Solaris"),
            ELFOSABI_AIX => Ok("AIX"),
            ELFOSABI_IRIX => Ok("Irix"),
            ELFOSABI_FREEBSD => Ok("FreeBsd"),
            ELFOSABI_TRU64 => Ok("Tru64"),
            ELFOSABI_MODESTO => Ok("Modesto"),
            ELFOSABI_OPENBSD => Ok("OpenBsd"),
            ELFOSABI_ARM => Ok("ARM"),
            ELFOSABI_STANDALONE => Ok("Standalone"),
            _ => unknown!(),
        })?;
        self.pull::<[_; 8]>()?;
        self.add_entry("File Type", |ftype| match ftype {
            ET_NONE => Ok("None"),
            ET_REL => Ok("Relocatable"),
            ET_EXEC => Ok("Executable"),
            ET_DYN => Ok("Shared Object"),
            ET_CORE => Ok("Core"),
            _ => unknown!(),
        })?;
        self.add_entry("Architecture", |arch| match arch {
            EM_NONE => Ok("No machine"),
            EM_M32 => Ok("AT&T WE 32100"),
            EM_SPARC => Ok("SUN SPARC"),
            EM_386 => Ok("Intel 80386"),
            EM_68K => Ok("Motorola m68k Family"),
            EM_88K => Ok("Motorola m88k Family"),
            EM_860 => Ok("Intel 80860"),
            EM_MIPS => Ok("MIPS R3000 big-endian"),
            EM_S370 => Ok("IBM System/370"),
            EM_MIPS_RS3_LE => Ok("MIPS R3000 little-endian"),
            EM_PARISC => Ok("HPPA"),
            EM_VPP500 => Ok("Fujitsu VPP500"),
            EM_SPARC32PLUS => Ok("Sun's v8plus"),
            EM_960 => Ok("Intel 80960"),
            EM_PPC => Ok("PowerPC"),
            EM_PPC64 => Ok("PowerPC 64-bit"),
            EM_S390 => Ok("IBM S390"),
            EM_V800 => Ok("NEC V800 series"),
            EM_FR20 => Ok("Fujitsu FR20"),
            EM_RH32 => Ok("TRW RH-32"),
            EM_RCE => Ok("Motorola RCE"),
            EM_ARM => Ok("ARM"),
            EM_FAKE_ALPHA => Ok("Digital Alpha"),
            EM_SH => Ok("Hitachi SH"),
            EM_SPARCV9 => Ok("SPARC v9 64-bit"),
            EM_TRICORE => Ok("Siemens Tricore"),
            EM_ARC => Ok("Argonaut RISC Core"),
            EM_H8_300 => Ok("Hitachi H8/300"),
            EM_H8_300H => Ok("Hitachi H8/300H"),
            EM_H8S => Ok("Hitachi H8S"),
            EM_H8_500 => Ok("Hitachi H8/500"),
            EM_IA_64 => Ok("Intel Merced"),
            EM_MIPS_X => Ok("Stanford MIPS-X"),
            EM_COLDFIRE => Ok("Motorola Coldfire"),
            EM_68HC12 => Ok("Motorola M68HC12"),
            EM_MMA => Ok("Fujitsu MMA Multimedia Accelerator"),
            EM_PCP => Ok("Siemens PCP"),
            EM_NCPU => Ok("Sony nCPU embeeded RISC"),
            EM_NDR1 => Ok("Denso NDR1 microprocessor"),
            EM_STARCORE => Ok("Motorola Start*Core processor"),
            EM_ME16 => Ok("Toyota ME16 processor"),
            EM_ST100 => Ok("STMicroelectronic ST100 processor"),
            EM_TINYJ => Ok("Advanced Logic Corp. Tinyj emb.fam"),
            EM_X86_64 => Ok("AMD x86-64"),
            EM_PDSP => Ok("Sony DSP Processor"),
            EM_FX66 => Ok("Siemens FX66 microcontroller"),
            EM_ST9PLUS => Ok("STMicroelectronics ST9+ 8/16 mc"),
            EM_ST7 => Ok("STmicroelectronics ST7 8 bit mc"),
            EM_68HC16 | EM_68HC11 | EM_68HC08 | EM_68HC05 => Ok("Motorola microcontroller"),
            EM_SVX => Ok("Silicon Graphics SVx"),
            EM_ST19 => Ok("STMicroelectronics ST19 8 bit mc"),
            EM_VAX => Ok("Digital VAX"),
            EM_CRIS => Ok("Axis Communications 32-bit embedded processor"),
            EM_JAVELIN => Ok("Infineon Technologies 32-bit embedded processor"),
            EM_FIREPATH => Ok("Element 14 64-bit DSP Processor"),
            EM_ZSP => Ok("LSI Logic 16-bit DSP Processor"),
            EM_MMIX => Ok("Donald Knuth's educational 64-bit processor"),
            EM_HUANY => Ok("Harvard University machine-independent object files"),
            EM_PRISM => Ok("SiTera Prism"),
            EM_AVR => Ok("Atmel AVR 8-bit microcontroller"),
            EM_FR30 => Ok("Fujitsu FR30"),
            EM_D10V => Ok("Mitsubishi D10V"),
            EM_D30V => Ok("Mitsubishi D30V"),
            EM_V850 => Ok("NEC v850"),
            EM_M32R => Ok("Mitsubishi M32R"),
            EM_MN10300 => Ok("Matsushita MN10300"),
            EM_MN10200 => Ok("Matsushita MN10200"),
            EM_PJ => Ok("picoJava"),
            EM_OPENRISC => Ok("OpenRISC 32-bit embedded processor"),
            EM_ARC_A5 => Ok("ARC Cores Tangent-A5"),
            EM_XTENSA => Ok("Tensilica Xtensa Architecture"),
            _ => unknown!(),
        })?;
        if self.pull::<u32>()? != EV_CURRENT as u32 {
            unknown!();
        }
        self.add_word_entry(
            "Entry Address",
            |addr| Ok(format!("0x{addr:08X}")),
            |addr| Ok(format!("0x{addr:016X}")),
        )?;
        fn fmt_bytes<B: std::fmt::Display>(bytes: B) -> Res<String> {
            Ok(format!("{bytes} bytes"))
        }
        self.add_word_entry("Start of Program Headers", fmt_bytes, fmt_bytes)?;
        self.add_word_entry("Start of Section Headers", fmt_bytes, fmt_bytes)?;

        Ok(())
    }
}
