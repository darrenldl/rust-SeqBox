use sbx_specs::{Version,
                ver_to_usize,
                ver_to_block_size,
                ver_to_data_size,
                ver_uses_rs};

use std::str::FromStr;

use file_utils;

use clap::*;
use cli_utils::*;
pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("calc")
        .about("Compute and display detailed information given an encoding configuration")
        .arg(Arg::with_name("in_file_size")
             .value_name("INFILE-SIZE")
             .required(true)
             .index(1)
             .help("Input file size"))
        .arg(sbx_version_arg())
        .arg(Arg::with_name("no_meta")
             .long("no-meta")
             .help("Skip metadata block in the calculations. Metadata block is
never skipped for version 17, 18, 19.
This means this option has no effect for version 17, 18, 19."))
        .arg(rs_data_arg())
        .arg(rs_parity_arg())
        .arg(burst_arg())
}

pub fn calc<'a>(matches : &ArgMatches<'a>) -> i32 {
    let version   = get_version!(matches);

    let in_file_size =
        match u64::from_str(matches.value_of("in_file_size").unwrap()) {
            Ok(x)  => x,
            Err(_) => exit_with_msg!(usr => "Invalid file size")
        };

    let data_par_burst =
        if ver_uses_rs(version) {
            // deal with RS related options
            let data_shards   = get_data_shards!(matches, version);
            let parity_shards = get_parity_shards!(matches, version);

            check_data_parity_shards!(data_shards, parity_shards);

            let burst = get_burst_or_zero!(matches);

            Some((data_shards, parity_shards, burst))
        } else {
            None
        };

    let out_file_size =
        file_utils::from_orig_file_size::calc_container_size(version,
                                                             data_par_burst,
                                                             in_file_size);

    let total_block_count =
        file_utils::from_orig_file_size::calc_total_block_count_exc_burst_gaps(version,
                                                                               data_par_burst,
                                                                               in_file_size);

    let (data_only_block_count, parity_block_count) =
        file_utils::from_orig_file_size::calc_data_only_and_parity_block_count_exc_burst_gaps(version,
                                                                                              data_par_burst,
                                                                                              in_file_size);

    println!(    "SBX container general info");
    println!(    "========================================");
    if ver_uses_rs(version) {
        println!("    SBX container version        : {} (0x{:X})",
                 ver_to_usize(version),
                 ver_to_usize(version));
    } else {
        println!("    SBX container version        : {}", ver_to_usize(version));
    }
    println!(    "    SBX container block size     : {}", ver_to_block_size(version));
    println!(    "    SBX container data  size     : {}", ver_to_data_size(version));

    println!();

    println!(    "SBX block distribution");
    println!(    "========================================");
    if ver_uses_rs(version) {
        println!("    Metadata    block count      : {}", 1 + data_par_burst.unwrap().1);
        println!("    Data only   block count      : {}", data_only_block_count);
        println!("    Data parity block count      : {}", parity_block_count);
        println!("    Total       block count      : {}", total_block_count);
    } else {
        println!("    Metadata block count         : {}", 1);
        println!("    Data     block count         : {}", data_only_block_count);
        println!("    Total    block count         : {}", total_block_count);
    }

    println!();

    println!(    "Error correction info");
    println!(    "========================================");
    if ver_uses_rs(version) {
        println!("    RS data   shard count        : {}", data_par_burst.unwrap().0);
        println!("    RS parity shard count        : {}", data_par_burst.unwrap().1);
        println!("    Burst error resistance level : {}", data_par_burst.unwrap().2);
    } else {
        println!("    RS data   shard count        : {}", "version does not use RS");
        println!("    RS parity shard count        : {}", "version does not use RS");
        println!("    Burst error resistance level : {}", "version does not support burst error resistance");
    }

    println!();

    if ver_uses_rs(version) {
        let (data, par, burst) = data_par_burst.unwrap();

        let block_size = ver_to_block_size(version);

        println!("Error correction parameters interpretation");
        println!("========================================");
        if burst == 0 {
            println!("    The container can tolerate corruption of {} SBX blocks(totalling {} bytes) in
    every set of {} consecutive blocks({} bytes).",
                     par,
                     par * block_size,
                     data + par,
                     (data + par) * block_size);
        } else {
            println!("    The container can tolerate {} burst SBX block corruptions in
    every set of {} consecutive blocks({} bytes).

    Each burst error may be up to {} blocks({} bytes) in size.

    In total, {} bytes(aligned at block boundary) may be corrupted in
    any set of {} consecutive blocks({} bytes).",
                     par,
                     (data + par) * burst,
                     (data + par) * burst * block_size,
                     burst,
                     burst * block_size,
                     par * burst * block_size,
                     (data + par) * burst,
                     (data + par) * burst * block_size);
            println!();
            println!("    Note that the actual tolerance depends on the behaviour of the file system.");
        }

        println!();
    }

    println!(    "File and container size");
    println!(    "========================================");
    println!(    "    File size                    : {}", in_file_size);
    println!(    "    SBX container size           : {}", out_file_size);

    exit_with_msg!(ok => "")
}
