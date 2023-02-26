use std::{fs::File, path::Path, io::{Read, Seek}};

use cab::{Cabinet, FileEntry};
use cabextract_ce::read_msce000;

fn main() {
    let cabinet = read_cab("PocketEQDemoPPC2002.STRONGARM.cab");
    let msce_file_entry = cabinet.get_000_file_entry().expect("Failed to locate .000 file.");

    println!("Reading {}", msce_file_entry.name());
    let msce = read_msce000(msce_file_entry.name()).unwrap();

    for folder in cabinet.folder_entries() {
        for file in folder.file_entries() {
            if file.name() == msce_file_entry.name() {
                continue;
            }

            let file_ext = Path::new(file.name()).extension().unwrap().to_string_lossy().to_string();
            let file_path =  match msce.file_mapping.get(&file_ext) {
                Some(path) => path,
                _ => "None",
            };

            println!("File {} ({} B) {:?} (compression {:?}) destination {}",
                     file.name(),
                     file.uncompressed_size(),
                     file.datetime(),
                     folder.compression_type(),
                     file_path);
        }
    }

    println!("Done extracting {}", msce.app_name);
}

fn read_cab(name: &str) -> Cabinet<File> {
    let cab_file = File::open(name).unwrap();
    Cabinet::new(cab_file).unwrap()
}

trait Cabinet000 {
    fn get_000_file_entry(&self) -> Option<&FileEntry>;
}

impl<R: Read + Seek> Cabinet000 for Cabinet<R> {
    fn get_000_file_entry(&self) -> Option<&FileEntry> {
        for folder in self.folder_entries() {
            for file in folder.file_entries() {
                if Path::new(file.name()).extension().unwrap().to_string_lossy().to_string() == "000" {
                    return Some(file);
                }
            }
        }
        None
    }
}
