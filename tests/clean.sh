#!/bin/bash

if [[ $PWD != */tests ]]; then
  cd tests
fi

tests=(
  "version_tests"
  "version_tests_encode_stdin"
  "version_tests_decode_stdout"
  "compare_encode_file_and_stdin"
  "compare_decode_file_and_stdout"
  "compare_decode_file_and_stdout_corrupted_container"
  "decode_manual_burst"
  "decode_manual_burst_encode_stdin"
  "decode_manual_burst_decode_stdout"
  "repair_manual_burst"
  "repair_manual_burst_encode_stdin"
  "repair_manual_burst_decode_stdout"
  "nometa_tests"
  "nometa_tests_encode_stdin"
  "nometa_tests_decode_stdout"
  "compare_decode_file_and_stdout_nometa"
  "hash_tests"
  "hash_tests_encode_stdin"
  "hash_tests_decode_stdout"
  "rescue_tests"
  "rescue_tests_encode_stdin"
  "rescue_tests_decode_stdout"
  "rescue_pick_uid_tests"
  "rescue_pick_uid_tests_encode_stdin"
  "rescue_pick_uid_tests_decode_stdout"
  "show_pick_uid_tests"
  "out_file_logic_tests"
  "corruption_tests"
  "corruption_tests_encode_stdin"
  "corruption_tests_decode_stdout"
  "burst_corruption_tests"
  "burst_corruption_tests_encode_stdin"
  "burst_corruption_tests_decode_stdout"
  "sort_tests"
  "sort_tests_encode_stdin"
  "sort_tests_decode_stdout"
  "sort_stats_tests"
  "sort_dry_run"
  "sort_multi_pass"
  "sort_multi_pass_no_skip"
  "decode_multi_pass"
  "decode_multi_pass_no_skip"
  "file_size_calc_tests"
  "repair_truncated_tests"
  "repair_truncated_tests_encode_stdin"
  "repair_truncated_tests_decode_stdout"
  "decode_blanks"
  "decode_blanks_decode_stdout"
  "check_from_to_tests"
  "check_from_to_rounding"
  "check_from_to_force_misalign"
  "decode_from_to_tests"
  "decode_from_to_tests_rounding"
  "decode_from_to_tests_force_misalign"
  "decode_from_to_tests_corruption_based"
  "decode_from_to_tests_corruption_based_rounding"
  "decode_from_to_tests_corruption_based_force_misalign"
  "decode_from_to_tests_decode_stdout"
  "decode_from_to_tests_decode_stdout_rounding"
  "decode_from_to_tests_decode_stdout_force_misalign"
  "show_from_to_tests"
  "show_from_to_tests_rounding"
  "show_from_to_tests_force_misalign"
  "rescue_from_to_tests"
  "rescue_from_to_tests_encode_stdin"
  "rescue_from_to_tests_rounding"
  "rescue_from_to_tests_force_misalign"
  "sort_from_to_tests"
  "sort_from_to_tests_rounding"
  "sort_from_to_tests_force_misalign"
  "check_ref_from_to_tests"
  "check_ref_from_to_tests_rounding"
  "check_ref_from_to_tests_force_misalign"
  "decode_ref_from_to_tests"
  "decode_ref_from_to_tests_rounding"
  "decode_ref_from_to_tests_force_misalign"
  "sort_ref_from_to_tests"
  "sort_ref_from_to_tests_rounding"
  "sort_ref_from_to_tests_force_misalign"
  "show_guess_burst_force_misalign"
  "decode_guess_burst_force_misalign"
  "sort_guess_burst_force_misalign"
)

rm -f dummy*

rm -f  *.sbx

rm -f rescued_data/*

rm -f rescued_data2/*

rm -f rescue_log

rm -f filler*

rm -f out_test/*

rm -f sort_*.sbx.*

rm -f exit_code

rm -f ../blkar

rm -f data_chunk

rm -f data_chunk_orig

rm -f chunk_*

rm -f decode*.sbx.*

for t in ${tests[@]}; do
  if [[ "$t" != "" ]]; then
    rm -rf $t
  fi
done
