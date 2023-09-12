use std::{env, fs, io::{Write, stdout}};

mod helper;
mod token;

use helper::{ReadIter, err, wrap_add, wrap_sub};
use token::{parse, Token};

fn main() {
    if let Some(filename) = env::args().nth(1) {
        let input = fs::read_to_string(filename)
            .unwrap_or_else(|e|
                err("failed", e)
            );

        let toks = parse(&input);

        interpret(&toks, ReadIter::new(), stdout());
    }
}

fn interpret(code: &[Token], mut stdin: impl Iterator<Item = u8>, mut stdout: impl Write) {
    let mut tape = [0u8; 30000];
    let mut sp = 0;
    let mut ip = 0;

    #[cfg(any(debug_assertions, feature = "debug"))]
    let mut highest = 0;

    while ip < code.len() {
        let tok = code[ip];

        match tok {
            Token::Add(i) => tape[sp] = tape[sp].wrapping_add(i as u8),
            Token::Sub(i) => tape[sp] = tape[sp].wrapping_sub(i as u8),
            Token::Right(i) => sp = wrap_add(sp, i),
            Token::Left(i) => sp = wrap_sub(sp, i),
            Token::In => tape[sp] = stdin.next().unwrap_or_default(),
            Token::Out => stdout.write(&[tape[sp]]).map(|_| ()).unwrap_or_else(|e| err("failed to write to output", e)),

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

            #[cfg(any(debug_assertions, feature = "debug"))]
            Token::Dump => {
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

                eprint!("\nsp: 0x{sp:04x}   ctx: {left}{cur}{right}");

                for tsp in 0..highest.max(8) {
                    if tsp % 8 == 0 {
                        eprintln!();
                    } else {
                        eprint!(" | ");
                    }

                    eprint!("0x{tsp:04x} : 0x{:02x}", tape[tsp]);
                }

                eprintln!();
            }
        }

        ip += 1;

        #[cfg(any(debug_assertions, feature = "debug"))]
        { highest = highest.max(sp); }
    }
}
