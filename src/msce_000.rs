use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use crate::strings::WinNullString;
use binrw::{binrw, BinRead, BinResult, BinWrite, Endian, VecArgs};
use derive_more::Into;

#[binrw]
#[brw(little)]
pub struct MSCE000 {
    pub header: Header,
    #[brw(pad_size_to = header.length_app_name)]
    pub app_name: WinNullString,
    #[brw(pad_size_to = header.length_provider)]
    pub provider: WinNullString,
    #[brw(pad_size_to = header.length_unsupported)]
    pub unsupported: WinNullString,
    #[br(count = header.num_entries_strings)]
    pub strings: SharedStrings,
    #[br(args { inner: (strings.clone(),), count: header.num_entries_dirs.into()})]
    pub directories: DirectoryEntries,
    #[br(args { inner: (directories.clone(),), count: header.num_entries_files.into()})]
    pub files: Vec<FileEntry>,
    #[br(count = header.num_entries_reg_hives)]
    pub reg_hives: Vec<RegHiveEntry>,
    #[br(count = header.num_entries_reg_keys)]
    pub reg_keys: Vec<RegKeyEntry>,
    #[br(count = header.num_entries_links)]
    pub links: Vec<LinkEntry>,
    #[br(calc = (|| {
        let mut hm = HashMap::with_capacity(header.num_entries_files.into());
        for file in &files {
            let ext = format!("{:0>3}", file.extension_id);
            hm.insert(ext, file.file_path.clone());
        }
        hm
    })())]
    #[bw(ignore)]
    pub file_mapping: HashMap<String, String>,
}

#[binrw]
#[brw(little, magic = b"MSCE")]
pub struct Header {
    pub unk_04: u32,
    pub file_length: u32,
    pub unk_12: u32,
    pub unk_16: u32,
    pub target_architecture: u32,
    pub min_ce_version_major: u32,
    pub min_ce_version_minor: u32,
    pub max_ce_version_major: u32,
    pub max_ce_version_minor: u32,
    pub min_ce_build_number: u32,
    pub max_ce_build_number: u32,
    pub num_entries_strings: u16,
    pub num_entries_dirs: u16,
    pub num_entries_files: u16,
    pub num_entries_reg_hives: u16,
    pub num_entries_reg_keys: u16,
    pub num_entries_links: u16,
    pub offset_strings: u32,
    pub offset_dirs: u32,
    pub offset_files: u32,
    pub offset_reg_hives: u32,
    pub offset_reg_keys: u32,
    pub offset_links: u32,
    pub offset_app_name: u16,
    pub length_app_name: u16,
    pub offset_provider: u16,
    pub length_provider: u16,
    pub offset_unsupported: u16,
    pub length_unsupported: u16,
    pub unk_96: u16,
    pub unk_98: u16,
}

#[binrw]
#[brw(little)]
pub struct StringEntry {
    pub id: u16,
    pub length: u16,
    pub string: WinNullString,
}

#[derive(Into, Clone)]
pub struct SharedStrings(HashMap<u16, String>);

impl BinRead for SharedStrings {
    type Args<'a> = VecArgs<()>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let count = args.count;
        let mut strings = HashMap::with_capacity(count);
        for _ in 0..count {
            let entry = StringEntry::read_options(reader, endian, ())?;
            strings.insert(entry.id, entry.string.to_string());
        }
        Ok(Self(strings))
    }
}

impl BinWrite for SharedStrings {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        // TODO:
        Ok(())
    }
}

#[derive(Into, Clone)]
pub struct DirectoryEntries(HashMap<u16, String>);

impl BinRead for DirectoryEntries {
    type Args<'a> = VecArgs<(SharedStrings,)>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let count = args.count;
        let mut hm = HashMap::with_capacity(count);
        for _ in 0..count {
            let entry = DirectoryEntry::read_options(reader, endian, args.inner.clone())?;
            hm.insert(entry.id, entry.path);
        }
        Ok(Self(hm))
    }
}

impl BinWrite for DirectoryEntries {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        // TODO:
        Ok(())
    }
}

#[binrw]
#[brw(little)]
#[br(import(strings: SharedStrings))]
pub struct DirectoryEntry {
    pub id: u16,
    pub spec_length: u16,
    #[br(count = spec_length / 2)]
    pub specs: Vec<u16>,
    #[br(calc = specs.iter()
        .filter_map(|string_id| {
            let hm = <HashMap<u16, String>>::from(strings.clone());
            hm.get(string_id).cloned()
        })
        .collect()
    )]
    #[bw(ignore)]
    pub path: String,
}

#[binrw]
#[brw(little)]
#[br(import(diretories: DirectoryEntries))]
pub struct FileEntry {
    pub id: u16,
    pub directory_id: u16,
    pub extension_id: u16,
    pub flags: u32,
    pub name_length: u16,
    #[brw(pad_size_to = name_length.clone())]
    pub name: WinNullString,
    #[br(calc = (|| {
        let hm = <HashMap<u16, String>>::from(diretories.clone());
        let mut path = match hm.get(&directory_id) {
            Some(path) => PathBuf::from(path),
            _ => PathBuf::new()
        };
        path.push(name.to_string());

        String::from(path.to_string_lossy())
    })())]
    #[bw(ignore)]
    pub file_path: String,
}

#[binrw]
#[brw(little)]
pub struct RegHiveEntry {
    pub id: u16,
    pub root: u16,
    pub unk_04: u16,
    pub spec_length: u16,
    #[br(count = spec_length / 2)]
    pub specs: Vec<u16>,
}

#[binrw]
#[brw(little)]
pub struct RegKeyEntry {
    pub id: u16,
    pub hive_id: u16,
    pub variable_substitution: u16,
    pub flags: u32,
    pub data_length: u16,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}

#[binrw]
#[brw(little)]
pub struct LinkEntry {
    pub id: u16,
    pub unk_02: u16,
    pub base_diretory: u16,
    pub target_id: u16,
    pub link_type: u16,
    pub spec_length: u16,
    #[br(count = spec_length / 2)]
    pub specs: Vec<u16>,
}
