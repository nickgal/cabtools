use binrw::io::{Read, Seek, Write};
use binrw::{BinRead, BinResult, BinWrite, Endian, NullString};
use derive_more::Display;
use encoding_rs::WINDOWS_1252;

#[derive(Clone, Eq, PartialEq, Default, Display, Debug)]
pub struct WinNullString(pub String);

impl BinRead for WinNullString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let null_string = NullString::read_options(reader, endian, ())?;
        let null_string_bytes = <Vec<u8>>::from(null_string);
        let win_string = WINDOWS_1252.decode(&null_string_bytes).0.to_string();
        return Ok(Self(win_string));
    }
}

impl BinWrite for WinNullString {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let encoded_bytes = WINDOWS_1252.encode(&self.0).0;
        encoded_bytes.write_options(writer, endian, args)?;
        0u8.write_options(writer, endian, args)?;

        Ok(())
    }
}
