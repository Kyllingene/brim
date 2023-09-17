use std::{
    fs::{self, File},
    io::stdout,
};

use sarge::prelude::*;

use brim::{
    helper::{err, warn, ReadIter, WriteWrapper},
    interpret, parse,
    token::optimize,
};

fn main() {
    let parser = ArgumentParser::new();
    let help = parser.add(tag::both('h', "help"));
    let stdin_arg = parser.add::<String>(tag::both('i', "input"));
    let stdout_arg = parser.add::<String>(tag::both('o', "output"));

    let files = parser
        .parse()
        .unwrap_or_else(|e| err("failed to parse arguments", e));

    if help.get() == Ok(true) || files.is_empty() {
        warn(include_str!("usage.txt"));

        return;
    }

    let mut stdin = if let Ok(i) = stdin_arg.get() {
        let file =
            File::open(i).unwrap_or_else(|e| err("failed to open input file", e));

        ReadIter::new(Box::new(file))
    } else {
        ReadIter::stdin()
    };

    let mut stdout = if let Ok(o) = stdout_arg.get() {
        let file = File::create(o)
                .unwrap_or_else(|e| err("failed to open output file", e));

        WriteWrapper::File(file)
    } else {
        WriteWrapper::Stdout(stdout())
    };

    for filename in files {
        let input = fs::read_to_string(filename).unwrap_or_else(|e| err("failed", e));

        let toks = parse(&input);
        let toks = optimize(&toks);

        interpret(&toks, &mut stdin, &mut stdout);
    }
}
