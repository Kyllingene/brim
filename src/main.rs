use std::{
    fs::{self, File},
    io::{stdout, stdin, BufReader, Read},
};

use sarge::prelude::*;

use brim::{
    helper::{err, warn},
    interpret, parse,
    token::optimize,
};

#[cfg(feature = "debug")]
sarge! {
    Args,

    'h' help: bool,
    #ok 'i' input: String,
    #ok 'o' output: String,

    #ok 'w' @BRIM_DEBUG_WIDTH debug_width: usize,
}

#[cfg(not(feature = "debug"))]
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

        Box::new(BufReader::new(file)) as Box<dyn Read>
    } else {
        Box::new(BufReader::new(stdin())) as Box<dyn Read>
    }.bytes().map(|data| data.unwrap_or_else(|e| err("failed to read input", e)));

    if let Some(o) = args.output {
        let mut file = File::create(o).unwrap_or_else(|e| err("failed to open output file", e));

        for filename in files.iter().skip(1) {
            let input = fs::read_to_string(filename).unwrap_or_else(|e| err("failed", e));

            let toks = parse(&input);
            let toks = optimize(&toks);

            #[cfg(not(feature = "debug"))]
            interpret(&toks, &mut stdin, &mut file);

            #[cfg(feature = "debug")]
            interpret(&toks, &mut stdin, &mut file, args.debug_width.unwrap_or(8));
        }
    } else {
        for filename in files.iter().skip(1) {
            let input = fs::read_to_string(filename).unwrap_or_else(|e| err("failed", e));

            let toks = parse(&input);
            let toks = optimize(&toks);

            #[cfg(not(feature = "debug"))]
            interpret(&toks, &mut stdin, &mut stdout());

            #[cfg(feature = "debug")]
            interpret(&toks, &mut stdin, &mut stdout(), args.debug_width.unwrap_or(8));
        }
    }
}
