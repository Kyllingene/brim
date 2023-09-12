use std::{
    fs::{self, File},
    io::stdout,
};

use sarge::prelude::*;

use brim::{
    helper::{err, warn, ReadIter},
    interpret, parse,
    token::optimize,
};

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(str, both, 'i', "input"));
    parser.add(arg!(str, both, 'o', "output"));

    let files = parser
        .parse()
        .unwrap_or_else(|e| err("failed to parse arguments", e));

    if get_flag!(parser, both, 'h', "help") || files.is_empty() {
        warn(include_str!("usage.txt"));

        return;
    }

    for filename in files {
        let input = fs::read_to_string(filename).unwrap_or_else(|e| err("failed", e));

        let toks = parse(&input);
        let toks = optimize(&toks);

        let stdin = if let Some(out) = get_arg!(parser, both, 'i', "input").unwrap().val.clone() {
            let file =
                File::open(out.get_str()).unwrap_or_else(|e| err("failed to open input file", e));

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
