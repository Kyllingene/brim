use std::fmt::Display;

use crate::err;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Add(u8),
    Sub(u8),
    Right(usize),
    Left(usize),
    LBrack(usize),
    RBrack(usize),
    Out,
    In,

    Zero,
    Set(u8),

    #[cfg(any(debug_assertions, feature = "debug"))]
    Dump,
}

pub fn parse(input: &str) -> Vec<Token> {
    let mut input = input.to_string();
    while let Some(_) = input.find("<>") {
        input = input.replace("<>", "");
    }

    while let Some(_) = input.find("><") {
        input = input.replace("><", "");
    }

    while let Some(_) = input.find("+-") {
        input = input.replace("+-", "");
    }

    while let Some(_) = input.find("-+") {
        input = input.replace("-+", "");
    }

    let mut toks = Vec::new();

    for ch in input.chars() {
        match ch {
            '+' => {
                if let Some(Token::Add(i)) = toks.last().copied() {
                    toks.pop();
                    toks.push(Token::Add(i.wrapping_add(1)));
                } else {
                    toks.push(Token::Add(1));
                }
            }
            '-' => {
                if let Some(Token::Sub(i)) = toks.last().copied() {
                    toks.pop();
                    toks.push(Token::Sub(i.wrapping_add(1)));
                } else {
                    toks.push(Token::Sub(1));
                }
            }
            '>' => {
                if let Some(Token::Right(i)) = toks.last().copied() {
                    toks.pop();
                    toks.push(Token::Right(i + 1));
                } else {
                    toks.push(Token::Right(1));
                }

            }
            '<' => {
                if let Some(Token::Left(i)) = toks.last().copied() {
                    toks.pop();
                    toks.push(Token::Left(i + 1));
                } else {
                    toks.push(Token::Left(1));
                }
            }
            '[' => {
                toks.push(Token::LBrack(0));
            }
            ']' => {
                toks.push(Token::RBrack(0));
            }
            '.' => {
                toks.push(Token::Out);
            }
            ',' => {
                toks.push(Token::In);
            }

            #[cfg(any(debug_assertions, feature = "debug"))]
            ';' => {
                toks.push(Token::Dump);
            }

            _ => {}
        }
    }

    optimize(toks)
}

fn optimize(toks: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::new();
    let mut lbracks = Vec::new();

    let mut si = 0;
    let mut nsi = 0;
    while si < toks.len() {
        let mut tok = toks[si];

        if matches!(tok, Token::LBrack(_)) {
            if matches!(toks.get(si+1), Some(Token::Sub(_))) {
                if matches!(toks.get(si+2), Some(Token::RBrack(_))) {
                    if let Some(Token::Add(i)) = toks.get(si+3) {
                        out.push(Token::Set(*i));

                        si += 4;
                    } else {
                        out.push(Token::Zero);
                        
                        si += 3;
                    }

                    nsi += 1;
                    continue;
                }
            }
        }

        if matches!(tok, Token::LBrack(_)) {
            lbracks.push(nsi);
        } else if matches!(tok, Token::RBrack(_)) {
            let lb = lbracks.pop().unwrap_or_else(|| err("invalid input", "unmatched closing bracket"));
            out[lb] = Token::LBrack(nsi);
            tok = Token::RBrack(lb);
        }

        out.push(tok);
        si += 1;
        nsi += 1;
    }

    out
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Add(i) => write!(f, "{}", "+".repeat(*i as usize)),
            Token::Sub(i) => write!(f, "{}", "-".repeat(*i as usize)),
            Token::Right(i) => write!(f, "{}", ">".repeat(*i)),
            Token::Left(i) => write!(f, "{}", "<".repeat(*i)),
            Token::Set(i) => write!(f, "[-]{}", "+".repeat(*i as usize)),
            Token::LBrack(_) => write!(f, "["),
            Token::RBrack(_) => write!(f, "]"),
            Token::Out => write!(f, "."),
            Token::In => write!(f, ","),
            Token::Zero => write!(f, "[-]"),

            #[cfg(any(debug_assertions, feature = "debug"))]
            Token::Dump => write!(f, ";"),
        }
    }
}
