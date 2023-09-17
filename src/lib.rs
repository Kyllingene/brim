pub mod helper;
pub mod token;

use std::io::{BufWriter, Write};

use helper::{err, wrap_goto};
use token::Token;

pub use token::parse;

/// The core of brim: the interpreter.
/// `stdin` must be an iterator over `u8`, but you can use
/// [`ReadIter`](helper::ReadIter) to easily create a buffered iterator over
/// any [`Read`](std::io::Read)able type.
///
/// Since [`Token`] is `Copy`, this takes a slice instead of a `Vec`. It's
/// other arguments, `stdin` and `stdout`, are also generic enough to provide
/// for many use cases.
pub fn interpret(code: &[Token], stdin: &mut impl Iterator<Item = u8>, stdout: &mut impl Write) {
    let mut stdout = BufWriter::new(stdout);

    let mut tape = [0u8; 30000];
    let mut sp = 0;
    let mut ip = 0;

    #[cfg(any(debug_assertions, feature = "debug"))]
    let mut highest = 0;

    while ip < code.len() {
        let tok = code[ip];

        match tok {
            Token::Add(i) => tape[sp] = tape[sp].wrapping_add(i),
            Token::Sub(i) => tape[sp] = tape[sp].wrapping_sub(i),
            Token::Goto(i) => sp = wrap_goto(sp, i),
            Token::In => tape[sp] = stdin.next().unwrap_or_default(),
            Token::Out => {
                stdout
                    .write(&[tape[sp]])
                    .map(|_| ())
                    .unwrap_or_else(|e| err("failed to write to output", e));

                if tape[sp] == b'\n' {
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
            Token::Move(i) => {
                tape[wrap_goto(sp, i)] += tape[sp];
                tape[sp] = 0;
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
