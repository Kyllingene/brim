use std::{fs::{self, File}, io::{Write, stdout, BufWriter}};

use sarge::prelude::*;

mod helper;
mod token;

use helper::{ReadIter, err, warn, wrap_add, wrap_sub};
use token::{parse, Token};

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(str, both, 'i', "input"));
    parser.add(arg!(str, both, 'o', "output"));

    let files = parser.parse().unwrap_or_else(|e| err("failed to parse arguments", e));

    if get_flag!(parser, both, 'h', "help") || files.is_empty() {
        warn(include_str!("usage.txt"));
        
        return;
    }

    for filename in files {
        let input = fs::read_to_string(filename)
            .unwrap_or_else(|e|
                err("failed", e)
            );

        let toks = parse(&input);

        let stdin = if let Some(out) = get_arg!(parser, both, 'i', "input").unwrap().val.clone() {
            let file = File::open(out.get_str())
                .unwrap_or_else(|e| err("failed to open input file", e));

            ReadIter::new(Box::new(file))
        } else {
            ReadIter::stdin()
        };

        if let Some(out) = get_arg!(parser, both, 'o', "output").unwrap().val.clone() {
            let file = File::create(out.get_str())
                .unwrap_or_else(|e| err("failed to open output file", e));

                interpret(&toks, stdin, file);
        } else {
            interpret(&toks, stdin, stdout());
        }
    }
}

fn interpret(code: &[Token], mut stdin: impl Iterator<Item = u8>, stdout: impl Write) {
    let mut stdout = BufWriter::new(stdout);

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
            Token::Out => {
                stdout.write(&[tape[sp]]).map(|_| ()).unwrap_or_else(|e| err("failed to write to output", e));

                if tape[sp] == b'\n' {
                    stdout.flush().unwrap_or_else(|e| err("failed to flush stdout", e));
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

    stdout.flush().unwrap_or_else(|e| err("failed to flush stdout", e));
}
