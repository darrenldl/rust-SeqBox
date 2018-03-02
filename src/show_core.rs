use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use super::misc_utils;
use super::misc_utils::RequiredLenAndSeekTo;

use std::io::SeekFrom;

use super::progress_report::*;

use super::file_reader::FileReader;
use super::file_reader::FileReaderParam;
use super::file_writer::FileWriter;
use super::file_writer::FileWriterParam;

use super::sbx_specs::SBX_SCAN_BLOCK_SIZE;

use super::multihash;
use super::multihash::*;

use super::Error;
use super::sbx_specs::Version;

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_specs::SBX_LARGEST_BLOCK_SIZE;
use super::sbx_specs::{ver_to_block_size,
                       ver_to_data_size,
                       ver_supports_rs,
                       ver_to_usize};

use std::str::from_utf8;

use super::time_utils;
use super::block_utils;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub bytes_processed : u64,
    pub total_bytes     : u64,
    meta_block_count    : u64,
    start_time          : f64,
    end_time            : f64,
}

impl Stats {
    pub fn new(file_metadata : &fs::Metadata) -> Stats {
        Stats {
            bytes_processed   : 0,
            total_bytes       : file_metadata.len(),
            meta_block_count  : 0,
            start_time        : 0.,
            end_time          : 0.,
        }
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 { &mut self.start_time }

    fn end_time_mut(&mut self)   -> &mut f64 { &mut self.end_time }

    fn units_so_far(&self)       -> u64      { self.bytes_processed }

    fn total_units(&self)        -> u64      { self.total_bytes }
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        if self.meta_block_count == 0 {
            writeln!(f, "No metadata blocks found")
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    show_all       : bool,
    force_misalign : bool,
    from_pos       : Option<u64>,
    to_pos         : Option<u64>,
    in_file        : String,
    silence_level  : SilenceLevel
}

impl Param {
    pub fn new(show_all       : bool,
               force_misalign : bool,
               from_pos       : Option<u64>,
               to_pos         : Option<u64>,
               in_file        : &str,
               silence_level  : SilenceLevel) -> Param {
        Param {
            show_all,
            force_misalign,
            from_pos,
            to_pos,
            in_file : String::from(in_file),
            silence_level,
        }
    }
}

pub fn show_file(param : &Param)
                 -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;

    let stats = Arc::new(Mutex::new(Stats::new(&metadata)));

    let reporter = ProgressReporter::new(&stats,
                                         "Metadata block scanning progress",
                                         "bytes",
                                         param.silence_level);

    let mut block = Block::dummy();
    let mut buffer : [u8; SBX_LARGEST_BLOCK_SIZE] =
        [0; SBX_LARGEST_BLOCK_SIZE];

    let mut reader = FileReader::new(&param.in_file,
                                     FileReaderParam { write    : false,
                                                       buffered : true   })?;

    // calulate length to read and position to seek to
    let RequiredLenAndSeekTo { required_len, seek_to } =
        misc_utils::calc_required_len_and_seek_to_from_byte_range(param.from_pos,
                                                                  param.to_pos,
                                                                  param.force_misalign,
                                                                  stats.lock().unwrap().bytes_processed,
                                                                  metadata.len());

    // seek to calculated position
    reader.seek(SeekFrom::Start(seek_to))?;

    reporter.start();

    let mut meta_block_count : u64 = 0;

    let mut block_pos       : u64;
    let mut bytes_processed : u64 = 0;

    loop {
        let lazy_read_res = block_utils::read_block_lazily(&mut block,
                                                           &mut buffer,
                                                           &mut reader)?;

        block_pos        = bytes_processed;
        bytes_processed += lazy_read_res.len_read as u64;

        stats.lock().unwrap().bytes_processed = bytes_processed;

        if bytes_processed > required_len {
            break;
        }

        if lazy_read_res.eof     { break; }

        if !lazy_read_res.usable { continue; }

        if block.is_meta() {
            reporter.pause();

            if param.show_all {
                println!();
                println!("Metadata block number : {}", meta_block_count);
                println!("========================================");
            }

            println!(                      "Found at byte          : {} (0x{:X})",
                                            block_pos,
                                            block_pos);
            println!();
            println!(                      "File UID               : {}",
                                            misc_utils::bytes_to_upper_hex_string(
                                                &block.get_file_uid()));
            match block.get_FNM().unwrap() {
                None    => println!(       "File name              : N/A"),
                Some(x) => {
                    match from_utf8(&x) {
                        Ok(x)  => println!("File name              : {}", x),
                        Err(_) => println!("File name              : Invalid UTF-8 string")
                    }
                }
            }
            match block.get_SNM().unwrap() {
                None    => println!(       "SBX container name     : N/A"),
                Some(x) => {
                    match from_utf8(&x) {
                        Ok(x)  => println!("SBX container name     : {}", x),
                        Err(_) => println!("SBX container name     : Invalid UTF-8 string")
                    }
                }
            }
            println!(                      "SBX container version  : {}",
                                            ver_to_usize(block.get_version()));
            match block.get_FSZ().unwrap() {
                None    => println!(       "File size              : N/A"),
                Some(x) => println!(       "File size              : {}", x)
            }
            match block.get_FDT().unwrap() {
                None    => println!(       "File modification time : N/A"),
                Some(x) => println!(       "File modification time : {}", x)
            }
            match block.get_SDT().unwrap() {
                None    => println!(       "SBX encoding time      : N/A"),
                Some(x) => println!(       "SBX encoding time      : {}", x)
            }
            match block.get_HSH().unwrap() {
                None    => println!(       "Hash                   : N/A"),
                Some(h) => println!(       "Hash                   : {} - {}",
                                            hash_type_to_string(h.0),
                                            misc_utils::bytes_to_lower_hex_string(&h.1))
            }

            meta_block_count += 1;

            reporter.resume();

            if !param.show_all { break; }
        }
    }

    reporter.stop();

    stats.lock().unwrap().meta_block_count = meta_block_count;

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
