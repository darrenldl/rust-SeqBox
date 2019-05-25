#!/bin/bash

tests=(
    "version_tests"
    "version_tests_encode_stdin"
    "version_tests_decode_stdout"
    "nometa_tests"
    "hash_tests"
    "rescue_tests"
    "rescue_from_to_tests"
    "rescue_pick_uid_tests"
    "show_pick_uid_tests"
    "out_file_logic_tests"
    "corruption_tests"
    "burst_corruption_tests"
    "sort_tests"
    "file_size_calc_tests"
    "repair_truncated_tests"
    "update_fnm_tests"
    "update_no_fnm_tests"
    "update_snm_tests"
    "update_no_snm_tests"
    "check_hash_tests"
)
