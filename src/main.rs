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

sarge! {
    Args,

    'h' help: bool,
    #ok 'i' input: String,
    #ok 'o' output: String,
}

fn main() {
    let (args, files) = Args::parse().unwrap_or_else(|e| err("failed to parse arguments", e));

    if args.help || files.is_empty() {
        warn(include_str!("usage.txt"));

        return;
    }

    let mut stdin = if let Some(i) = args.input {
        let file = File::open(i).unwrap_or_else(|e| err("failed to open input file", e));

        ReadIter::new(Box::new(file))
    } else {
        ReadIter::stdin()
    };

    if let Some(o) = args.output {
        let mut file = File::create(o).unwrap_or_else(|e| err("failed to open output file", e));

        for filename in files.iter().skip(1) {
            let input = fs::read_to_string(filename).unwrap_or_else(|e| err("failed", e));

            let toks = parse(&input);
            let toks = optimize(&toks);

            interpret(&toks, &mut stdin, &mut file);
        }
    } else {
        for filename in files.iter().skip(1) {
            let input = fs::read_to_string(filename).unwrap_or_else(|e| err("failed", e));

            let toks = parse(&input);
            let toks = optimize(&toks);

            interpret(&toks, &mut stdin, &mut stdout());
        }
    }
}
