use std;
use super::super::sbx_specs::{Version, SBX_FILE_UID_LEN, SBX_SIGNATURE};
use super::super::sbx_specs;
use super::BlockType;

use std::num::Wrapping;

use super::crc::*;

use super::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub version  : Version,
    pub crc      : u16,
    pub file_uid : [u8; SBX_FILE_UID_LEN],
    pub seq_num  : Wrapping<u32>,
}

impl Header {
    pub fn new(version   : Version,
               file_uid : [u8; SBX_FILE_UID_LEN]) -> Header {
        Header {
            version,
            crc       : 0,
            file_uid,
            seq_num   : Wrapping(0)
        }
    }

    pub fn to_bytes(&self, buffer : &mut [u8]) -> Result<(), Error> {
        if buffer.len() != 16 {
            return Err(Error::IncorrectBufferSize);
        }

        { // signature
            buffer[0..3].copy_from_slice(SBX_SIGNATURE); }
        { // version byte
            buffer[3] = sbx_specs::ver_to_usize(self.version) as u8; }
        { // crc ccitt
            let crc : [u8; 2] =
                unsafe { std::mem::transmute::<u16, [u8; 2]>(self.crc.to_be()) };
            buffer[4..6].copy_from_slice(&crc); }
        { // file uid
            buffer[6..12].copy_from_slice(&self.file_uid); }
        { // seq num
            let seq_num : [u8; 4] =
                unsafe { std::mem::transmute::<u32, [u8; 4]>(self.seq_num.0.to_be()) };
            buffer[12..16].copy_from_slice(&seq_num); }

        Ok(())
    }

    pub fn from_bytes(&mut self, buffer : &[u8]) -> Result<(), Error> {
        use super::Error;
        use nom::IResult::*;

        if buffer.len() != 16 {
            return Err(Error::IncorrectBufferSize);
        }

        match parsers::header_p(buffer) {
            Done(_, header) => { *self = header;
                                 Ok(()) },
            _               => Err(Error::ParseError)
        }
    }

    pub fn calc_crc(&self) -> u16 {
        let crc = sbx_crc_ccitt(self.version, &self.file_uid);
        let seq_num : [u8; 4] =
            unsafe { std::mem::transmute::<u32, [u8; 4]>(self.seq_num.0.to_be()) };
        crc_ccitt_generic(crc, &seq_num)
    }

    pub fn header_type(&self) -> BlockType {
        if self.seq_num.0 == 0 {
            BlockType::Meta
        } else {
            BlockType::Data
        }
    }

    pub fn is_meta(&self) -> bool {
        match self.header_type() {
            BlockType::Data => false,
            BlockType::Meta => true
        }
    }

    pub fn is_data(&self) -> bool {
        match self.header_type() {
            BlockType::Data => true,
            BlockType::Meta => false
        }
    }
}

mod parsers {
    use nom::{be_u16, be_u32};
    use super::Header;
    use super::Version;

    use std::num::Wrapping;

    named!(sig_p,
           tag!(b"SBx")
    );

    named!(ver_p <Version>,
           alt!(
               do_parse!(_v : tag!(&[ 1]) >> (Version::V1))  |
               do_parse!(_v : tag!(&[ 2]) >> (Version::V2))  |
               do_parse!(_v : tag!(&[ 3]) >> (Version::V3))  |
               do_parse!(_v : tag!(&[11]) >> (Version::V11)) |
               do_parse!(_v : tag!(&[12]) >> (Version::V12)) |
               do_parse!(_v : tag!(&[13]) >> (Version::V13))
           )
    );

    named!(uid_p,
           take!(6)
    );

    named!(pub header_p <Header>,
           do_parse!(
               _sig : sig_p >>
                   version      : ver_p >>
                   crc          : be_u16 >>
                   file_uid_raw : uid_p >>
                   seq_num      : be_u32 >>
                   ({
                       let mut file_uid : [u8; 6] = [0; 6];
                       file_uid.copy_from_slice(file_uid_raw);
                       Header {
                           version,
                           crc,
                           file_uid,
                           seq_num : Wrapping(seq_num)
                       }
                   })
           )
    );
}
