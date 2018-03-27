#![cfg(test)]

use super::*;
use sbx_specs;

#[test]
fn test_calc_rs_enabled_meta_write_indices() {
    {
        const PARITY : usize = 2;
        const BURST  : usize = 4;

        let data_par_burst = Some((0, PARITY, BURST));

        let indices = calc_meta_block_dup_write_indices(data_par_burst);

        assert_eq!(2, indices.len());

        assert_eq!( 5, indices[0]);
        assert_eq!(10, indices[1]);
    }
    {
        const PARITY : usize = 1;
        const BURST  : usize = 2;

        let data_par_burst = Some((0, PARITY, BURST));

        let indices = calc_meta_block_dup_write_indices(data_par_burst);

        assert_eq!(1, indices.len());

        assert_eq!(3, indices[0]);
    }
    {
        const PARITY : usize = 2;
        const BURST  : usize = 11;

        let data_par_burst = Some((0, PARITY, BURST));

        let indices = calc_meta_block_dup_write_indices(data_par_burst);

        assert_eq!(2, indices.len());

        assert_eq!(12, indices[0]);
        assert_eq!(24, indices[1]);
    }
}

#[test]
fn test_calc_rs_enabled_data_write_index_simple_cases() {
    {
        const DATA   : usize = 3;
        const PARITY : usize = 2;
        const TOTAL  : usize = DATA + PARITY;
        const BURST  : usize = 4;

        let data_par_burst = Some((DATA, PARITY, BURST));

        let table : [u32; 2 * (TOTAL * BURST) + (1 + PARITY)] =
            [00, 01, 06, 11, 16,
             00, 02, 07, 12, 17,
             00, 03, 08, 13, 18,
             04, 09, 14, 19,
             05, 10, 15, 20,

             21, 26, 31, 36,
             22, 27, 32, 37,
             23, 28, 33, 38,
             24, 29, 34, 39,
             25, 30, 35, 40];

        // go through data seq num
        for seq in 1..40 {
            let write_index =
                calc_data_block_write_index(seq,
                                            None,
                                            data_par_burst) as usize;

            assert_eq!(table[write_index], seq);
        }

        // go through the table
        for index in 0..table.len() {
            let seq = table[index];

            if seq > 0 {
                let write_index =
                    calc_data_block_write_index(seq,
                                                None,
                                                data_par_burst) as usize;

                assert_eq!(index, write_index);
            }
        }
    }
    {
        const DATA   : usize = 1;
        const PARITY : usize = 1;
        const TOTAL  : usize = DATA + PARITY;
        const BURST  : usize = 2;

        let data_par_burst = Some((DATA, PARITY, BURST));

        let table : [u32; 5 * (TOTAL * BURST) + (1 + PARITY)] =
            [00, 01, 03,
             00, 02, 04,

             05, 07,
             06, 08,

             09, 11,
             10, 12,

             13, 15,
             14, 16,

             17, 19,
             18, 20];

        // go through data seq num
        for seq in 1..20 {
            let write_index =
                calc_data_block_write_index(seq,
                                            None,
                                            data_par_burst) as usize;

            assert_eq!(table[write_index], seq);
        }

        // go through the table
        for index in 0..table.len() {
            let seq = table[index];

            if seq > 0 {
                let write_index =
                    calc_data_block_write_index(seq,
                                                None,
                                                data_par_burst) as usize;

                assert_eq!(index, write_index);
            }
        }
    }
    {
        const DATA   : usize = 10;
        const PARITY : usize = 2;
        const TOTAL  : usize = DATA + PARITY;
        const BURST  : usize = 11;

        let data_par_burst = Some((DATA, PARITY, BURST));

        let table : [u32; 1 * (TOTAL * BURST) + (1 + PARITY)] =
            [00, 01, 13, 25, 37, 49, 61, 73, 85,  97,  109, 121,
             00, 02, 14, 26, 38, 50, 62, 74, 86,  98,  110, 122,
             00, 03, 15, 27, 39, 51, 63, 75, 87,  99,  111, 123,
             04, 16, 28, 40, 52, 64, 76, 88, 100, 112, 124,
             05, 17, 29, 41, 53, 65, 77, 89, 101, 113, 125,
             06, 18, 30, 42, 54, 66, 78, 90, 102, 114, 126,
             07, 19, 31, 43, 55, 67, 79, 91, 103, 115, 127,
             08, 20, 32, 44, 56, 68, 80, 92, 104, 116, 128,
             09, 21, 33, 45, 57, 69, 81, 93, 105, 117, 129,
             10, 22, 34, 46, 58, 70, 82, 94, 106, 118, 130,
             11, 23, 35, 47, 59, 71, 83, 95, 107, 119, 131,
             12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132];

        // go through data seq num
        for seq in 1..132 {
            let write_index =
                calc_data_block_write_index(seq,
                                            None,
                                            data_par_burst) as usize;

            assert_eq!(table[write_index], seq);
        }

        // go through the table
        for index in 0..table.len() {
            let seq = table[index];

            if seq > 0 {
                let write_index =
                    calc_data_block_write_index(seq,
                                                None,
                                                data_par_burst) as usize;

                assert_eq!(index, write_index);
            }
        }
    }
}

#[test]
fn test_calc_rs_enabled_seq_num_at_index_simple_cases() {
    {
        const DATA   : usize = 3;
        const PARITY : usize = 2;
        const TOTAL  : usize = DATA + PARITY;
        const BURST  : usize = 4;

        let data_par_burst = Some((DATA, PARITY, BURST));

        let table : [u32; 2 * (TOTAL * BURST) + (1 + PARITY)] =
            [00, 01, 06, 11, 16,
             00, 02, 07, 12, 17,
             00, 03, 08, 13, 18,
             04, 09, 14, 19,
             05, 10, 15, 20,

             21, 26, 31, 36,
             22, 27, 32, 37,
             23, 28, 33, 38,
             24, 29, 34, 39,
             25, 30, 35, 40];

        // go through the table
        for index in 0..table.len() {
            let seq_num_from_index =
                calc_seq_num_at_index(index as u64,
                                      None,
                                      data_par_burst);

            assert_eq!(table[index], seq_num_from_index);
        }
    }
    {
        const DATA   : usize = 1;
        const PARITY : usize = 1;
        const TOTAL  : usize = DATA + PARITY;
        const BURST  : usize = 2;

        let data_par_burst = Some((DATA, PARITY, BURST));

        let table : [u32; 5 * (TOTAL * BURST) + (1 + PARITY)] =
            [00, 01, 03,
             00, 02, 04,

             05, 07,
             06, 08,

             09, 11,
             10, 12,

             13, 15,
             14, 16,

             17, 19,
             18, 20];

        // go through the table
        for index in 0..table.len() {
            let seq_num_from_index =
                calc_seq_num_at_index(index as u64,
                                      None,
                                      data_par_burst);

            assert_eq!(table[index], seq_num_from_index);
        }
    }
    {
        const DATA   : usize = 10;
        const PARITY : usize = 2;
        const TOTAL  : usize = DATA + PARITY;
        const BURST  : usize = 11;

        let data_par_burst = Some((DATA, PARITY, BURST));

        let table : [u32; 1 * (TOTAL * BURST) + (1 + PARITY)] =
            [00, 01, 13, 25, 37, 49, 61, 73, 85,  97,  109, 121,
             00, 02, 14, 26, 38, 50, 62, 74, 86,  98,  110, 122,
             00, 03, 15, 27, 39, 51, 63, 75, 87,  99,  111, 123,
             04, 16, 28, 40, 52, 64, 76, 88, 100, 112, 124,
             05, 17, 29, 41, 53, 65, 77, 89, 101, 113, 125,
             06, 18, 30, 42, 54, 66, 78, 90, 102, 114, 126,
             07, 19, 31, 43, 55, 67, 79, 91, 103, 115, 127,
             08, 20, 32, 44, 56, 68, 80, 92, 104, 116, 128,
             09, 21, 33, 45, 57, 69, 81, 93, 105, 117, 129,
             10, 22, 34, 46, 58, 70, 82, 94, 106, 118, 130,
             11, 23, 35, 47, 59, 71, 83, 95, 107, 119, 131,
             12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132];

        // go through the table
        for index in 0..table.len() {
            let seq_num_from_index =
                calc_seq_num_at_index(index as u64,
                                      None,
                                      data_par_burst);

            assert_eq!(table[index], seq_num_from_index);
        }
    }
}

quickcheck! {
    fn qc_data_seq_num_to_index_to_seq_num_meta_disabled(seq_num : u32) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };

        let index = calc_data_block_write_index(seq_num,
                                                Some(false),
                                                None);

        let seq_num_from_index = calc_seq_num_at_index(index,
                                                       Some(false),
                                                       None);

        seq_num_from_index == seq_num
    }

    fn qc_data_seq_num_to_index_to_seq_num_meta_enabled(seq_num : u32) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };

        let index = calc_data_block_write_index(seq_num,
                                                Some(true),
                                                None);

        let seq_num_from_index = calc_seq_num_at_index(index,
                                                       Some(true),
                                                       None);

        seq_num_from_index == seq_num
    }

    fn qc_data_seq_num_to_index_to_seq_num_meta_default(seq_num : u32) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };

        let index = calc_data_block_write_index(seq_num,
                                                None,
                                                None);

        let seq_num_from_index = calc_seq_num_at_index(index,
                                                       None,
                                                       None);

        seq_num_from_index == seq_num
    }

    fn qc_data_seq_num_to_index_to_seq_num_rs_enabled(seq_num       : u32,
                                                      data_shards   : usize,
                                                      parity_shards : usize,
                                                      burst         : usize) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };
        let data_shards   = 1 + data_shards % 256;
        let parity_shards = 1 + parity_shards % 256;
        let burst         = burst % sbx_specs::SBX_MAX_BURST_ERR_RESISTANCE;

        let data_par_burst = Some((data_shards, parity_shards, burst));

        let index = calc_data_block_write_index(seq_num,
                                                None,
                                                data_par_burst);

        let seq_num_from_index = calc_seq_num_at_index(index,
                                                       None,
                                                       data_par_burst);

        seq_num_from_index == seq_num
    }

    fn qc_data_block_write_pos_consistent_rs_disabled(seq_num : u32,
                                                      meta_enabled : Option<bool>) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };
        calc_data_block_write_index(seq_num,
                                    meta_enabled,
                                    None) * ver_to_block_size(Version::V1) as u64
            == calc_data_block_write_pos(Version::V1,
                                         seq_num,
                                         meta_enabled,
                                         None)
            && calc_data_block_write_index(seq_num,
                                           meta_enabled,
                                           None) * ver_to_block_size(Version::V2) as u64
            == calc_data_block_write_pos(Version::V2,
                                         seq_num,
                                         meta_enabled,
                                         None)
            && calc_data_block_write_index(seq_num,
                                           meta_enabled,
                                           None) * ver_to_block_size(Version::V3) as u64
            == calc_data_block_write_pos(Version::V3,
                                         seq_num,
                                         meta_enabled,
                                         None)
    }

    fn qc_data_block_write_pos_consistent_rs_enabled(seq_num        : u32,
                                                     meta_enabled   : Option<bool>,
                                                     data_par_burst : (usize, usize, usize)) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };
        let data_par_burst = Some(data_par_burst);
            calc_data_block_write_index(seq_num,
                                           meta_enabled,
                                           data_par_burst) * ver_to_block_size(Version::V17) as u64
            == calc_data_block_write_pos(Version::V17,
                                         seq_num,
                                         meta_enabled,
                                         data_par_burst)
            && calc_data_block_write_index(seq_num,
                                           meta_enabled,
                                           data_par_burst) * ver_to_block_size(Version::V18) as u64
            == calc_data_block_write_pos(Version::V18,
                                         seq_num,
                                         meta_enabled,
                                         data_par_burst)
            && calc_data_block_write_index(seq_num,
                                           meta_enabled,
                                           data_par_burst) * ver_to_block_size(Version::V19) as u64
            == calc_data_block_write_pos(Version::V19,
                                         seq_num,
                                         meta_enabled,
                                         data_par_burst)
    }

    fn qc_meta_block_write_indices_data_block_write_indices_disjoint_rs_disabled(seq_num : u32) -> bool {
        let seq_num = if seq_num == 0 { 1 } else { seq_num };

        let meta_indices = calc_meta_block_all_write_indices(None);

        let data_index = calc_data_block_write_index(seq_num, None, None);

        for &m in meta_indices.iter() {
            if data_index == m {
                return false;
            }
        }

        true
    }
}

#[test]
fn test_sync_to_buffer_simple_cases() {
    let uid : [u8; 6] = [3; 6];

    {
        let mut block = Block::new(sbx_specs::Version::V1,
                                   &uid,
                                   BlockType::Meta);

        let mut buffer : [u8; 512] = [0; 512];

        {
            block.set_seq_num(1);

            block.sync_to_buffer(None, &mut buffer).unwrap();
        }

        let expect : &[u8; 512] = b"SBx\x01\x3F\x56\x03\x03\x03\x03\x03\x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

        for i in 0..512 {
            assert_eq!(expect[i], buffer[i]);
        }
    }
    {
        let mut block = Block::new(sbx_specs::Version::V1,
                                   &uid,
                                   BlockType::Data);

        let mut buffer : [u8; 512] = [0; 512];

        {
            block.set_seq_num(0);

            block.sync_to_buffer(None, &mut buffer).unwrap();
        }

        let expect : &[u8; 512] = b"SBx\x01\x33\x3B\x03\x03\x03\x03\x03\x03\x00\x00\x00\x00\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A";

        for i in 0..512 {
            assert_eq!(expect[i], buffer[i]);
        }
    }
}

#[test]
fn test_sync_from_buffer_simple_cases() {
    {
        let template : &[u8; 512] = b"SBx\x01\x3F\x56\x03\x03\x03\x03\x03\x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

        let mut block = Block::new(sbx_specs::Version::V1,
                                   &[0; 6],
                                   BlockType::Meta);

        block.sync_from_buffer(template, None).unwrap();

        assert_eq!(BlockType::Data, block.block_type());
        assert!(block.is_data());
        assert!(!block.is_meta());
    }
    {
        let template : &[u8; 512] = b"SBx\x01\x94\xBD\x03\x03\x03\x03\x03\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

        let mut block = Block::new(sbx_specs::Version::V1,
                                   &[0; 6],
                                   BlockType::Data);

        block.sync_from_buffer(template, None).unwrap();

        assert_eq!(BlockType::Meta, block.block_type());
        assert!(!block.is_data());
        assert!(block.is_meta());
    }
    {
        let template : &[u8; 512] = b"SBx\x01\x33\x3B\x03\x03\x03\x03\x03\x03\x00\x00\x00\x00\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A";
        let mut block = Block::new(sbx_specs::Version::V1,
                                   &[0; 6],
                                   BlockType::Data);

        block.sync_from_buffer(template, None).unwrap();

        assert_eq!(BlockType::Meta, block.block_type());
        assert!(!block.is_data());
        assert!(block.is_meta());
    }
}

#[test]
fn test_seq_num_is_parity_simple_cases() {
    assert_eq!(false, seq_num_is_parity(0, 0, 0));
    assert_eq!(false, seq_num_is_parity(0, 1, 1));
    assert_eq!(false, seq_num_is_parity(0, 128, 128));

    assert_eq!(false, seq_num_is_parity(1, 3, 2));
    assert_eq!(false, seq_num_is_parity(2, 3, 2));
    assert_eq!(false, seq_num_is_parity(3, 3, 2));
    assert_eq!(true,  seq_num_is_parity(4, 3, 2));
    assert_eq!(true,  seq_num_is_parity(5, 3, 2));

    assert_eq!(false, seq_num_is_parity(6, 3, 2));
    assert_eq!(false, seq_num_is_parity(7, 3, 2));
    assert_eq!(false, seq_num_is_parity(8, 3, 2));
    assert_eq!(true,  seq_num_is_parity(9, 3, 2));
    assert_eq!(true,  seq_num_is_parity(10, 3, 2));

    assert_eq!(false, seq_num_is_parity(11, 3, 2));
    assert_eq!(false, seq_num_is_parity(12, 3, 2));
    assert_eq!(false, seq_num_is_parity(13, 3, 2));
    assert_eq!(true,  seq_num_is_parity(14, 3, 2));
    assert_eq!(true,  seq_num_is_parity(15, 3, 2));
}
