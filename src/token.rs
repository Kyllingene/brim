use std::fmt::Display;

use crate::{err, helper::left_right};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Add(u8),
    Sub(u8),
    Goto(isize),
    LBrack(usize),
    RBrack(usize),
    Out,
    In,

    Zero,
    Set(u8),
    Move(isize),

    #[cfg(any(debug_assertions, feature = "debug"))]
    Dump,
}

pub fn parse(input: &str) -> Vec<Token> {
    // let input = input.to_string();
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
                if let Some(Token::Goto(i)) = toks.last().copied() {
                    toks.pop();
                    toks.push(Token::Goto(i + 1));
                } else {
                    toks.push(Token::Goto(1));
                }
            }
            '<' => {
                if let Some(Token::Goto(i)) = toks.last().copied() {
                    toks.pop();
                    toks.push(Token::Goto(i - 1));
                } else {
                    toks.push(Token::Goto(-1));
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
            let next = toks.get(si + 1);
            if matches!(next, Some(Token::Sub(1))) {
                let next = toks.get(si + 2);
                if matches!(next, Some(Token::RBrack(_))) {
                    if let Some(Token::Add(i)) = toks.get(si + 3) {
                        out.push(Token::Set(*i));

                        si += 4;
                    } else {
                        out.push(Token::Zero);

                        si += 3;
                    }

                    nsi += 1;
                    continue;
                } else if let Some(Token::Goto(there)) = next {
                    if matches!(toks.get(si + 3), Some(Token::Add(1))) {
                        if let Some(Token::Goto(back)) = toks.get(si + 4) {
                            if there.abs() == back.abs()
                                && matches!(toks.get(si + 5), Some(Token::RBrack(_)))
                            {
                                out.push(Token::Move(*there));

                                si += 6;
                                nsi += 1;
                                continue;
                            }
                        }
                    }
                }
            } else if let Some(Token::Goto(there)) = next {
                if matches!(toks.get(si + 2), Some(Token::Add(1))) {
                    if let Some(Token::Goto(back)) = toks.get(si + 3) {
                        if there.abs() == back.abs()
                            && matches!(toks.get(si + 4), Some(Token::Sub(1)))
                            && matches!(toks.get(si + 5), Some(Token::RBrack(_)))
                        {
                            out.push(Token::Move(*there));

                            si += 6;
                            nsi += 1;
                            continue;
                        }
                    }
                }
            }
        }

        if matches!(tok, Token::LBrack(_)) {
            lbracks.push(nsi);
        } else if matches!(tok, Token::RBrack(_)) {
            let lb = lbracks
                .pop()
                .unwrap_or_else(|| err("invalid input", "unmatched closing bracket"));
            out[lb] = Token::LBrack(nsi);
            tok = Token::RBrack(lb);
        } else if matches!(tok, Token::Goto(0)) {
            si += 1;
            continue;
        }

        out.push(tok);
        si += 1;
        nsi += 1;
    }

    if !lbracks.is_empty() {
        err("invalid input", "unmatched opening bracket");
    }

    out
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Add(i) => write!(f, "{}", "+".repeat(*i as usize)),
            Token::Sub(i) => write!(f, "{}", "-".repeat(*i as usize)),
            Token::Goto(i) => write!(f, "{}", left_right(*i)),
            Token::Set(i) => write!(f, "[-]{}", "+".repeat(*i as usize)),
            Token::LBrack(_) => write!(f, "["),
            Token::RBrack(_) => write!(f, "]"),
            Token::Out => write!(f, "."),

            Token::In => write!(f, ","),
            Token::Zero => write!(f, "[-]"),
            Token::Move(i) => write!(f, "[-{}+{}]", left_right(*i), left_right(-i)),

            #[cfg(any(debug_assertions, feature = "debug"))]
            Token::Dump => write!(f, ";"),
        }
    }
}
