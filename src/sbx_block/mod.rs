mod helper;
mod header;
mod metadata;
mod crc;
mod test;

use self::header::Header;
use self::metadata::Metadata;

use super::sbx_specs::{Version, SBX_HEADER_SIZE, SBX_FILE_UID_LEN};
extern crate reed_solomon_erasure;
extern crate smallvec;
use self::smallvec::SmallVec;

use self::crc::*;

use super::sbx_specs;

#[derive(Clone, Copy, Debug)]
pub enum BlockType {
    Data, Meta
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    IncorrectBlockType,
    Metadata(metadata::Error),
    IncorrectBufferSize,
}

#[derive(Debug)]
pub enum Data<'a> {
    Data(&'a [u8]),
    Meta(SmallVec<[Metadata; 16]>, &'a mut [u8])
}

#[derive(Debug)]
pub struct Block<'a> {
    header     : Header,
    data       : Data<'a>,
    header_buf : &'a mut [u8],
}

impl<'a> Block<'a> {
    pub fn new(version    : Version,
               file_uid   : &[u8; SBX_FILE_UID_LEN],
               block_type : BlockType,
               buffer     : &'a mut [u8])
               -> Result<Block<'a>, Error> {
        if buffer.len() != sbx_specs::ver_to_block_size(version) {
            return Err(Error::IncorrectBufferSize);
        }

        Ok(match block_type {
            BlockType::Data => {
                let (header_buf, data_buf) = buffer.split_at_mut(SBX_HEADER_SIZE);
                Block {
                    header     : Header::new(version, file_uid.clone()),
                    data       : Data::Data(data_buf),
                    header_buf : header_buf,
                }
            },
            BlockType::Meta => {
                let (header_buf, data_buf) = buffer.split_at_mut(16);
                Block {
                    header     : Header::new(version, file_uid.clone()),
                    data       : Data::Meta(SmallVec::new(), data_buf),
                    header_buf : header_buf,
                }
            }
        })
    }

    pub fn block_type(&self) -> BlockType {
        match self.data {
            Data::Data(_)    => BlockType::Data,
            Data::Meta(_, _) => BlockType::Meta
        }
    }

    pub fn is_meta(&self) -> bool {
        match self.block_type() {
            BlockType::Data => false,
            BlockType::Meta => true
        }
    }

    pub fn is_data(&self) -> bool {
        match self.block_type() {
            BlockType::Data => true,
            BlockType::Meta => false
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn add_meta(&mut self,
                     meta : Metadata) -> Result<(), Error> {
        match self.data {
            Data::Data(_) => Err(Error::IncorrectBlockType),
            Data::Meta(ref mut x, _) => {
                x.push(meta);
                Ok(())
            }
        }
    }

    pub fn crc_ccitt(&self) -> u16 {
        match self.data {
            Data::Meta(_, ref buf) => {
                let crc = self.header.crc_ccitt();
                crc_ccitt_generic(crc, buf)
            },
            Data::Data(buf) => {
                let crc = self.header.crc_ccitt();
                crc_ccitt_generic(crc, buf)
            }
        }
    }

    pub fn sync_to_buffer(&mut self) -> Result<(), Error> {
        match self.data {
            Data::Meta(ref meta, ref mut buf) => {
                // transform metadata to bytes
                if let Err(x) = metadata::write_to_bytes(meta, buf) {
                    return Err(Error::Metadata(x));
                }
            },
            Data::Data(_) => {}
        }

        self.header.crc = self.crc_ccitt();

        self.header.write_to_bytes(&mut self.header_buf);

        Ok(())
    }

    pub fn sync_from_buffer(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn verify_crc(&self) -> bool {
        let crc = self.crc_ccitt();

        self.header.crc == crc
    }
}
