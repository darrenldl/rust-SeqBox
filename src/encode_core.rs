use crate::file_utils;
use crate::misc_utils;
use crate::time_utils;
use std::fmt;
use std::fs;
use std::io::SeekFrom;
use std::sync::{Arc, Mutex};

use crate::misc_utils::RequiredLenAndSeekTo;

use crate::json_printer::{BracketType, JSONPrinter};

use crate::progress_report::*;

use crate::cli_utils::setup_ctrlc_handler;
use std::time::UNIX_EPOCH;

use crate::file_reader::{FileReader, FileReaderParam};
use crate::file_writer::{FileWriter, FileWriterParam};
use crate::reader::{Reader, ReaderType};

use crate::multihash;

use crate::general_error::Error;
use crate::sbx_specs::Version;

use crate::sbx_block::{
    calc_data_block_write_pos, make_too_much_meta_err_string, Block, BlockType, Metadata,
};

use crate::sbx_block;
use crate::sbx_specs::{
    ver_forces_meta_enabled, ver_to_block_size, ver_to_data_size, ver_to_max_data_file_size,
    ver_to_usize, ver_uses_rs, SBX_FILE_UID_LEN, SBX_LARGEST_BLOCK_SIZE,
};

use crate::misc_utils::{PositionOrLength, RangeEnd};

use rayon::prelude::*;

use reed_solomon_erasure::ReedSolomon;

use smallvec::SmallVec;

const DEFAULT_SINGLE_LOT_SIZE: usize = 10;

#[derive(Clone, Debug)]
pub struct Stats {
    uid: [u8; SBX_FILE_UID_LEN],
    version: Version,
    chunk_size: u64,
    hash_bytes: Option<multihash::HashBytes>,
    pub meta_blocks_written: u64,
    pub data_blocks_written: u64,
    pub parity_blocks_written: u64,
    pub data_padding_bytes: usize,
    pub in_file_size: u64,
    pub out_file_size: u64,
    total_data_blocks: Option<u64>,
    start_time: f64,
    end_time: f64,
    json_printer: Arc<JSONPrinter>,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rs_enabled = ver_uses_rs(self.version);
        let block_size = ver_to_block_size(self.version);
        let data_size = ver_to_data_size(self.version);
        let meta_blocks_written = self.meta_blocks_written;
        let data_blocks_written = self.data_blocks_written;
        let parity_blocks_written = self.parity_blocks_written;
        let blocks_written = meta_blocks_written + data_blocks_written + parity_blocks_written;
        let data_bytes_encoded = self.data_bytes_encoded();
        let in_file_size = self.in_file_size;
        let out_file_size = self.out_file_size;
        let time_elapsed = (self.end_time - self.start_time) as i64;
        let (hour, minute, second) = time_utils::seconds_to_hms(time_elapsed);

        let json_printer = &self.json_printer;

        json_printer.write_open_bracket(f, Some("stats"), BracketType::Curly)?;

        if rs_enabled {
            write_maybe_json!(
                f,
                json_printer,
                "File UID                               : {}",
                misc_utils::bytes_to_upper_hex_string(&self.uid)
                    => force_quotes
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX version                            : {}",
                ver_to_usize(self.version)
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Block size used in encoding            : {}",
                block_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Data  size used in encoding            : {}",
                data_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written               : {}",
                blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (metadata)    : {}",
                meta_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (data)        : {}",
                data_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (parity)      : {}",
                parity_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Amount of data encoded (bytes)         : {}",
                data_bytes_encoded
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "File size                              : {}",
                in_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX container size                     : {}",
                out_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Hash                                   : {}",
                match self.hash_bytes {
                    None => null_if_json_else_NA!(json_printer).to_string(),
                    Some(ref h) => format!(
                        "{} - {}",
                        multihash::hash_type_to_string(h.0),
                        misc_utils::bytes_to_lower_hex_string(&h.1)
                    ),
                }
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Time elapsed                           : {:02}:{:02}:{:02}",
                hour,
                minute,
                second
            )?;
        } else {
            write_maybe_json!(
                f,
                json_printer,
                "File UID                            : {}",
                misc_utils::bytes_to_upper_hex_string(&self.uid)
                    => force_quotes
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX version                         : {}",
                ver_to_usize(self.version)
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Block size used in encoding         : {}",
                block_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Data  size used in encoding         : {}",
                data_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written            : {}",
                blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (metadata) : {}",
                meta_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Number of blocks written (data)     : {}",
                data_blocks_written
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Amount of data encoded (bytes)      : {}",
                data_bytes_encoded
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "File size                           : {}",
                in_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "SBX container size                  : {}",
                out_file_size
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Hash                                : {}",
                match self.hash_bytes {
                    None => null_if_json_else_NA!(json_printer).to_string(),
                    Some(ref h) => format!(
                        "{} - {}",
                        multihash::hash_type_to_string(h.0),
                        misc_utils::bytes_to_lower_hex_string(&h.1)
                    ),
                }
            )?;
            write_maybe_json!(
                f,
                json_printer,
                "Time elapsed                        : {:02}:{:02}:{:02}",
                hour,
                minute,
                second
            )?;
        }

        json_printer.write_close_bracket(f)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    version: Version,
    uid: [u8; SBX_FILE_UID_LEN],
    data_par_burst: Option<(usize, usize, usize)>,
    rs_enabled: bool,
    meta_enabled: bool,
    json_printer: Arc<JSONPrinter>,
    hash_type: multihash::HashType,
    from_pos: Option<u64>,
    to_pos: Option<RangeEnd<u64>>,
    in_file: Option<String>,
    out_file: String,
    pr_verbosity_level: PRVerbosityLevel,
}

impl Param {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        json_printer: &Arc<JSONPrinter>,
        hash_type: multihash::HashType,
        from_pos: Option<u64>,
        to_pos: Option<RangeEnd<u64>>,
        in_file: Option<&str>,
        out_file: &str,
        pr_verbosity_level: PRVerbosityLevel,
    ) -> Param {
        Param {
            version,
            uid: uid.clone(),
            data_par_burst,
            rs_enabled: ver_uses_rs(version),
            meta_enabled: ver_forces_meta_enabled(version) || meta_enabled,
            json_printer: Arc::clone(json_printer),
            hash_type,
            from_pos,
            to_pos,
            in_file: match in_file {
                None => None,
                Some(f) => Some(String::from(f)),
            },
            out_file: String::from(out_file),
            pr_verbosity_level,
        }
    }
}

impl Stats {
    pub fn new(param: &Param, required_len: Option<u64>) -> Stats {
        use crate::file_utils::from_orig_file_size::calc_data_chunk_count;
        Stats {
            uid: param.uid,
            version: param.version,
            chunk_size: ver_to_data_size(param.version) as u64,
            hash_bytes: None,
            meta_blocks_written: 0,
            data_blocks_written: 0,
            parity_blocks_written: 0,
            data_padding_bytes: 0,
            total_data_blocks: match required_len {
                Some(len) => Some(calc_data_chunk_count(param.version, len)),
                None => None,
            },
            in_file_size: 0,
            out_file_size: 0,
            start_time: 0.,
            end_time: 0.,
            json_printer: Arc::clone(&param.json_printer),
        }
    }

    pub fn data_bytes_encoded(&self) -> u64 {
        let data_size = ver_to_data_size(self.version);

        self.data_blocks_written as u64 * data_size as u64 - self.data_padding_bytes as u64
    }
}

impl ProgressReport for Stats {
    fn start_time_mut(&mut self) -> &mut f64 {
        &mut self.start_time
    }

    fn end_time_mut(&mut self) -> &mut f64 {
        &mut self.end_time
    }

    fn units_so_far(&self) -> u64 {
        self.data_blocks_written * self.chunk_size
    }

    fn total_units(&self) -> Option<u64> {
        match self.total_data_blocks {
            None => None,
            Some(x) => Some(x * self.chunk_size),
        }
    }
}

struct Lot {
    version: Version,
    block: Block,
    data_par_burst: Option<(usize, usize, usize)>,
    meta_enabled: bool,
    block_size: usize,
    lot_size: usize,
    slots_used: usize,
    data: Vec<u8>,
    write_pos_s: Vec<u64>,
}

struct BlockBuffer {
    lots: Vec<Lot>
}

enum GetSlotResult<'a> {
    Full,
    Vacant(&'a mut [u8]),
}

impl Lot {
    pub fn new(
        version: Version,
        uid: &[u8; SBX_FILE_UID_LEN],
        data_par_burst: Option<(usize, usize, usize)>,
        meta_enabled: bool,
        lot_size: usize,
    ) -> Self {
        let block_size = ver_to_block_size(version);

        Lot {
            version,
            block: Block::new(version, uid, BlockType::Data),
            data_par_burst,
            meta_enabled,
            block_size,
            lot_size,
            slots_used: 0,
            data: vec![0; block_size * lot_size],
            write_pos_s: vec![0; lot_size],
        }
    }

    pub fn lot_size(&self) -> usize {
        self.lot_size
    }

    pub fn get_slot(&mut self) -> GetSlotResult {
        if self.slots_used < self.lot_size {
            let start = self.slots_used * self.block_size;
            let end_exc = start + self.block_size;
            self.slots_used += 1;
            GetSlotResult::Vacant(&mut self.data[start..end_exc])
        } else {
            GetSlotResult::Full
        }
    }

    fn fill_in_padding(&mut self) {
        if let Some((ref data, _, _)) = self.data_par_burst {
            for i in self.slots_used..self.lot_size {
                let start = i * self.block_size;
                let end_exc = start + self.block_size;
                let slot = &mut self.data[start..end_exc];

                sbx_block::write_padding(self.version, 0, slot);
            }
        }
    }

    pub fn encode(&mut self, lot_start_seq_num: usize) {
        self.block.set_seq_num(lot_start_seq_num as u32);

        for (slot_index, slot) in self.data.chunks_mut(self.block_size).enumerate() {
            match self.data_par_burst {
                None => block.sync_to_buffer(None, slot),
                Some((data, _, _)) =>
                    if slot_index < data {
                        self.block.sync_to_buffer(None, slot)
                    }
            }

            let write_pos = calc_data_block_write_pos(
                self.version,
                self.block.get_seq_num(),
                Some(self.meta_enabled),
                self.data_par_burst,
            );

            self.write_pos_s[slot_index] = write_pos;

            block.add1_seq_num().unwrap();
        }
    }
}

fn pack_metadata(
    block: &mut Block,
    param: &Param,
    stats: &Stats,
    file_metadata: &Option<fs::Metadata>,
    file_size: Option<u64>,
    hash: Option<multihash::HashBytes>,
) {
    block.set_seq_num(0);

    let metas = block.metas_mut().unwrap();

    {
        // add file name
        match param.in_file {
            None => {}
            Some(ref f) => {
                let file_name = file_utils::get_file_name_part_of_path(f);
                metas.push(Metadata::FNM(file_name));
            }
        }
    }
    {
        // add SBX file name
        let file_name = file_utils::get_file_name_part_of_path(&param.out_file);
        metas.push(Metadata::SNM(file_name));
    }
    {
        // add file size
        match file_size {
            Some(f) => metas.push(Metadata::FSZ(f)),
            None => {}
        }
    }
    {
        // add file last modifcation time
        match file_metadata {
            &Some(ref m) => match m.modified() {
                Ok(t) => match t.duration_since(UNIX_EPOCH) {
                    Ok(t) => metas.push(Metadata::FDT(t.as_secs() as i64)),
                    Err(_) => {}
                },
                Err(_) => {}
            },
            &None => {}
        }
    }
    {
        // add SBX encoding time
        metas.push(Metadata::SDT(stats.start_time as i64));
    }
    {
        // add hash
        let hsh = match hash {
            Some(hsh) => hsh,
            None => {
                let ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();
                ctx.finish_into_hash_bytes()
            }
        };
        metas.push(Metadata::HSH(hsh));
    }
    {
        // add RS params
        if param.rs_enabled {
            metas.push(Metadata::RSD(param.data_par_burst.unwrap().0 as u8));
            metas.push(Metadata::RSP(param.data_par_burst.unwrap().1 as u8));
        }
    }
}

fn write_meta_blocks(
    param: &Param,
    stats: &Arc<Mutex<Stats>>,
    file_metadata: &Option<fs::Metadata>,
    file_size: Option<u64>,
    hash: Option<multihash::HashBytes>,
    block: &mut Block,
    writer: &mut FileWriter,
    record_stats: bool,
) -> Result<(), Error> {
    let mut buffer: [u8; SBX_LARGEST_BLOCK_SIZE] = [0; SBX_LARGEST_BLOCK_SIZE];

    // pack metadata into the block
    pack_metadata(
        block,
        param,
        &stats.lock().unwrap(),
        file_metadata,
        file_size,
        hash,
    );

    match block.sync_to_buffer(None, &mut buffer) {
        Ok(()) => {}
        Err(sbx_block::Error::TooMuchMetadata(ref m)) => {
            return Err(Error::with_msg(&make_too_much_meta_err_string(
                block.get_version(),
                m,
            )));
        }
        Err(_) => unreachable!(),
    }

    let write_pos_s =
        sbx_block::calc_meta_block_all_write_pos_s(param.version, param.data_par_burst);

    for &p in write_pos_s.iter() {
        writer.seek(SeekFrom::Start(p))?;

        writer.write(sbx_block::slice_buf(block.get_version(), &buffer))?;

        if record_stats {
            stats.lock().unwrap().meta_blocks_written += 1;
        }
    }

    block.add1_seq_num().unwrap();

    Ok(())
}

fn write_data_block(
    param: &Param,
    block: &mut Block,
    buffer: &mut [u8],
    writer: &mut FileWriter,
) -> Result<(), Error> {
    let write_pos = calc_data_block_write_pos(
        param.version,
        block.get_seq_num(),
        Some(param.meta_enabled),
        param.data_par_burst,
    );

    block_sync_and_write(block, buffer, writer, write_pos)
}

fn block_sync_and_write(
    block: &mut Block,
    buffer: &mut [u8],
    writer: &mut FileWriter,
    pos: u64,
) -> Result<(), Error> {
    match block.sync_to_buffer(None, buffer) {
        Ok(()) => {}
        Err(sbx_block::Error::TooMuchMetadata(ref m)) => {
            return Err(Error::with_msg(&make_too_much_meta_err_string(
                block.get_version(),
                m,
            )));
        }
        Err(_) => unreachable!(),
    }

    writer.seek(SeekFrom::Start(pos))?;

    writer.write(sbx_block::slice_buf(block.get_version(), buffer))?;

    match block.add1_seq_num() {
        Ok(_)  => Ok(()),
        Err(_) => Err(Error::with_msg("Block seq num already at max, addition causes overflow. This might be due to file size being changed during the encoding, or too much data from stdin"))
    }
}

pub fn encode_file(param: &Param) -> Result<Stats, Error> {
    let ctrlc_stop_flag = setup_ctrlc_handler(param.json_printer.json_enabled());

    // setup file reader and writer
    let mut reader = match param.in_file {
        Some(ref f) => Reader::new(ReaderType::File(FileReader::new(
            f,
            FileReaderParam {
                write: false,
                buffered: true,
            },
        )?)),
        None => Reader::new(ReaderType::Stdin(std::io::stdin())),
    };

    let mut writer = FileWriter::new(
        &param.out_file,
        FileWriterParam {
            read: false,
            append: false,
            truncate: true,
            buffered: true,
        },
    )?;

    let metadata = match reader.metadata() {
        Some(m) => Some(m?),
        None => None,
    };

    let file_size = match reader.get_file_size() {
        Some(s) => Some(s?),
        None => None,
    };

    // calulate length to read and position to seek to
    let (required_len, seek_to) = match file_size {
        Some(file_size) => {
            let RequiredLenAndSeekTo {
                required_len,
                seek_to,
            } = misc_utils::calc_required_len_and_seek_to_from_byte_range(
                param.from_pos,
                param.to_pos,
                true,
                0,
                PositionOrLength::Len(file_size),
                None,
            );
            (Some(required_len), Some(seek_to))
        }
        None => (None, None),
    };

    {
        // check if required length exceeds maximum
        match required_len {
            None => {}
            Some(required_len) => {
                let max_in_file_size = ver_to_max_data_file_size(param.version);

                if required_len > max_in_file_size {
                    return Err(Error::with_msg(&format!("Encoding range specified for \"{}\" exceeds the maximum supported file size, size to be encoded : {}, max : {}",
                                                            param.in_file.as_ref().unwrap(),
                                                            required_len,
                                                            max_in_file_size)));
                }
            }
        }
    }

    // setup stats
    let stats = Arc::new(Mutex::new(Stats::new(param, required_len)));

    // setup reporter
    let reporter = ProgressReporter::new(
        &stats,
        "Data encoding progress",
        "bytes",
        param.pr_verbosity_level,
        param.json_printer.json_enabled(),
    );

    // set up hash state
    let mut hash_ctx = multihash::hash::Ctx::new(param.hash_type).unwrap();

    // setup Reed-Solomon things
    let mut rs_codec = match param.data_par_burst {
        None => None,
        Some((data, parity, _)) => Some(ReedSolomon::new(data, parity).unwrap()),
    };

    let single_lot_size = match param.data_par_burst {
        None => DEFAULT_SINGLE_LOT_SIZE,
        Some((data, parity, _)) => data + parity,
    };

    let lot_count = num_cpus::get();

    let block_size = ver_to_block_size(param.version);

    // setup main data buffer
    let mut buffer: Vec<u8> = vec![0; block_size * single_lot_size * lot_count];

    let mut write_pos_s: Vec<u64> = vec![0; single_lot_size * lot_count];

    let data_slots_per_lot = match param.data_par_burst {
        None => single_lot_size,
        Some((data, _, _)) => data,
    };

    let total_data_slot_count = data_slots_per_lot * lot_count;

    // setup padding block
    let padding: [u8; SBX_LARGEST_BLOCK_SIZE] = [0x1A; SBX_LARGEST_BLOCK_SIZE];

    // setup main data block
    let mut block = Block::new(param.version, &param.uid, BlockType::Data);

    // seek to calculated position
    if let Some(seek_to) = seek_to {
        if let Some(r) = reader.seek(SeekFrom::Start(seek_to)) {
            r?;
        }
    }

    reporter.start();

    if param.meta_enabled {
        // write dummy metadata block
        write_meta_blocks(
            param,
            &stats,
            &metadata,
            required_len,
            None,
            &mut block,
            &mut writer,
            true,
        )?;
    }

    let mut bytes_processed: u64 = 0;

    let mut batch_start_seq_num = Some(1);

    let mut last_batch = false;

    while !last_batch {
        let mut stats = stats.lock().unwrap();

        // full up data buffer
        let mut data_slots_used = 0;
        loop {
            break_if_atomic_bool!(ctrlc_stop_flag);

            if let Some(required_len) = required_len {
                last_batch = true;
                break_if_reached_required_len!(bytes_processed, required_len);
            }

            // read data in
            let start = match param.data_par_burst {
                None => data_slots_used * block_size,
                Some((data, _, _)) => {
                    let lot_to_use = data_slots_used / data;
                    let start_index_in_lot = data_slots_used % data;
                    let slot_to_use = lot_to_use * single_lot_size + start_index_in_lot;

                    slot_to_use * block_size
                },
            };

            let end_exc = start + block_size;
            let slot = &mut buffer[start..end_exc];
            let read_res = reader.read(sbx_block::slice_data_buf_mut(param.version, slot))?;

            bytes_processed += read_res.len_read as u64;

            if read_res.len_read == 0 {
                last_batch = true;
                break;
            }

            stats.data_padding_bytes +=
                sbx_block::write_padding(param.version, read_res.len_read, slot);

            data_slots_used += 1;

            if data_slots_used == total_data_slot_count {
                break;
            }
        }

        break_if_atomic_bool!(ctrlc_stop_flag);

        // do SBX block encoding and record write positions
        buffer.par_chunks_mut(block_size * single_lot_size)
            .enumerate()
            .for_each(|(lot_index, lot)| {
                let mut block = Block::new(param.version, &param.uid, BlockType::Data);
                let batch_start_seq_num = batch_start_seq_num.unwrap();
                let lot_start_seq_num = batch_start_seq_num + lot_index * single_lot_size;

                block.set_seq_num(lot_start_seq_num as u32);

                for (slot_index, slot) in lot.chunks_mut(block_size).enumerate() {
                    match param.data_par_burst {
                        None => block.sync_to_buffer(None, slot),
                        Some((data, _, _)) =>
                            if slot_index < data {
                                block.sync_to_buffer(None, slot)
                            }
                    }

                    if let Some((data, _, _)) = param.data_par_burst {
                        if slot_index >= data {
                            break;
                        }
                    }

                    block.add1_seq_num().unwrap();
                }
            });

        // do Reed-Solomon encoding if needed
        if let Some(ref mut rs_codec) = rs_codec {
            buffer.par_chunks_mut(block_size * single_lot_size)
                .for_each(|lot| {
                    let mut refs: SmallVec<[&mut [u8]; 32]> = SmallVec::with_capacity(single_lot_size);

                    // collect references to data segments
                    for slot in lot.chunks_mut(block_size) {
                        refs.push(slot);
                    }

                    rs_codec.encode(&mut refs);
                });
        }

        // output used slots of buffer
        match param.data_par_bust {
            None => {
                // use data_slots_used as main reference
                for i in 0..data_slots_used {
                    write_data_block(param, )
                }
            }
        }

        let lots_used = data_slots_used / data_slots_per_lot;

        // batch_start_seq_num is only used for the next loop
        // but if it's the last batch, then it's meaningless to calculate it
        if last_batch {
            batch_start_seq_num = None;
        } else {
            batch_start_seq_num = Some(batch_start_seq_num.unwrap() + lots_used * single_lot_size);
        }
    }

    // loop {
    //     let mut stats = stats.lock().unwrap();

    //     break_if_atomic_bool!(ctrlc_stop_flag);

    //     if let Some(required_len) = required_len {
    //         break_if_reached_required_len!(bytes_processed, required_len);
    //     }

    //     // read data in
    //     let read_res = reader.read(sbx_block::slice_data_buf_mut(param.version, &mut data))?;

    //     bytes_processed += read_res.len_read as u64;

    //     if read_res.len_read == 0 {
    //         break;
    //     }

    //     let mut data_blocks_written = 0;
    //     let mut parity_blocks_written = 0;

    //     stats.data_padding_bytes +=
    //         sbx_block::write_padding(param.version, read_res.len_read, &mut data);

    //     // start encoding
    //     write_data_block(param, &mut block, &mut data, &mut writer)?;

    //     data_blocks_written += 1;

    //     // update hash state if needed
    //     if param.meta_enabled {
    //         let data_part = &sbx_block::slice_data_buf(param.version, &data)[0..read_res.len_read];
    //         hash_ctx.update(data_part);
    //     }

    //     // update Reed-Solomon data if needed
    //     if let Some(ref mut rs_codec) = rs_codec {
    //         // encode normally once
    //         if let Some(parity_to_use) = rs_codec.encode_no_block_sync(&data) {
    //             for p in parity_to_use.iter_mut() {
    //                 write_data_block(param, &mut block, p, &mut writer)?;

    //                 parity_blocks_written += 1;
    //             }
    //         }
    //     }

    //     // update stats
    //     stats.data_blocks_written += data_blocks_written;
    //     stats.parity_blocks_written += parity_blocks_written;
    // }

    // if let Some(ref mut rs_codec) = rs_codec {
    //     // fill remaining slots with padding if required
    //     if rs_codec.active() {
    //         let mut stats = stats.lock().unwrap();

    //         let slots_to_fill = rs_codec.unfilled_slot_count();
    //         for i in 0..slots_to_fill {
    //             // write padding
    //             write_data_block(param, &mut block, &mut padding, &mut writer)?;

    //             stats.data_blocks_written += 1;
    //             stats.data_padding_bytes += ver_to_data_size(param.version);

    //             if let Some(parity_to_use) = rs_codec.encode_no_block_sync(&padding) {
    //                 // this should only be executed at the last iteration
    //                 assert_eq!(i, slots_to_fill - 1);

    //                 for p in parity_to_use.iter_mut() {
    //                     write_data_block(param, &mut block, p, &mut writer)?;

    //                     stats.parity_blocks_written += 1;
    //                 }
    //             }
    //         }
    //     }
    // }

    let data_bytes_encoded = match required_len {
        Some(x) => x,
        None => stats.lock().unwrap().data_bytes_encoded(),
    };

    if param.meta_enabled {
        let hash_bytes = hash_ctx.finish_into_hash_bytes();

        // write actual medata blocks
        write_meta_blocks(
            param,
            &stats,
            &metadata,
            Some(data_bytes_encoded),
            Some(hash_bytes.clone()),
            &mut block,
            &mut writer,
            false,
        )?;

        // record hash in stats
        stats.lock().unwrap().hash_bytes = Some(hash_bytes.clone());
    }

    reporter.stop();

    stats.lock().unwrap().in_file_size = data_bytes_encoded;
    stats.lock().unwrap().out_file_size = file_utils::from_orig_file_size::calc_container_size(
        param.version,
        Some(param.meta_enabled),
        param.data_par_burst,
        data_bytes_encoded,
    );

    let stats = stats.lock().unwrap().clone();

    Ok(stats)
}
