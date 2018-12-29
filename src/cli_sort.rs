use sort_core::Param;

use sort_core;
use file_utils;
use misc_utils;

use clap::*;
use cli_utils::*;

use json_printer::BracketType;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("sort")
        .about("Sort SBX blocks in container, can also readjust burst error resistance level")
        .arg(in_file_arg()
             .help("SBX container to sort"))
        .arg(out_arg()
             .help("Sorted SBX container (defaults to INFILE.sorted). If OUT is a directory, then the
container is stored as OUT/INFILE.sorted (only the file part of INFILE is used).
Ignored if --dry-run is supplied."))
        .arg(force_arg()
             .help("Force overwrite even if OUTFILE exists"))
        .arg(multi_pass_arg()
             .help("Disable truncation of OUT. This allows writing to OUT multiple
times to update it gradually."))
        .arg(pr_verbosity_level_arg())
        .arg(dry_run_arg()
             .help("Only do sorting in memory, does not output the sorted container."))
        .arg(ref_from_byte_arg())
        .arg(ref_to_byte_arg())
        .arg(from_byte_arg())
        .arg(to_byte_arg())
        .arg(burst_arg()
             .help("Burst error resistance level to use for the output container.
Defaults to guessing the level (guesses up to 1000) used by the
original container and uses the result."))
        .arg(verbose_arg()
             .help("Show reference block info"))
        .arg(json_arg())
}

pub fn sort<'a>(matches : &ArgMatches<'a>) -> i32 {
    let json_printer = get_json_printer!(matches);

    json_printer.print_open_bracket(None, BracketType::Curly);

    let in_file = get_in_file!(matches, json_printer);
    let out = match matches.value_of("out") {
        None    => {
            format!("{}.sorted", in_file)
        },
        Some(x) => {
            if file_utils::check_if_file_is_dir(x) {
                let in_file = file_utils::get_file_name_part_of_path(in_file);
                misc_utils::make_path(&[x,
                                        &format!("{}.sorted", in_file)])
            } else {
                String::from(x)
            }
        },
    };

    let force      = matches.is_present("force");
    let multi_pass = matches.is_present("multi_pass");
    let dry_run    = matches.is_present("dry_run");

    let burst = get_burst_opt!(matches, json_printer);

    exit_if_file!(exists &out
                  => force || multi_pass || dry_run
                  => json_printer
                  => "File \"{}\" already exists", out);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let out : Option<&str> =
        if dry_run {
            None
        } else {
            Some(&out)
        };

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos   = get_to_pos!(matches, json_printer);

    let ref_from_pos = get_ref_from_pos!(matches, json_printer);
    let ref_to_pos   = get_ref_to_pos!(matches, json_printer);

    let param = Param::new(get_ref_block_choice!(matches),
                           ref_from_pos,
                           ref_to_pos,
                           multi_pass,
                           &json_printer,
                           from_pos,
                           to_pos,
                           in_file,
                           out,
                           matches.is_present("verbose"),
                           pr_verbosity_level,
                           burst);
    match sort_core::sort_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok json_printer => "{}", s),
        Ok(None)    => exit_with_msg!(ok json_printer => ""),
        Err(e)      => exit_with_msg!(op json_printer => "{}", e),
    }
}
