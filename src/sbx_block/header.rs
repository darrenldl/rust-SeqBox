extern crate crc_ccitt;

use super::super::sbx_specs::{Version, SBX_HEADER_SIZE};

#[derive(Debug, Clone)]
pub struct Header {
    pub version   : Version,
    pub crc_ccitt : u16,
    pub file_uid  : [u8; SBX_HEADER_SIZE],
    pub seq_num   : u32
}

impl Header {
    pub fn new(version   : Version,
               file_uid : [u8; SBX_HEADER_SIZE]) -> Header {
        Header {
            version,
            crc_ccitt : 0,
            file_uid  : [0; SBX_HEADER_SIZE],
            seq_num   : 0
        }
    }

    pub fn set_seq_num(&mut self, seq : u32) {
        self.seq_num = seq;
    }
}
