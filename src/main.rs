#![no_main]
#![feature(slice_as_chunks)]

use cmpr::{cprint, Mmap, RawArgs};
use std::{fs::File, os::unix::prelude::AsRawFd, path::Path};

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    let mut args = RawArgs::new(argc, argv);
    let _ = args.next().unwrap();
    if args.len() < 2 {
        if let Some(arg) = args.next() && arg == "--version" {
            cprint("cmpr by j-hc (github.com/j-hc)\nLicense GPL-3.0\n\0");
        } else {
            cprint("Usage: cmpr <file_path> <file_path>\n\0");
        }
        return 1;
    }

    match run(args.next().unwrap(), args.next().unwrap()) {
        Some(true) => 0,
        _ => 1,
    }
}

fn run(f1_path: impl AsRef<Path>, f2_path: impl AsRef<Path>) -> Option<bool> {
    const CHUNK_SIZE: usize = 5840;

    let f1 = File::open(f1_path).ok()?;
    let f1_size = f1.metadata().ok()?.len();

    let f2 = File::open(f2_path).ok()?;
    let f2_size = f2.metadata().ok()?.len();

    if f1_size != f2_size {
        return Some(false);
    }
    let mmap1 = Mmap::new(f1.as_raw_fd(), f1_size as _).unwrap();
    let mmap2 = Mmap::new(f2.as_raw_fd(), f2_size as _).unwrap();

    let (chunks1, r1) = mmap1.as_chunks::<CHUNK_SIZE>();
    let (chunks2, r2) = mmap2.as_chunks::<CHUNK_SIZE>();
    for (b1, b2) in chunks1.iter().zip(chunks2.iter()) {
        if b1 != b2 {
            println!("{b1:?}");
            println!("{b2:?}");
            return Some(false);
        }
    }
    Some(r1 == r2)
}
