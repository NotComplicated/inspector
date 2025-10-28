use crate::elf_header::*;
use crate::error::{Error, Res};
use crate::parse::{Bytes, Table};
use crate::unknown;

pub fn matching_magic(bytes: &mut impl Bytes) -> Res<bool> {
    Ok(bytes.pull_arr()? == [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3])
}

#[derive(Copy, Clone)]
enum WordSize {
    Four,
    Eight,
}

#[derive(Default)]
pub struct Parser {
    word_size: Option<WordSize>,
}

impl Parser {
    fn add_word_entry<
        V32: Into<std::borrow::Cow<'static, str>>,
        V64: Into<std::borrow::Cow<'static, str>>,
    >(
        &mut self,
        table: &mut Table,
        key: &'static str,
        bytes: &mut impl Bytes,
        get_value_32: impl FnOnce(u32) -> Res<V32>,
        get_value_64: impl FnOnce(u64) -> Res<V64>,
    ) -> Res<()> {
        match self.word_size.expect("word size must be set") {
            WordSize::Four => table.add_entry(key, get_value_32(bytes.pull()?)?),
            WordSize::Eight => table.add_entry(key, get_value_64(bytes.pull()?)?),
        }
        Ok(())
    }

    pub fn parse(&mut self, mut bytes: impl Bytes) -> Res<Table> {
        let mut table = Default::default();
        self.header(&mut bytes, &mut table)?;
        Ok(table)
    }

    fn header(&mut self, bytes: &mut impl Bytes, table: &mut Table) -> Res<()> {
        bytes.pull_arr::<_, 4>()?; // ignore magic
        let (word_size, entry_value) = match bytes.pull()? {
            ELFCLASS32 => (WordSize::Four, "32 bit"),
            ELFCLASS64 => (WordSize::Eight, "64 bit"),
            _ => unknown!(),
        };
        table.add_entry("Word Size", entry_value);
        self.word_size = Some(word_size);
        table.add_entry(
            "Endianness",
            match bytes.pull()? {
                ELFDATA2LSB => "Little",
                ELFDATA2MSB => "Big",
                _ => unknown!(),
            },
        );
        if bytes.pull::<u8>()? != EV_CURRENT {
            unknown!();
        }
        table.add_entry(
            "OS ABI",
            match bytes.pull()? {
                ELFOSABI_SYSV => "System V",
                ELFOSABI_HPUX => "HPUX",
                ELFOSABI_NETBSD => "NetBsd",
                ELFOSABI_LINUX => "Linux",
                ELFOSABI_SOLARIS => "Solaris",
                ELFOSABI_AIX => "AIX",
                ELFOSABI_IRIX => "Irix",
                ELFOSABI_FREEBSD => "FreeBsd",
                ELFOSABI_TRU64 => "Tru64",
                ELFOSABI_MODESTO => "Modesto",
                ELFOSABI_OPENBSD => "OpenBsd",
                ELFOSABI_ARM => "ARM",
                ELFOSABI_STANDALONE => "Standalone",
                _ => unknown!(),
            },
        );
        bytes.pull::<[_; 8]>()?; // padding
        table.add_entry(
            "File Type",
            match bytes.pull()? {
                ET_NONE => "None",
                ET_REL => "Relocatable",
                ET_EXEC => "Executable",
                ET_DYN => "Shared Object",
                ET_CORE => "Core",
                _ => unknown!(),
            },
        );
        table.add_entry(
            "Architecture",
            match bytes.pull()? {
                EM_NONE => "No machine",
                EM_M32 => "AT&T WE 32100",
                EM_SPARC => "SUN SPARC",
                EM_386 => "Intel 80386",
                EM_68K => "Motorola m68k Family",
                EM_88K => "Motorola m88k Family",
                EM_860 => "Intel 80860",
                EM_MIPS => "MIPS R3000 big-endian",
                EM_S370 => "IBM System/370",
                EM_MIPS_RS3_LE => "MIPS R3000 little-endian",
                EM_PARISC => "HPPA",
                EM_VPP500 => "Fujitsu VPP500",
                EM_SPARC32PLUS => "Sun's v8plus",
                EM_960 => "Intel 80960",
                EM_PPC => "PowerPC",
                EM_PPC64 => "PowerPC 64-bit",
                EM_S390 => "IBM S390",
                EM_V800 => "NEC V800 series",
                EM_FR20 => "Fujitsu FR20",
                EM_RH32 => "TRW RH-32",
                EM_RCE => "Motorola RCE",
                EM_ARM => "ARM",
                EM_FAKE_ALPHA => "Digital Alpha",
                EM_SH => "Hitachi SH",
                EM_SPARCV9 => "SPARC v9 64-bit",
                EM_TRICORE => "Siemens Tricore",
                EM_ARC => "Argonaut RISC Core",
                EM_H8_300 => "Hitachi H8/300",
                EM_H8_300H => "Hitachi H8/300H",
                EM_H8S => "Hitachi H8S",
                EM_H8_500 => "Hitachi H8/500",
                EM_IA_64 => "Intel Merced",
                EM_MIPS_X => "Stanford MIPS-X",
                EM_COLDFIRE => "Motorola Coldfire",
                EM_68HC12 => "Motorola M68HC12",
                EM_MMA => "Fujitsu MMA Multimedia Accelerator",
                EM_PCP => "Siemens PCP",
                EM_NCPU => "Sony nCPU embeeded RISC",
                EM_NDR1 => "Denso NDR1 microprocessor",
                EM_STARCORE => "Motorola Start*Core processor",
                EM_ME16 => "Toyota ME16 processor",
                EM_ST100 => "STMicroelectronic ST100 processor",
                EM_TINYJ => "Advanced Logic Corp. Tinyj emb.fam",
                EM_X86_64 => "AMD x86-64",
                EM_PDSP => "Sony DSP Processor",
                EM_FX66 => "Siemens FX66 microcontroller",
                EM_ST9PLUS => "STMicroelectronics ST9+ 8/16 mc",
                EM_ST7 => "STmicroelectronics ST7 8 bit mc",
                EM_68HC16 | EM_68HC11 | EM_68HC08 | EM_68HC05 => "Motorola microcontroller",
                EM_SVX => "Silicon Graphics SVx",
                EM_ST19 => "STMicroelectronics ST19 8 bit mc",
                EM_VAX => "Digital VAX",
                EM_CRIS => "Axis Communications 32-bit embedded processor",
                EM_JAVELIN => "Infineon Technologies 32-bit embedded processor",
                EM_FIREPATH => "Element 14 64-bit DSP Processor",
                EM_ZSP => "LSI Logic 16-bit DSP Processor",
                EM_MMIX => "Donald Knuth's educational 64-bit processor",
                EM_HUANY => "Harvard University machine-independent object files",
                EM_PRISM => "SiTera Prism",
                EM_AVR => "Atmel AVR 8-bit microcontroller",
                EM_FR30 => "Fujitsu FR30",
                EM_D10V => "Mitsubishi D10V",
                EM_D30V => "Mitsubishi D30V",
                EM_V850 => "NEC v850",
                EM_M32R => "Mitsubishi M32R",
                EM_MN10300 => "Matsushita MN10300",
                EM_MN10200 => "Matsushita MN10200",
                EM_PJ => "picoJava",
                EM_OPENRISC => "OpenRISC 32-bit embedded processor",
                EM_ARC_A5 => "ARC Cores Tangent-A5",
                EM_XTENSA => "Tensilica Xtensa Architecture",
                _ => unknown!(),
            },
        );
        if bytes.pull::<u32>()? != EV_CURRENT as u32 {
            unknown!();
        }
        self.add_word_entry(
            table,
            "Entry Address",
            bytes,
            |addr| Ok(format!("0x{addr:08X}")),
            |addr| Ok(format!("0x{addr:016X}")),
        )?;
        fn fmt_bytes<B: std::fmt::Display>(bytes: B) -> Res<String> {
            Ok(format!("{bytes} bytes"))
        }
        self.add_word_entry(
            table,
            "Start of Program Headers",
            bytes,
            fmt_bytes,
            fmt_bytes,
        )?;
        self.add_word_entry(
            table,
            "Start of Section Headers",
            bytes,
            fmt_bytes,
            fmt_bytes,
        )?;
        bytes.pull::<u32>()?; // flags, unimplemented
        bytes.pull::<u16>()?; // header size
        let ph_size: u16 = bytes.pull()?;
        let ph_count: u16 = bytes.pull()?;
        let sh_size: u16 = bytes.pull()?;
        let sh_count: u16 = bytes.pull()?;

        Ok(())
    }
}
