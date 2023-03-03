use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

use binrw::BinRead;
use cab::{Cabinet, CompressionType, FileEntry};
use filetime::FileTime;
use msce_000::MSCE000;
use once_cell::sync::Lazy;
use time::OffsetDateTime;

pub mod msce_000;
pub mod strings;

static CE_DIRS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("%CE1%", r"Program Files");
    m.insert("%CE2%", r"Windows");
    m.insert("%CE3%", r"Windows\Desktop");
    m.insert("%CE4%", r"Windows\StartUp");
    m.insert("%CE5%", r"My Documents");
    m.insert("%CE6%", r"Program Files\Accessories");
    m.insert("%CE7%", r"Program Files\Communications");
    m.insert("%CE8%", r"Program Files\Games");
    m.insert("%CE9%", r"Program Files\Pocket Outlook");
    m.insert("%CE10%", r"Program Files\Office");
    m.insert("%CE11%", r"Windows\Programs");
    m.insert("%CE12%", r"Windows\Programs\Accessories");
    m.insert("%CE13%", r"Windows\Programs\Communications");
    m.insert("%CE14%", r"Windows\Programs\Games");
    m.insert("%CE15%", r"Windows\Fonts");
    m.insert("%CE16%", r"Windows\Recent");
    m.insert("%CE17%", r"Windows\Favorites");
    m
});

pub fn read_cab(path: PathBuf) -> Cabinet<File> {
    let cab_file = File::open(path).unwrap();
    Cabinet::new(cab_file).unwrap()
}

pub trait CECabinet {
    fn find_000_manifest(&self) -> Option<&FileEntry>;
    fn read_000_manifest(&self) -> MSCE000;
    fn list_files(&self) -> Vec<WinCECabFileEntry>;
    fn extract_files<P: Into<PathBuf>>(
        &mut self,
        file_entries: &[WinCECabFileEntry],
        output_path: P,
    );
}

impl<R: Read + Seek> CECabinet for Cabinet<R> {
    fn find_000_manifest(&self) -> Option<&FileEntry> {
        for folder in self.folder_entries() {
            for file in folder.file_entries() {
                if file.extension() == "000" {
                    return Some(file);
                }
            }
        }
        None
    }

    fn read_000_manifest(&self) -> MSCE000 {
        let manifest = self
            .find_000_manifest()
            .expect("Failed to locate .000 manifest.");
        let mut file = File::open(manifest.name()).unwrap();
        MSCE000::read(&mut file).unwrap()
    }

    fn list_files(&self) -> Vec<WinCECabFileEntry> {
        let msce = self.read_000_manifest();
        let mut entries = Vec::new();

        for folder in self.folder_entries() {
            for file in folder.file_entries() {
                let file_ext = file.extension();
                let file_path = match msce.file_mapping.get(&file_ext) {
                    Some(path) => path,
                    _ => match file_ext.as_ref() {
                        "000" => "manifest.bin",
                        "999" => "setup.dll",
                        _ => file.name(),
                    },
                };

                let odt = match file.datetime() {
                    Some(pdt) => pdt.assume_utc(),
                    _ => OffsetDateTime::now_utc(),
                };
                let date_time = FileTime::from_unix_time(odt.unix_timestamp(), 0);

                entries.push(WinCECabFileEntry {
                    cab_filename: file.name().to_string(),
                    compression: folder.compression_type(),
                    file_time: Some(date_time),
                    file_size: file.uncompressed_size(),
                    destination: file_path.to_string(),
                });
            }
        }

        entries
    }

    fn extract_files<P: Into<PathBuf>>(
        &mut self,
        file_entries: &[WinCECabFileEntry],
        output_path: P,
    ) {
        let base_path: PathBuf = output_path.into();

        for file_entry in file_entries {
            let mut reader = self.read_file(&file_entry.cab_filename).unwrap();
            let mut output_filepath = PathBuf::from(&base_path);

            let should_expand_ce_variables = true;
            let destination = if should_expand_ce_variables {
                expand_ce_variables(&file_entry.destination).to_string()
            } else {
                file_entry.destination.to_string()
            };

            output_filepath.push(destination);

            println!(
                "Extracting {} ({} B Compression {:?}) to {}",
                file_entry.cab_filename,
                file_entry.file_size,
                file_entry.compression,
                output_filepath.to_string_lossy()
            );

            fs::create_dir_all(output_filepath.parent().unwrap())
                .expect("Failed to create output path.");
            let mut writer = File::create(output_filepath).unwrap();
            io::copy(&mut reader, &mut writer).unwrap();
            filetime::set_file_handle_times(&writer, file_entry.file_time, file_entry.file_time)
                .err();
        }
    }
}

trait FileEntryExtension {
    fn extension(&self) -> String;
}

impl FileEntryExtension for FileEntry {
    fn extension(&self) -> String {
        Path::new(self.name())
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}

fn expand_ce_variables(path: &str) -> Cow<str> {
    CE_DIRS
        .iter()
        .fold(Cow::from(path), |s, (from, to)| s.replace(from, to).into())
}

pub struct WinCECabFileEntry {
    cab_filename: String,
    compression: CompressionType,
    file_time: Option<FileTime>,
    file_size: u32,
    destination: String,
}
