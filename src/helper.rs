use std::cmp::Ordering;
use std::fmt::Display;
use std::io::{stdin, Read};
use std::process::exit;

#[cfg(any(debug_assertions, feature = "debug"))]
use std::sync::Mutex;

use crate::{Cell, CellMod};

/// Prevents multiple ReadIter's over stdin from being made.
#[cfg(any(debug_assertions, feature = "debug"))]
static STDIN_ITER_EXISTS: Mutex<bool> = Mutex::new(false);

/// Prints to stderr (in the format "msg: e") in red, then exits.
pub fn err(msg: impl Display, e: impl Display) -> ! {
    eprintln!("{0}[38:5:1m{1}: {2}{0}[0m", 27 as char, msg, e);
    exit(1);
}

/// Prints a warning to stderr in yellow.
pub fn warn(msg: impl Display) {
    eprintln!("{0}[38:5:3m{1}{0}[0m", 27 as char, msg);
}

/// Wraps an addition/subtraction around array bounds.
///
/// With feature `dynamic_array`, just prevents underflow. Otherwise, wraps
/// around 30000 and 0.
#[inline]
pub fn wrap_goto(x: usize, c: isize) -> usize {
    #[cfg(not(feature = "dynamic_array"))]
    if c > 0 {
        (x + c as usize) % 30000
    } else if c.unsigned_abs() > x {
        30000 - (c.unsigned_abs() - x)
    } else {
        x - c.unsigned_abs()
    }

    #[cfg(feature = "dynamic_array")]
    x.saturating_add_signed(c)
}

/// Wraps an addition/subtraction around 255 and 0.
///
/// With feature `nowrap`, instead prevents over/underflow without wrapping.
#[inline]
pub fn wrap_cell(x: Cell, c: CellMod) -> Cell {
    #[cfg(not(feature = "nowrap"))]
    return (x as CellMod).wrapping_add(c) as Cell;

    #[cfg(feature = "nowrap")]
    return (x as CellMod).saturating_add(c) as Cell;
}

/// Turns a relative index into left/right symbols.
#[inline]
pub fn left_right(i: isize) -> String {
    match i.cmp(&0) {
        Ordering::Greater => ">".repeat(i as usize),
        Ordering::Less => "<".repeat(-i as usize),
        Ordering::Equal => String::new(),
    }
}

/// An iterator over bytes from a [`Read`]able source.
/// Provides a `stdin` method for creating a `ReadIter` over [`stdin`].
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

    /// Initializes a new `ReadIter` over [`stdin`].
    ///
    /// # Panics
    /// Panics if a `ReadIter` already exists, and debug assertions are enabled.
    /// If not, the two iterators will fight for input, causing race conditions.
    pub fn stdin() -> Self {
        #[cfg(debug_assertions)]
        if *STDIN_ITER_EXISTS.lock().unwrap() {
            panic!("only one StdinIter can be initialized");
        } else {
            *STDIN_ITER_EXISTS.lock().unwrap() = true;
        }

        Self::new(Box::new(stdin()))
    }

    /// Pops a value from the buffer, unless it is empty.
    /// Doesn't read anything if the buffer is empty.
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
            let read = self
                .read
                .read(&mut self.buf)
                .unwrap_or_else(|e| err("failed to read from stdin", e));
            self.len = read;
            self.pop()
        }
    }
}

#[cfg(any(debug_assertions, feature = "debug"))]
impl Drop for ReadIter {
    fn drop(&mut self) {
        *STDIN_ITER_EXISTS.lock().unwrap() = false;
    }
}
