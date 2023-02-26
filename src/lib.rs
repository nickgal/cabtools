use std::fs::File;

use binrw::{io::BufReader, BinRead, BinResult};
use msce_000::MSCE000;

pub mod msce_000;
pub mod strings;

pub fn read_msce000(name: &str) -> BinResult<MSCE000> {
    let mut reader = read_file(name);
    MSCE000::read(&mut reader)
}

fn read_file(name: &str) -> BufReader<File> {
    let f = File::open(name).expect("Failed to open");
    BufReader::new(f)
}
