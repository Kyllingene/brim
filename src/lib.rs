pub mod helper;
pub mod token;

use std::io::{BufWriter, Write};

use helper::{err, wrap_cell, wrap_goto};
use token::Token;

pub use token::parse;

#[cfg(not(any(feature = "wide_cell", feature = "signed_cell")))]
type Cell = u8;
#[cfg(not(any(feature = "wide_cell", feature = "signed_cell")))]
type CellMod = i16;

#[cfg(all(feature = "wide_cell", not(feature = "signed_cell")))]
type Cell = u64;
#[cfg(all(feature = "wide_cell", not(feature = "signed_cell")))]
type CellMod = i128;

#[cfg(all(not(feature = "wide_cell"), feature = "signed_cell"))]
type Cell = i8;
#[cfg(all(not(feature = "wide_cell"), feature = "signed_cell"))]
type CellMod = i8;

#[cfg(all(feature = "wide_cell", feature = "signed_cell"))]
type Cell = i64;
#[cfg(all(feature = "wide_cell", feature = "signed_cell"))]
type CellMod = i64;

/// The core of brim: the interpreter.
/// `stdin` must be an iterator over `u8`, but you can use
/// [`ReadIter`](helper::ReadIter) to easily create a buffered iterator over
/// any [`Read`](std::io::Read)able type.
///
/// Since [`Token`] is `Copy`, this takes a slice instead of a `Vec`. `stdout`
/// just has to be [`Write`](std::io::Write)able. However, it's wrapped
/// internally by a [`BufWriter`](std::io::BufWriter), so **don't bother
/// buffering it.**
pub fn interpret(code: &[Token], stdin: &mut impl Iterator<Item = u8>, stdout: &mut impl Write) {
    let mut stdout = BufWriter::new(stdout);

    #[cfg(not(feature = "dynamic_array"))]
    let mut tape = [0 as Cell; 30000];

    #[cfg(feature = "dynamic_array")]
    let mut tape: Vec<Cell> = Vec::with_capacity(1024);

    let mut sp = 0;
    let mut ip = 0;

    #[cfg(any(debug_assertions, feature = "debug"))]
    let mut highest = 0;

    while ip < code.len() {
        let tok = code[ip];

        match tok {
            Token::Inc(i) => tape[sp] = wrap_cell(tape[sp], i as CellMod),
            Token::Dec(i) => tape[sp] = wrap_cell(tape[sp], -(i as CellMod)),
            Token::Goto(i) => sp = wrap_goto(sp, i),
            Token::In => tape[sp] = stdin.next().unwrap_or(0) as Cell,
            Token::Out => {
                #[allow(clippy::unnecessary_cast)]
                let bytes = tape[sp].to_ne_bytes();
                stdout
                    .write_all(&bytes)
                    .unwrap_or_else(|e| err("failed to write to output", e));

                #[allow(clippy::unnecessary_cast)]
                if tape[sp] as u8 == b'\n' {
                    stdout
                        .flush()
                        .unwrap_or_else(|e| err("failed to flush stdout", e));
                }
            }

            Token::LBrack(i) => {
                if tape[sp] == 0 {
                    ip = i;
                }
            }

            Token::RBrack(i) => {
                if tape[sp] != 0 {
                    ip = i;
                }
            }

            Token::Zero => tape[sp] = 0,
            Token::Set(i) => tape[sp] = i,
            Token::Add(i, mul) => {
                let v = tape[sp];
                let cell = &mut tape[wrap_goto(sp, i)];
                *cell = wrap_cell(*cell, (v * mul) as CellMod);
                tape[sp] = 0;
            }
            Token::Sub(i) => {
                let v = tape[sp];
                let cell = &mut tape[wrap_goto(sp, i)];
                *cell = wrap_cell(*cell, -(v as CellMod));
                tape[sp] = 0;
            }
            Token::Dup(i1, mul1, i2, mul2) => {
                let v = tape[sp];

                let cell = &mut tape[wrap_goto(sp, i1)];
                *cell = wrap_cell(*cell, (v * mul1) as CellMod);

                let cell = &mut tape[wrap_goto(sp, i1 + i2)];
                *cell = cell.wrapping_add(v * mul2);

                tape[sp] = 0;
            }
            Token::Scan(i) => {
                while tape[sp] != 0 {
                    sp = wrap_goto(sp, i);
                }
            }

            #[cfg(any(debug_assertions, feature = "debug"))]
            Token::Dump => {
                highest = highest.max(sp);

                let left = if ip == 0 {
                    " ".to_string()
                } else {
                    code[ip - 1].to_string()
                };

                let cur = code[ip].to_string();

                let right = if ip >= code.len() - 1 {
                    " ".to_string()
                } else {
                    code[ip + 1].to_string()
                };

                eprint!("\nsp: 0x{sp:04x}   ctx: {left} {cur} {right}");

                for (tsp, item) in tape.iter().enumerate().take((highest + 1).max(8)) {
                    if tsp % 8 == 0 {
                        eprintln!();
                    } else {
                        eprint!(" | ");
                    }

                    eprint!("0x{tsp:04x} : 0x{:02x}", item);
                }

                eprintln!();
            }
        }

        ip += 1;

        #[cfg(any(debug_assertions, feature = "debug"))]
        {
            highest = highest.max(sp);
        }
    }

    stdout
        .flush()
        .unwrap_or_else(|e| err("failed to flush stdout", e));
}
