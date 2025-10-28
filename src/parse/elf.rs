use crate::elf_header::*;
use crate::error::{Error, Res};
use crate::parse::{Bytes, Pull, Table};
use crate::unknown;

const MAGIC: [u8; 4] = [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3];

pub fn matching_magic(bytes: &mut impl Bytes) -> Res<bool> {
    Ok(bytes.pull_arr()? == MAGIC)
}

#[repr(C)]
#[derive(Debug)]
struct ProgramHeader32 {
    r#type: u32,
    offset: u32,
    vaddr: u32,
    paddr: u32,
    filesz: u32,
    memsz: u32,
    flags: u32,
    align: u32,
}

#[repr(C)]
#[derive(Debug)]
struct ProgramHeader64 {
    r#type: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

impl Pull for ProgramHeader32 {
    fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self> {
        let r#type = bytes.pull()?;
        let offset = bytes.pull()?;
        let vaddr = bytes.pull()?;
        let paddr = bytes.pull()?;
        let filesz = bytes.pull()?;
        let memsz = bytes.pull()?;
        let flags = bytes.pull()?;
        let align = bytes.pull()?;
        Ok(Self {
            r#type,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            flags,
            align,
        })
    }
}

impl Pull for ProgramHeader64 {
    fn pull<B: Bytes + ?Sized>(bytes: &mut B) -> Res<Self> {
        let r#type = bytes.pull()?;
        let flags = bytes.pull()?;
        let offset = bytes.pull()?;
        let vaddr = bytes.pull()?;
        let paddr = bytes.pull()?;
        let filesz = bytes.pull()?;
        let memsz = bytes.pull()?;
        let align = bytes.pull()?;
        Ok(Self {
            r#type,
            flags,
            offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
        })
    }
}

#[derive(Copy, Clone, Debug)]
enum WordSize {
    Four,
    Eight,
}

#[derive(Default, Debug)]
pub struct Parser {
    word_size: Option<WordSize>,
    ph_offset: u64,
    ph_size: u16,
    ph_count: u16,
    sh_offset: u64,
    sh_size: u16,
    sh_count: u16,
    sh_idx_str_table: u16,
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
        get_value_32: impl FnOnce(&mut Self, u32) -> Res<V32>,
        get_value_64: impl FnOnce(&mut Self, u64) -> Res<V64>,
    ) -> Res<()> {
        match self.word_size.expect("word size must be set") {
            WordSize::Four => table.add_entry(key, get_value_32(self, bytes.pull()?)?),
            WordSize::Eight => table.add_entry(key, get_value_64(self, bytes.pull()?)?),
        }
        Ok(())
    }

    pub fn parse(&mut self, mut bytes: impl Bytes) -> Res<Table> {
        let mut table = Default::default();
        self.header(&mut bytes, &mut table)?;
        self.pheaders(&mut bytes, &mut table)?;
        Ok(table)
    }

    fn header(&mut self, bytes: &mut impl Bytes, table: &mut Table) -> Res<()> {
        bytes.forward(MAGIC.len())?; // ignore magic
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
        bytes.forward(8)?; // padding
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
            |_, addr| Ok(format!("0x{addr:08X}")),
            |_, addr| Ok(format!("0x{addr:016X}")),
        )?;
        fn fmt_byte_count<B: std::fmt::Display>(byte_count: B) -> Res<String> {
            Ok(format!("{byte_count} bytes"))
        }
        self.add_word_entry(
            table,
            "Start of Program Headers",
            bytes,
            |this, ph_offset| {
                this.ph_offset = ph_offset.into();
                fmt_byte_count(ph_offset)
            },
            |this, ph_offset| {
                this.ph_offset = ph_offset;
                fmt_byte_count(ph_offset)
            },
        )?;
        self.add_word_entry(
            table,
            "Start of Section Headers",
            bytes,
            |this, sh_offset| {
                this.sh_offset = sh_offset.into();
                fmt_byte_count(sh_offset)
            },
            |this, sh_offset| {
                this.sh_offset = sh_offset;
                fmt_byte_count(sh_offset)
            },
        )?;
        bytes.forward_sizeof::<u32>()?; // flags, unimplemented
        bytes.forward_sizeof::<u16>()?; // header size
        self.ph_size = bytes.pull()?;
        self.ph_count = bytes.pull()?;
        self.sh_size = bytes.pull()?;
        self.sh_count = bytes.pull()?;
        self.sh_idx_str_table = bytes.pull()?;

        Ok(())
    }

    fn pheaders(&mut self, bytes: &mut impl Bytes, table: &mut Table) -> Res<()> {
        bytes.seek(std::io::SeekFrom::Start(self.ph_offset))?;
        let pheader: ProgramHeader64 = bytes.pull()?;
        dbg!(pheader);
        dbg!(self);

        Ok(())
    }
}
