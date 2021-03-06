use crate::cli_utils::*;
use crate::decode_core;
use crate::decode_core::Param;
use crate::file_utils;
use crate::json_printer::BracketType;
use crate::output_channel::OutputChannel;
use clap::*;
use std::sync::Arc;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("decode")
        .about("Decode SBX container")
        .arg(in_file_arg().help("SBX container to decode"))
        .arg(out_arg().help(
            "Decoded file name. Supply - to use stdout as output. Use ./- for files named -.
If output is stdout, progress text and final stats are outputted to stderr instead.
If OUT is not provided, then the original file name stored in the SBX container
(STOREDNAME) is used if present (only the file part of STOREDNAME is used). If
OUT is provided and is a directory, then the output file is stored as OUT/STOREDNAME
if STOREDNAME is present (only the file part of STOREDNAME is used). If OUT is
provided and is not a directory, then it is used directly.",
        ))
        .arg(force_arg().help("Force overwrite even if OUT exists"))
        .arg(multi_pass_arg().help(
            "Disable truncation of OUT, and skip writing if a non-blank data
chunk already exists at the location. This allows writing to OUT
multiple times to update it gradually. Ignored if output is stdout.",
        ))
        .arg(multi_pass_no_skip_arg().help(
            "Disable truncation of OUT, write even if a non-blank data chunk
exists at the location. This allows writing to OUT multiple times
to update it gradually. Ignored if output is stdout.",
        ))
        .arg(no_meta_arg())
        .arg(pr_verbosity_level_arg())
        .arg(ref_from_byte_arg())
        .arg(ref_to_byte_inc_arg())
        .arg(ref_to_byte_exc_arg())
        .arg(guess_burst_from_byte_arg())
        .arg(from_byte_arg().help(FROM_BYTE_ARG_HELP_MSG_REF_BLOCK))
        .arg(to_byte_inc_arg())
        .arg(to_byte_exc_arg())
        .arg(force_misalign_arg())
        .arg(burst_arg().help(
            "Burst error resistance level used by the container.
Use this if the level used by the container is above 1000,
as blkar will only guess up to 1000. Or use this when blkar
fails to guess correctly. blkar uses this value only if
output is stdout and container version is RS enabled.",
        ))
        .arg(verbose_arg().help("Show reference block info"))
        .arg(json_arg())
}

pub fn decode<'a>(matches: &ArgMatches<'a>) -> i32 {
    let mut json_printer = get_json_printer!(matches);

    let out = matches.value_of("out");

    // update json_printer output channel if stdout is going to be used by file output
    if let Some(ref f) = out {
        if file_utils::check_if_file_is_stdout(f) {
            let output_channel = OutputChannel::Stderr;

            if !json_printer.json_enabled() {
                print_block!(output_channel =>
                             "Warning :";
                             "";
                             "    Since output is stdout, blkar can only output data chunks in the";
                             "    anticipated encoding order.";
                             "";
                             "        For version with no FEC enabled (version 1, 2, 3), this means blkar";
                             "        reads in the sequential pattern with optional metadata block and";
                             "        outputs the data chunks.";
                             "";
                             "        For version with FEC enabled (version 17, 18, 19), this means blkar";
                             "        first guesses the burst resistance level, then reads using the block";
                             "        set interleaving pattern and outputs the data chunks.";
                             "";
                             "    blkar also tries to strip the data padding at the end of the container";
                             "    at a best effort basis, but does not provide any guarantees.";
                             "";
                             "    If the ordering matches the anticipated ordering, output of blkar to";
                             "    stdout should match the one produced in output to file mode. If the";
                             "    ordering is not as anticipated, you may fix it by sorting the SBX";
                             "    container using the blkar sort command first.";
                             "";
                );
            }

            Arc::get_mut(&mut json_printer)
                .unwrap()
                .set_output_channel(output_channel);
        }
    }

    json_printer.print_open_bracket(None, BracketType::Curly);

    let pr_verbosity_level = get_pr_verbosity_level!(matches, json_printer);

    let burst = get_burst_opt!(matches, json_printer);

    let in_file = get_in_file!(matches, json_printer);

    let from_pos = get_from_pos!(matches, json_printer);
    let to_pos = get_to_pos!(matches, json_printer);

    let ref_from_pos = get_ref_from_pos!(matches, json_printer);
    let ref_to_pos = get_ref_to_pos!(matches, json_printer);

    let guess_burst_from_pos = get_guess_burst_from_pos!(matches, json_printer);

    let param = Param::new(
        get_ref_block_choice!(matches),
        ref_from_pos,
        ref_to_pos,
        guess_burst_from_pos,
        matches.is_present("force"),
        get_multi_pass!(matches, json_printer),
        &json_printer,
        from_pos,
        to_pos,
        matches.is_present("force_misalign"),
        in_file,
        out,
        matches.is_present("verbose"),
        pr_verbosity_level,
        burst,
    );
    match decode_core::decode_file(&param) {
        Ok(Some(s)) => exit_with_msg!(ok json_printer => "{}", s),
        Ok(None) => exit_with_msg!(ok json_printer => ""),
        Err(e) => exit_with_msg!(op json_printer => "{}", e),
    }
}
