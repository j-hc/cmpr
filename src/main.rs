#![no_main]
#![feature(slice_as_chunks)]

use memmap::MmapOptions;
use std::{fs::File, path::Path};

fn print(s: &str) {
    unsafe { libc::printf(s.as_ptr() as *const libc::c_char) };
}

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    let mut args = RawArgs::new(argc, argv);
    let _ = args.next().unwrap();
    if args.len() < 2 {
        if let Some(arg) = args.next() && arg == "--version" {
            print("cmpr by j-hc (github.com/j-hc)\nLicense GPL-3.0\n\0");
        } else {
            print("Usage: cmpr <file_path> <file_path>\n\0");
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

    let mmap1 = unsafe { MmapOptions::new().map(&f1).ok()? };
    let mmap2 = unsafe { MmapOptions::new().map(&f2).ok()? };

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

pub struct RawArgs {
    argc: i32,
    #[cfg(target_os = "android")]
    argv: *const *const u8,
    #[cfg(target_os = "linux")]
    argv: *const *const i8,
}

impl ExactSizeIterator for RawArgs {
    fn len(&self) -> usize {
        self.argc as usize
    }
}

impl RawArgs {
    pub fn new(argc: i32, argv: *const *const u8) -> Self {
        Self {
            argc,
            argv: argv as _,
        }
    }
}

impl Iterator for RawArgs {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.argc == 0 {
            return None;
        }
        unsafe {
            let b = std::slice::from_raw_parts(*self.argv as _, libc::strlen(*self.argv));
            let s = std::str::from_utf8_unchecked(b);
            self.argv = self.argv.add(1);
            self.argc -= 1;
            Some(s)
        }
    }
}
