use crate::{
    elf_header::*,
    error::{Error, Res},
    parse::{Bytes, Pull, Str, Table},
    unknown,
};

const MAGIC: [u8; 4] = [ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3];

pub fn matching_magic(bytes: &mut impl Bytes) -> Res<bool> {
    Ok(bytes.pull::<[_; _]>()? == MAGIC)
}

#[derive(Copy, Clone, Debug)]
enum WordSize {
    Four,
    Eight,
}

#[repr(u32)]
#[derive(Debug)]
enum SegmentType {
    Null = PT_NULL,
    Load = PT_LOAD,
    Dynamic = PT_DYNAMIC,
    Interp = PT_INTERP,
    Note = PT_NOTE,
    ShLib = PT_SHLIB,
    PHdr = PT_PHDR,
    Tls = PT_TLS,
    GnuEhFrame = PT_GNU_EH_FRAME,
    GnuStack = PT_GNU_STACK,
    GnuRelRo = PT_GNU_RELRO,
    Unknown,
}

impl Pull for SegmentType {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        Ok(match bytes.pull()? {
            PT_NULL => Self::Null,
            PT_LOAD => Self::Load,
            PT_DYNAMIC => Self::Dynamic,
            PT_INTERP => Self::Interp,
            PT_NOTE => Self::Note,
            PT_SHLIB => Self::ShLib,
            PT_PHDR => Self::PHdr,
            PT_TLS => Self::Tls,
            PT_GNU_EH_FRAME => Self::GnuEhFrame,
            PT_GNU_STACK => Self::GnuStack,
            PT_GNU_RELRO => Self::GnuRelRo,
            _ => Self::Unknown,
        })
    }
}

#[derive(Debug)]
struct ProgramHeader {
    r#type: SegmentType,
    flags: u32,
    offset: u64,
}

impl Pull for ProgramHeader {
    type Format = WordSize;

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, word_size: Self::Format) -> Res<Self> {
        let r#type = bytes.pull()?;
        let flags;
        let offset;
        match word_size {
            WordSize::Four => {
                offset = bytes.pull::<u32>()?.into();
                bytes.forward_sizeof::<[u32; 4]>()?;
                flags = bytes.pull()?;
                bytes.forward_sizeof::<u32>()?;
            }
            WordSize::Eight => {
                flags = bytes.pull()?;
                offset = bytes.pull()?;
                bytes.forward_sizeof::<[u64; 5]>()?;
            }
        }
        Ok(Self {
            r#type,
            flags,
            offset,
        })
    }
}

#[repr(u32)]
#[derive(PartialEq, Debug)]
enum SectionType {
    Null = SHT_NULL,
    ProgBits = SHT_PROGBITS,
    SymTab = SHT_SYMTAB,
    StrTab = SHT_STRTAB,
    Rela = SHT_RELA,
    Hash = SHT_HASH,
    Dynamic = SHT_DYNAMIC,
    Note = SHT_NOTE,
    NoBits = SHT_NOBITS,
    Rel = SHT_REL,
    ShLib = SHT_SHLIB,
    DynSym = SHT_DYNSYM,
    InitArray = SHT_INIT_ARRAY,
    FiniArray = SHT_FINI_ARRAY,
    PreinitArray = SHT_PREINIT_ARRAY,
    Group = SHT_GROUP,
    SymTabShNdx = SHT_SYMTAB_SHNDX,
    GnuAttributes = SHT_GNU_ATTRIBUTES,
    GnuHash = SHT_GNU_HASH,
    GnuLibList = SHT_GNU_LIBLIST,
    Checksum = SHT_CHECKSUM,
    GnuVerDef = SHT_GNU_VERDEF,
    GnuVerNeed = SHT_GNU_VERNEED,
    GnuVerSym = SHT_GNU_VERSYM,
    Unknown,
}

impl Pull for SectionType {
    type Format = ();

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, _: Self::Format) -> Res<Self> {
        Ok(match bytes.pull()? {
            SHT_NULL => Self::Null,
            SHT_PROGBITS => Self::ProgBits,
            SHT_SYMTAB => Self::SymTab,
            SHT_STRTAB => Self::StrTab,
            SHT_RELA => Self::Rela,
            SHT_HASH => Self::Hash,
            SHT_DYNAMIC => Self::Dynamic,
            SHT_NOTE => Self::Note,
            SHT_NOBITS => Self::NoBits,
            SHT_REL => Self::Rel,
            SHT_SHLIB => Self::ShLib,
            SHT_DYNSYM => Self::DynSym,
            SHT_INIT_ARRAY => Self::InitArray,
            SHT_FINI_ARRAY => Self::FiniArray,
            SHT_PREINIT_ARRAY => Self::PreinitArray,
            SHT_GROUP => Self::Group,
            SHT_SYMTAB_SHNDX => Self::SymTabShNdx,
            SHT_GNU_ATTRIBUTES => Self::GnuAttributes,
            SHT_GNU_HASH => Self::GnuHash,
            SHT_GNU_LIBLIST => Self::GnuLibList,
            SHT_CHECKSUM => Self::Checksum,
            SHT_GNU_VERDEF => Self::GnuVerDef,
            SHT_GNU_VERNEED => Self::GnuVerNeed,
            SHT_GNU_VERSYM => Self::GnuVerSym,
            _ => Self::Unknown,
        })
    }
}

#[derive(Debug)]
struct SectionHeader {
    name: u32,
    r#type: SectionType,
    flags: u64,
    offset: u64,
    size: u64,
}

impl Pull for SectionHeader {
    type Format = WordSize;

    fn pull_fmt<B: Bytes + ?Sized>(bytes: &mut B, word_size: Self::Format) -> Res<Self> {
        let name = bytes.pull()?;
        let r#type = bytes.pull()?;
        let flags;
        let offset;
        let size;
        match word_size {
            WordSize::Four => {
                flags = bytes.pull::<u32>()?.into();
                bytes.forward_sizeof::<u32>()?;
                offset = bytes.pull::<u32>()?.into();
                size = bytes.pull::<u32>()?.into();
                bytes.forward_sizeof::<[u32; 4]>()?;
            }
            WordSize::Eight => {
                flags = bytes.pull()?;
                bytes.forward_sizeof::<u64>()?;
                offset = bytes.pull()?;
                size = bytes.pull()?;
                bytes.forward_sizeof::<[u32; 2]>()?;
                bytes.forward_sizeof::<[u64; 2]>()?;
            }
        }
        Ok(Self {
            name,
            r#type,
            flags,
            offset,
            size,
        })
    }
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
    fn add_word_entry<V32: Into<Str>, V64: Into<Str>>(
        &mut self,
        table: &mut Table,
        key: impl Into<Str>,
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

    pub fn parse(&mut self, mut bytes: impl Bytes, all: bool) -> Res<Table> {
        let mut table = Default::default();
        self.header(&mut bytes, &mut table)?;
        if all {
            self.pheaders(&mut bytes, &mut table)?;
            self.sheaders(&mut bytes, &mut table)?;
        }
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
        bytes.jump(self.ph_offset)?;
        for i in 0..self.ph_count {
            table.new_named_section(format!("Program Segment {}/{}", i + 1, self.ph_count));
            let pheader: ProgramHeader =
                bytes.pull_via(self.word_size.expect("word size assigned"))?;

            table.add_entry(
                "Type",
                match pheader.r#type {
                    SegmentType::Null => "NULL",
                    SegmentType::Load => "LOAD",
                    SegmentType::Dynamic => "DYNAMIC",
                    SegmentType::Interp => "INTERP",
                    SegmentType::Note => "NOTE",
                    SegmentType::ShLib => "SHLIB",
                    SegmentType::PHdr => "PHDR",
                    SegmentType::Tls => "TLS",
                    SegmentType::GnuEhFrame => "GNU_EH_FRAME",
                    SegmentType::GnuStack => "GNU_STACK",
                    SegmentType::GnuRelRo => "GNU_RELRO",
                    SegmentType::Unknown => "Unknown",
                },
            );

            let flags: String = [
                (pheader.flags & PF_R > 0, "Read"),
                (pheader.flags & PF_W > 0, "Write"),
                (pheader.flags & PF_X > 0, "Execute"),
            ]
            .iter()
            .filter_map(|&(enabled, flag)| enabled.then_some(flag))
            .enumerate()
            .flat_map(|(i, flag)| [if i == 0 { "" } else { ", " }, flag])
            .collect();
            if flags.is_empty() {
                table.add_entry("Flags", "None");
            } else {
                table.add_entry("Flags", flags);
            }

            match pheader.r#type {
                SegmentType::Interp => {
                    let curr_pos = bytes.stream_position()?;
                    bytes.jump(pheader.offset)?;
                    let interpreter = match bytes.pull::<std::ffi::CString>()?.into_string() {
                        Ok(string) => string,
                        Err(err) => err.into_cstring().to_string_lossy().into_owned(),
                    };
                    table.add_entry("Interpreter", interpreter);
                    bytes.jump(curr_pos)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn sheaders(&mut self, bytes: &mut impl Bytes, table: &mut Table) -> Res<()> {
        let name_strtab_header_addr =
            self.sh_idx_str_table as u64 * self.sh_size as u64 + self.sh_offset;
        bytes.jump(name_strtab_header_addr)?;
        let name_strtab_header: SectionHeader =
            bytes.pull_via(self.word_size.expect("word size assigned"))?;
        if name_strtab_header.r#type != SectionType::StrTab {
            unknown!();
        }
        bytes.jump(name_strtab_header.offset)?; // jump to sheader name strtable
        let name_strtab = {
            let size = name_strtab_header
                .size
                .try_into()
                .expect("size is within usize::MAX");
            let mut strtab = vec![0; size];
            bytes.read_exact(&mut strtab)?;
            strtab
        };

        bytes.jump(self.sh_offset)?;
        let mut total_size = 0;
        for i in 0..self.sh_count {
            table.new_named_section(format!("Section {}/{}", i + 1, self.sh_count));
            let sheader: SectionHeader =
                bytes.pull_via(self.word_size.expect("word size assigned"))?;
            total_size += sheader.size;

            let Ok(name) = std::ffi::CStr::from_bytes_until_nul(
                &name_strtab[sheader.name.try_into().expect("u32 -> usize")..],
            ) else {
                unknown!()
            };
            table.add_entry("Name", name.to_string_lossy().into_owned());

            table.add_entry(
                "Type",
                match sheader.r#type {
                    SectionType::Null => "NULL",
                    SectionType::ProgBits => "PROGBITS",
                    SectionType::SymTab => "SYMTAB",
                    SectionType::StrTab => "STRTAB",
                    SectionType::Rela => "RELA",
                    SectionType::Hash => "HASH",
                    SectionType::Dynamic => "DYNAMIC",
                    SectionType::Note => "NOTE",
                    SectionType::NoBits => "NOBITS",
                    SectionType::Rel => "REL",
                    SectionType::ShLib => "SHLIB",
                    SectionType::DynSym => "DYNSYM",
                    SectionType::InitArray => "INITARRAY",
                    SectionType::FiniArray => "FINIARRAY",
                    SectionType::PreinitArray => "PREINITARRAY",
                    SectionType::Group => "GROUP",
                    SectionType::SymTabShNdx => "SYMTABSHNDX",
                    SectionType::GnuAttributes => "GNU_ATTRIBUTES",
                    SectionType::GnuHash => "GNU_HASH",
                    SectionType::GnuLibList => "GNU_LIBLIST",
                    SectionType::Checksum => "CHECKSUM",
                    SectionType::GnuVerDef => "GNU_VERDEF",
                    SectionType::GnuVerNeed => "GNU_VERNEED",
                    SectionType::GnuVerSym => "GNU_VERSYM",
                    SectionType::Unknown => "Unknown",
                },
            );
            let flags: String = [
                (sheader.flags & SHF_WRITE > 0, "Write"),
                (sheader.flags & SHF_ALLOC > 0, "Alloc"),
                (sheader.flags & SHF_EXECINSTR > 0, "Exec"),
                (sheader.flags & SHF_MERGE > 0, "Merge"),
                (sheader.flags & SHF_STRINGS > 0, "Strings"),
                (sheader.flags & SHF_INFO_LINK > 0, "Info Link"),
                (sheader.flags & SHF_LINK_ORDER > 0, "Link Order"),
                (sheader.flags & SHF_OS_NONCONFORMING > 0, "OS Nonconforming"),
                (sheader.flags & SHF_GROUP > 0, "Group"),
                (sheader.flags & SHF_TLS > 0, "TLS"),
                (sheader.flags & SHF_ORDERED > 0, "Ordered"),
                (sheader.flags & SHF_EXCLUDE > 0, "Exclude"),
            ]
            .iter()
            .filter_map(|&(enabled, flag)| enabled.then_some(flag))
            .enumerate()
            .flat_map(|(i, flag)| [if i == 0 { "" } else { ", " }, flag])
            .collect();
            if flags.is_empty() {
                table.add_entry("Flags", "None");
            } else {
                table.add_entry("Flags", flags);
            }

            table.add_entry("Size", format!("{} bytes", sheader.size));

            match name.to_bytes() {
                _ => {}
            }
        }

        table.new_unnamed_section();
        table.add_entry("Total Size of Sections", format!("{} bytes", total_size));

        Ok(())
    }
}
