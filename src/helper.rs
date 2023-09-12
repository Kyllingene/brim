use std::{fmt::Display, process::exit, io::{stdin, Read, Stdin}, sync::Mutex};

static STDIN_ITER_EXISTS: Mutex<bool> = Mutex::new(false);

pub fn err(msg: impl Display, e: impl Display) -> ! {
    eprintln!("{0}[38:5:1m{1}: {2}{0}[0m", 27 as char, msg, e);
    exit(1);
}

#[inline]
pub fn wrap_add(i: usize, c: usize) -> usize {
    (i + c) % 30000
}

#[inline]
pub fn wrap_sub(i: usize, c: usize) -> usize {
    if i < c {
        30000 - (c - i)
    } else {
        i - c
    }
}

/// An iterator over bytes from stdin.
/// 
/// Uses a buffer internally.
pub struct ReadIter<T: Read> {
    read: T,
    buf: [u8; 16],
    len: usize,
}

impl ReadIter<Stdin> {
    pub fn new() -> Self {
        if *STDIN_ITER_EXISTS.lock().unwrap() {
            panic!("only one StdinIter can be initialized");
        }

        *STDIN_ITER_EXISTS.lock().unwrap() = true;

        Self {
            read: stdin(),
            buf: [0; 16],
            len: 0,
        }
    }
}

impl<T: Read> ReadIter<T> {
    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.buf[self.len])
        }
    }
}

impl<T: Read> Iterator for ReadIter<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            self.pop()
        } else {
            let read = self.read.read(&mut self.buf).unwrap_or_else(|e| err("failed to read from stdin", e));
            self.len = read;
            self.pop()
        }
    }
}
