use std::{fmt::Display, process::exit, io::{stdin, Read}};

#[cfg(debug_assertions)]
use std::sync::Mutex;

/// Prevents multiple ReadIter's over stdin from being made.
#[cfg(debug_assertions)]
static STDIN_ITER_EXISTS: Mutex<bool> = Mutex::new(false);

/// Prints to stderr (in the format "msg: e") and exits.
pub fn err(msg: impl Display, e: impl Display) -> ! {
    eprintln!("{0}[38:5:1m{1}: {2}{0}[0m", 27 as char, msg, e);
    exit(1);
}

/// Prints a warning to stderr.
pub fn warn(msg: impl Display) {
    eprintln!("{0}[38:5:3m{1}{0}[0m", 27 as char, msg);
}

/// Wraps an addition around 30000.
#[inline]
pub fn wrap_add(i: usize, c: usize) -> usize {
    (i + c) % 30000
}

/// Wraps a subtraction around 30000.
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
/// Uses a 32-byte buffer internally.
pub struct ReadIter {
    read: Box<dyn Read>,
    buf: [u8; 32],
    len: usize,
}

impl ReadIter {
    /// Initializes a new ReadIter.
    pub fn new(read: Box<dyn Read>) -> Self {
        Self {
            read,
            buf: [0; 32],
            len: 0,
        }
    }

    /// Initializes a new ReadIter over stdin.
    pub fn stdin() -> Self {
        #[cfg(debug_assertions)]
        if *STDIN_ITER_EXISTS.lock().unwrap() {
            panic!("only one StdinIter can be initialized");
        }

        #[cfg(debug_assertions)]
        { *STDIN_ITER_EXISTS.lock().unwrap() = true; }

        Self::new(Box::new(stdin()))
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.buf[self.len])
        }
    }
}

impl Iterator for ReadIter {
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
