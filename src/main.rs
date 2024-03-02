#![no_main]
#![feature(array_chunks, portable_simd)]

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    let mut args = RawArgs::new(argc, argv);
    let _ = args.next().unwrap();
    if args.len() < 2 {
        cprintln!(
            "Usage: cmpr <file_path> <file_path>\ncmpr by j-hc (github.com/j-hc)\nLicense GPL-3.0"
        );
    }

    match run(args.next().unwrap(), args.next().unwrap()) {
        Ok(true) => 0,
        Err(e) => {
            match e {
                RunErr::IO => cprintln!("ERROR: could not read file"),
                RunErr::MMap => cprintln!("ERROR: could not mmap file"),
            }
            1
        }
        _ => 1,
    }
}

enum RunErr {
    IO,
    MMap,
}

impl From<io::Error> for RunErr {
    fn from(_: io::Error) -> Self {
        Self::IO
    }
}

fn run(p1: &str, p2: &str) -> Result<bool, RunErr> {
    let f1 = File::open(p1)?;
    let f1_size = f1.metadata()?.len();
    let f2 = File::open(p2)?;
    let f2_size = f2.metadata()?.len();
    if f1_size != f2_size {
        return Ok(false);
    }
    let Some(fm1) = Mmap::new(f1.as_raw_fd(), f1_size as _) else {
        return Err(RunErr::MMap);
    };
    let Some(fm2) = Mmap::new(f2.as_raw_fd(), f2_size as _) else {
        return Err(RunErr::MMap);
    };

    Ok(compare(&fm1, &fm2))
}

fn compare(fm1: &[u8], fm2: &[u8]) -> bool {
    const CHUNK_SIZE: usize = 16;
    let mut fm1 = fm1.array_chunks::<CHUNK_SIZE>();
    let mut fm2 = fm2.array_chunks::<CHUNK_SIZE>();

    iter::zip(fm1.by_ref(), fm2.by_ref())
        .all(|(m1, m2)| u8x16::from_slice(m1) == u8x16::from_slice(m2))
        && iter::zip(fm1.remainder(), fm2.remainder()).all(|(m1, m2)| m1 == m2)
}

use cmpr::{cprintln, Mmap, RawArgs};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::simd::u8x16;
use std::{io, iter};
