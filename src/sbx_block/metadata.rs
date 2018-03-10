use super::super::multihash;
use std;
use super::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Metadata {
    FNM(Box<[u8]>),
    SNM(Box<[u8]>),
    FSZ(u64),
    FDT(i64),
    SDT(i64),
    HSH(multihash::HashBytes),
    RSD(u8),
    RSP(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetadataID {
    FNM,
    SNM,
    FSZ,
    FDT,
    SDT,
    HSH,
    RSD,
    RSP,
}

static PREAMBLE_LEN : usize = 3 + 1;

fn single_info_size(meta : &Metadata) -> usize {
    use self::Metadata::*;
    use std::mem;
    match *meta {
        FNM(ref x) | SNM(ref x)  => x.len(),
        FSZ(_) | FDT(_) | SDT(_) => mem::size_of::<u64>(),
        HSH(ref x)               => multihash::specs::Param::new(x.0).total_length(),
        RSD(_) | RSP(_)          => mem::size_of::<u8>(),
    }
}

fn single_meta_size(meta : &Metadata) -> usize {
    PREAMBLE_LEN + single_info_size(meta)
}

pub fn id_to_bytes(id : MetadataID) -> [u8; 3] {
    use self::MetadataID::*;
    match id {
        FNM => [b'F', b'N', b'M'],
        SNM => [b'S', b'N', b'M'],
        FSZ => [b'F', b'S', b'Z'],
        FDT => [b'F', b'D', b'T'],
        SDT => [b'S', b'D', b'T'],
        HSH => [b'H', b'S', b'H'],
        RSD => [b'R', b'S', b'D'],
        RSP => [b'R', b'S', b'P'],
    }
}

pub fn meta_to_id(meta : &Metadata) -> MetadataID {
    match *meta {
        Metadata::FNM(_) => MetadataID::FNM,
        Metadata::SNM(_) => MetadataID::SNM,
        Metadata::FSZ(_) => MetadataID::FSZ,
        Metadata::FDT(_) => MetadataID::FDT,
        Metadata::SDT(_) => MetadataID::SDT,
        Metadata::HSH(_) => MetadataID::HSH,
        Metadata::RSD(_) => MetadataID::RSD,
        Metadata::RSP(_) => MetadataID::RSP,
    }
}

fn single_to_bytes(meta   : &Metadata,
                   buffer : &mut [u8]) -> Result<usize, Error> {
    let total_size = single_meta_size(meta);
    let info_size  = single_info_size(meta);

    if buffer.len() < total_size {
        return Err(Error::TooMuchMetaData);
    }

    use self::Metadata::*;

    // write id
    let id = id_to_bytes(meta_to_id(meta));
    for i in 0..id.len() {
        buffer[i] = id[i];
    }

    // write length
    buffer[3] = info_size as u8;

    let dst = &mut buffer[PREAMBLE_LEN..PREAMBLE_LEN + info_size];

    // write info
    match *meta {
        FNM(ref x) | SNM(ref x) => {
            dst.copy_from_slice(x);
        },
        FSZ(x) => {
            let be_bytes : [u8; 8] =
                unsafe { std::mem::transmute::<u64, [u8; 8]>(x.to_be()) };
            dst.copy_from_slice(&be_bytes);
        },
        FDT(x) | SDT(x) => {
            let be_bytes : [u8; 8] =
                unsafe { std::mem::transmute::<i64, [u8; 8]>(x.to_be()) };
            dst.copy_from_slice(&be_bytes);
        },
        HSH(ref x) => {
            multihash::hash_bytes_to_bytes(x, dst);
        },
        RSD(x) | RSP(x) => {
            dst[0] = x;
        }
    }

    Ok(total_size)
}

pub fn to_bytes(meta   : &[Metadata],
                buffer : &mut [u8])
                -> Result<(), Error> {
    let mut cur_pos = 0;
    for m in meta.iter() {
        let size_written = single_to_bytes(m, &mut buffer[cur_pos..])?;

        cur_pos += size_written;
    }

    // fill the rest with padding 0x1A
    for i in cur_pos..buffer.len() {
        buffer[i] = 0x1A;
    }

    Ok(())
}

mod parsers {
    use super::Metadata;
    use super::Metadata::*;
    use super::super::super::misc_utils;
    use super::super::super::multihash::parsers::multihash_w_len_p;

    use nom::be_u8;
    use nom::be_u64;
    use nom::be_i64;

    macro_rules! make_meta_parser {
        (
            $name:ident, $id:expr, $constructor:path
                => num, $res_parser:ident
        ) => {
            named!($name <Metadata>,
                   do_parse!(
                       _id : tag!($id) >>
                           n : be_u8 >>
                           res : cond_reduce!(n > 0, $res_parser) >>
                           ($constructor(res))
                   )
            );
        };
        (
            $name:ident, $id:expr, $constructor:path => str
        ) => {
            named!($name <Metadata>,
                   do_parse!(
                       tag!($id) >>
                           n : be_u8 >>
                           res : cond_reduce!(n > 0, take!(n)) >>
                           ($constructor(misc_utils::slice_to_vec(res)
                                         .into_boxed_slice()))
                   )
            );
        };
    }

    make_meta_parser!(fnm_p, b"FNM", FNM => str);
    make_meta_parser!(snm_p, b"SNM", SNM => str);
    make_meta_parser!(fsz_p, b"FSZ", FSZ => num, be_u64);
    make_meta_parser!(fdt_p, b"FDT", FDT => num, be_i64);
    make_meta_parser!(sdt_p, b"SDT", SDT => num, be_i64);
    make_meta_parser!(rsd_p, b"RSD", RSD => num, be_u8);
    make_meta_parser!(rsp_p, b"RSP", RSP => num, be_u8);

    named!(hsh_p <Metadata>,
           do_parse!(
               _id : tag!(b"HSH") >>
                   res : multihash_w_len_p >>
                   (HSH(res))
           )
    );

    named!(pub meta_p <Vec<Metadata>>,
           many0!(
               alt!(fnm_p  |
                    snm_p  |
                    fsz_p  |
                    fdt_p  |
                    sdt_p  |
                    hsh_p  |
                    rsd_p  |
                    rsp_p
               )
           )
    );
}

pub fn from_bytes(bytes : &[u8])
                  -> Result<Vec<Metadata>, Error> {
    use nom::IResult;
    match parsers::meta_p(bytes) {
        IResult::Done(_, res) => Ok(res),
        _                     => Err(Error::ParseError)
    }
}

pub fn get_meta_ref_by_id(id    : MetadataID,
                          metas : &[Metadata])
                          -> Option<&Metadata> {
    for m in metas.iter() {
        if meta_to_id(m) == id {
            return Some(m);
        }
    }
    None
}

pub fn get_meta_ref_mut_by_id(id    : MetadataID,
                              metas : &mut [Metadata])
                              -> Option<&mut Metadata> {
    for m in metas.iter_mut() {
        if meta_to_id(m) == id {
            return Some(m);
        }
    }

    None
}
