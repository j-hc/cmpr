#![no_std]

use core::ops::Deref;

#[macro_export]
macro_rules! cprintln {
    ($s:expr) => {{
        unsafe { libc::printf(concat!($s, "\n\0").as_ptr() as *const libc::c_char) };
    }};
}

pub struct RawArgs {
    argc: i32,
    argv: *const *const u8,
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
            let b = core::slice::from_raw_parts(*self.argv as _, libc::strlen(*self.argv as _));
            let s = core::str::from_utf8_unchecked(b);
            self.argv = self.argv.add(1);
            self.argc -= 1;
            Some(s)
        }
    }
}

pub struct Mmap {
    ptr: *const u8,
    len: usize,
}

impl Mmap {
    pub fn new(raw_fd: libc::c_int, len: usize) -> Option<Self> {
        let ptr = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_SHARED,
                raw_fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            None
        } else {
            Some(Self { ptr: ptr as _, len })
        }
    }
}

impl Deref for Mmap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl AsRef<[u8]> for Mmap {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}
