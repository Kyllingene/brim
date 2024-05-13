use std::cmp::Ordering;
use std::fmt::Display;
use std::process::exit;

use crate::{Cell, CellMod};

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
