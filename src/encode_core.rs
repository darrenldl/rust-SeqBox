use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::fs;
use std::fmt;
use super::file_utils;
use std::io::SeekFrom;

use integer_utils::IntegerUtils;

use super::progress_report;

use super::SmallVec;

use std::time::UNIX_EPOCH;

use super::file_reader;
use super::file_writer;

use super::multihash;

use super::Error;
use super::sbx_specs::Version;
use super::time_utils;
use super::rs_codec::RSCodec;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Sender,
                      channel};

use super::sbx_block::{Block, BlockType};
use super::sbx_block;
use super::sbx_block::metadata::Metadata;
use super::sbx_specs::{SBX_FILE_UID_LEN,
                       SBX_LARGEST_BLOCK_SIZE,
                       ver_to_block_size};

use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub version               : Version,
    pub meta_blocks_written   : u32,
    pub data_blocks_written   : u32,
    pub parity_blocks_written : u32,
    pub data_bytes_encoded    : u64,
    pub total_blocks          : u32,
    pub start_time            : f64,
    pub time_elapsed          : f64,
    pub data_shards           : usize,
    pub parity_shards         : usize
}

impl fmt::Display for Stats {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub version      : Version,
    pub file_uid     : [u8; SBX_FILE_UID_LEN],
    pub rs_enabled   : bool,
    pub rs_data      : usize,
    pub rs_parity    : usize,
    pub hash_enabled : bool,
    pub hash_type    : multihash::HashType,
    pub in_file      : String,
    pub out_file     : String,
    pub silence_level : progress_report::SilenceLevel
}

impl Stats {
    pub fn new(param : &Param, file_metadata : &fs::Metadata) -> Stats {
        let total_blocks =
            file_utils::calc_block_count(param.version, file_metadata) as u32;
        Stats {
            version               : param.version,
            meta_blocks_written   : 0,
            data_blocks_written   : 0,
            parity_blocks_written : 0,
            data_bytes_encoded    : 0,
            total_blocks,
            start_time            : time_utils::get_time_now(),
            time_elapsed          : 0.,
            data_shards           : 0,
            parity_shards         : 0,
        }
    }

    pub fn set_time_elapsed(&mut self) {
        self.time_elapsed = time_utils::get_time_now() - self.start_time;
    }
}

fn pack_metadata(block         : &mut Block,
                 param         : &Param,
                 stats         : &Stats,
                 file_metadata : &fs::Metadata,
                 hash          : Option<multihash::HashBytes>) {
    let meta = block.meta_mut().unwrap();

    { // add file name
        meta.push(Metadata::FNM(param
                                .in_file
                                .clone()
                                .into_bytes()
                                .into_boxed_slice())); }
    { // add sbx file name
        meta.push(Metadata::SNM(param
                                .out_file
                                .clone()
                                .into_bytes()
                                .into_boxed_slice())); }
    { // add file size
        meta.push(Metadata::FSZ(file_metadata
                                .len())); }
    { // add file last modifcation time
        match file_metadata.modified() {
            Ok(t)  => match t.duration_since(UNIX_EPOCH) {
                Ok(t)  => meta.push(Metadata::FDT(t.as_secs() as i64)),
                Err(_) => {}
            },
            Err(_) => {} }}
    { // add sbx encoding time
        meta.push(Metadata::SDT(stats.start_time as i64)); }
    { // add hash
        if param.hash_enabled {
            let hsh = match hash {
                Some(hsh) => hsh,
                None      => {
                    let ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();
                    ctx.finish_into_hash_bytes()
                }
            };
            meta.push(Metadata::HSH(hsh)); }}
    { // add RS params
        if param.rs_enabled {
            meta.push(Metadata::RSD(param.rs_data   as u8));
            meta.push(Metadata::RSP(param.rs_parity as u8)); }}
}

fn write_metadata_block(param         : &Param,
                        stats         : &Stats,
                        file_metadata : &fs::Metadata,
                        hash          : Option<multihash::HashBytes>,
                        buf           : &mut [u8]) {
    let mut block = Block::new(param.version,
                               &param.file_uid,
                               BlockType::Meta);
    pack_metadata(&mut block,
                  param,
                  stats,
                  file_metadata,
                  hash);
    block.sync_to_buffer(None, buf).unwrap();
}

fn make_reporter(param         : &Param,
                 stats         : &Arc<Mutex<Stats>>,
                 tx_error      : &Sender<Option<Error>>,
                 shutdown_flag : &Arc<AtomicBool>)
                 -> JoinHandle<()> {
    use progress_report::ProgressElement::*;

    let tx_error = tx_error.clone();

    let header = "Data encoding progress";
    let unit   = "chunks";
    let stats = Arc::clone(stats);
    let silence_settings =
        progress_report::silence_level_to_settings(param.silence_level);
    let mut progress_report_context =
        progress_report::Context::new(
            String::from(header),
            stats.lock().unwrap().start_time,
            String::from(unit),
            vec![ProgressBar, Percentage, CurrentRateShort, TimeUsedShort, TimeLeftShort],
            vec![TimeUsedLong, AverageRateLong]
        );
    let total_blocks = stats.lock().unwrap().total_blocks;
    let shutdown_flag = Arc::clone(shutdown_flag);

    thread::spawn(move || {
        loop {
            worker_stop!(graceful_if_shutdown =>
                         tx_error, shutdown_flag);

            thread::sleep(Duration::from_millis(300));

            progress_report::print_progress(&silence_settings,
                                            &mut progress_report_context,
                                            stats.lock().unwrap().data_blocks_written as u64,
                                            total_blocks as u64);
        }

        progress_report::print_progress(&silence_settings,
                                        &mut progress_report_context,
                                        stats.lock().unwrap().data_blocks_written as u64,
                                        total_blocks as u64);
    })
}

pub fn encode_file(param    : &Param)
                   -> Result<Stats, Error> {
    let metadata = file_utils::get_file_metadata(&param.in_file)?;

    let stats = Arc::new(Mutex::new(Stats::new(param, &metadata)));

    // set up file reader and writer
    let mut reader = file_reader::FileReader::new(&param.in_file)?;
    let mut writer = file_writer::FileWriter::new(&param.out_file)?;

    // setup reporter
    let (tx_error, _) = channel::<Option<Error>>();
    let shutdown_flag        = Arc::new(AtomicBool::new(false));
    let reporter = make_reporter(param, &stats, &tx_error, &shutdown_flag);

    // set up hash state
    let mut hash_ctx =
        multihash::hash::Ctx::new(param.hash_type).unwrap();

    // setup Reed-Solomon things
    let mut rs_codec_meta = RSCodec::new(1, 2, 1);
    let mut rs_codec_data = RSCodec::new(param.rs_data,
                                         param.rs_parity,
                                         file_utils::calc_block_count(param.version,
                                                                      &metadata));

    let mut parity_buf : Vec<Box<[u8]>> = Vec::with_capacity(param.rs_parity);
    if param.rs_enabled {
        for _ in 0..param.rs_parity {
            parity_buf.push(vec![0; ver_to_block_size(param.version)]
                            .into_boxed_slice());
        }
    }

    let mut parity_buf_meta : [[u8; SBX_LARGEST_BLOCK_SIZE]; 2] =
        [[0; SBX_LARGEST_BLOCK_SIZE]; 2];

    let mut parity_meta : SmallVec<[&mut [u8]; 2]> = SmallVec::new();
    for p in parity_buf_meta.iter_mut() {
        parity_meta.push(sbx_block::slice_data_buf_mut(param.version, p));
    }

    // setup data buffer
    let mut data : [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    // setup main data block
    let mut block = Block::new(param.version,
                               &param.file_uid,
                               BlockType::Data);

    let mut cur_seq_num : u32 = 1;

    { // write dummy metadata block
        write_metadata_block(param,
                             &stats.lock().unwrap(),
                             &metadata,
                             None,
                             &mut data);
        writer.write(sbx_block::slice_buf(param.version, &data))?;

        stats.lock().unwrap().meta_blocks_written += 1;

        if param.rs_enabled {
            let data_part = sbx_block::slice_data_buf(param.version, &data);
            let parity_to_use =
                rs_codec_meta.encode(data_part, &mut parity_meta).unwrap();

            for i in 0..parity_to_use {
                block.header.seq_num = u32::use_then_add1(&mut cur_seq_num);
                block.sync_to_buffer(None, &mut parity_buf[i]).unwrap();

                // write data out
                writer.write(sbx_block::slice_buf(param.version, &parity_buf[i]))?;
            }
        }

        stats.lock().unwrap().parity_blocks_written += 2;
    }

    loop {
        let mut data_blocks_written   = 0;
        let mut parity_blocks_written = 0;

        // read data in
        let len_read =
            reader.read(sbx_block::slice_data_buf_mut(param.version, &mut data))?;

        if len_read == 0 {
            break;
        }

        sbx_block::write_padding(param.version, len_read, &mut data);

        // start encoding
        block.header.seq_num = u32::use_then_add1(&mut cur_seq_num);
        data_blocks_written += 1;
        block.sync_to_buffer(None, &mut data).unwrap();

        // write data out
        writer.write(sbx_block::slice_buf(param.version, &data))?;

        // update hash state if needed
        if param.hash_enabled {
            let data_part = sbx_block::slice_data_buf(param.version, &data);
            hash_ctx.update(data_part);
        }

        // update Reed-Solomon data if needed
        if param.rs_enabled {
            let data_part = sbx_block::slice_data_buf(param.version, &data);
            let res = {
                let mut parity : SmallVec<[&mut [u8]; 32]> = SmallVec::new();
                for p in parity_buf.iter_mut() {
                    parity.push(sbx_block::slice_data_buf_mut(param.version, p));
                }
                rs_codec_data.encode(data_part, &mut parity)
            };
            if let Some(parity_to_use) = res {
                for i in 0..parity_to_use {
                    block.header.seq_num = u32::use_then_add1(&mut cur_seq_num);
                    parity_blocks_written += 1;
                    block.sync_to_buffer(None, &mut parity_buf[i]).unwrap();

                    // write data out
                    writer.write(sbx_block::slice_buf(param.version, &parity_buf[i]))?;
                }
            }
        }

        // update stats
        stats.lock().unwrap().data_blocks_written   += data_blocks_written;
        stats.lock().unwrap().parity_blocks_written += parity_blocks_written;
    }

    { // write actual metadata block
        write_metadata_block(param,
                             &stats.lock().unwrap(),
                             &metadata,
                             Some(hash_ctx.finish_into_hash_bytes()),
                             &mut data);

        writer.seek(SeekFrom::Start(0))?;

        writer.write(sbx_block::slice_buf(param.version, &data))?;

        cur_seq_num = 1;

        if param.rs_enabled {
            let data_part = sbx_block::slice_data_buf(param.version, &data);
            let parity_to_use =
                rs_codec_meta.encode(data_part, &mut parity_meta).unwrap();

            for i in 0..parity_to_use {
                block.header.seq_num = u32::use_then_add1(&mut cur_seq_num);
                block.sync_to_buffer(None, &mut parity_buf[i]).unwrap();

                // write data out
                writer.write(sbx_block::slice_buf(param.version, &parity_buf[i]))?;
            }
        }
    }

    // shutdown reporter
    shutdown_flag.store(true, Ordering::Relaxed);

    reporter.join().unwrap();

    let stats = stats.lock().unwrap().clone();
    Ok(stats)
}
