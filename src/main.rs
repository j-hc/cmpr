#![no_main]
#![feature(slice_as_chunks)]

use cmpr::{cprintln, Mmap, RawArgs};
use std::{fs::File, os::unix::prelude::AsRawFd, path::Path};

mod cmperror {
    use std::io;
    pub enum Error {
        FileReadErr(io::Error),
        MmapErr,
    }

    impl From<io::Error> for Error {
        fn from(e: io::Error) -> Self {
            Self::FileReadErr(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    let mut args = RawArgs::new(argc, argv);
    let _ = args.next().unwrap();
    if args.len() < 2 {
        if let Some(arg) = args.next() && arg == "--version" {
            cprintln!("cmpr by j-hc (github.com/j-hc)\nLicense GPL-3.0");
        } else {
            cprintln!("Usage: cmpr <file_path> <file_path>");
        }
        return 1;
    }

    match run(args.next().unwrap(), args.next().unwrap()) {
        Ok(true) => 0,
        Err(e) => {
            match e {
                cmperror::Error::FileReadErr(_) => cprintln!("ERROR: File read failed"),
                cmperror::Error::MmapErr => cprintln!("ERROR: Memory mapping failed"),
            }
            1
        }
        _ => 1,
    }
}

fn run(f1_path: impl AsRef<Path>, f2_path: impl AsRef<Path>) -> Result<bool, cmperror::Error> {
    const CHUNK_SIZE: usize = 4096;

    let f1 = File::open(f1_path)?;
    let f1_size = f1.metadata()?.len();

    let f2 = File::open(f2_path)?;
    let f2_size = f2.metadata()?.len();
    if f1_size != f2_size {
        return Ok(false);
    }

    let mmap1 = Mmap::new(f1.as_raw_fd(), f1_size as _).ok_or(cmperror::Error::MmapErr)?;
    let mmap2 = Mmap::new(f2.as_raw_fd(), f2_size as _).ok_or(cmperror::Error::MmapErr)?;

    let (chunks1, r1) = mmap1.as_chunks::<CHUNK_SIZE>();
    let (chunks2, r2) = mmap2.as_chunks::<CHUNK_SIZE>();
    Ok(chunks1.iter().zip(chunks2.iter()).all(|(b1, b2)| b1 == b2) && r1 == r2)
}
