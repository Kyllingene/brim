use std::fmt::Display;

use crate::{err, helper::left_right, Cell};

/// An optimized token; it may represent more than one brain* instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// Equivalent to `'+' * n`.
    Inc(Cell),
    /// Equivalent to `'-' * n`.
    Dec(Cell),
    /// Equivalent to `'>' * n` if `n > 0`, else `'<' * n`.
    Goto(isize),
    /// A left bracket. The value is the index of the corresponding right
    /// bracket.
    ///
    /// *Note*: [`parse`] leaves this set to 0, since [`optimize`] will change
    /// the indices anyways.
    LBrack(usize),
    /// A right bracket. The value is the index of the corresponding left
    /// bracket.
    ///
    /// *Note*: [`parse`] leaves this set to 0, since [`optimize`] will change
    /// the indices anyways.
    RBrack(usize),
    /// Equivalent to `'.'`.
    Out,
    /// Equivalent to `','`.
    In,

    /// Macro-optimization. Equivalent to `[-]`.
    Zero,
    /// Macro-optimization. Equivalent to `[-]+`.
    Set(Cell),
    /// Macro-optimization. Equivalent to `[->+<]`.
    Add(isize, Cell),
    /// Macro-optimization. Equivalent to `[->-<]`.
    Sub(isize),
    /// Macro-optimization. Equivalent to `[->+>+<<]`.
    Dup(isize, Cell, isize, Cell),
    /// Macro-optimization. Equivalent to `[>>>]`.
    Scan(isize),

    /// Macro-optimization. Equivalent to `[]`.
    End,

    /// Equivalent to `';'`. Only available if in debug mode, or if feature
    /// flag `debug` is enabled.
    #[cfg(any(debug_assertions, feature = "debug"))]
    Dump,
}

/// Parse brain* input into [`Token`]s. Groups together [`Inc`](Token::Inc) and
/// [`Dec`](Token::Dec) instructions, as well as converting `<` and `>` to
/// [`Goto`](Token::Goto).
///
/// ***Note:*** the output is *not yet valid!* Each [`LBrack`](Token::LBrack)
/// and [`RBrack`](Token::RBrack) still needs to be set to its match, i.e. via
/// [`optimize`].
pub fn parse(input: &str) -> Vec<Token> {
    // let input = input.to_string();
    let mut toks = Vec::new();

    for ch in input.chars() {
        match ch {
            '+' => {
                if let Some(&Token::Inc(i)) = toks.last() {
                    toks.pop();
                    toks.push(Token::Inc(i.wrapping_add(1)));
                } else {
                    toks.push(Token::Inc(1));
                }
            }
            '-' => {
                if let Some(&Token::Dec(i)) = toks.last() {
                    toks.pop();
                    toks.push(Token::Dec(i.wrapping_add(1)));
                } else {
                    toks.push(Token::Dec(1));
                }
            }
            '>' => {
                if let Some(&Token::Goto(i)) = toks.last() {
                    toks.pop();
                    toks.push(Token::Goto(i + 1));
                } else {
                    toks.push(Token::Goto(1));
                }
            }
            '<' => {
                if let Some(&Token::Goto(i)) = toks.last() {
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

    toks
}

/// Performs macro-optimizations, and sets the final indices for each bracket.
pub fn optimize(toks: &[Token]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut lbracks = Vec::new();

    let mut si = 0;
    while si < toks.len() {
        let mut tok = toks[si];

        if matches!(tok, Token::LBrack(_)) {
            let next = toks.get(si + 1);

            // [
            // Zero / Set / Add / Sub / Dup
            if matches!(next, Some(Token::Dec(1))) {
                let next = toks.get(si + 2);

                // [-
                // Zero / Set
                if matches!(next, Some(Token::RBrack(_))) {
                    // [-]
                    if let Some(Token::Inc(i)) = toks.get(si + 3) {
                        // [-]+++
                        out.push(Token::Set(*i));

                        si += 4;
                    } else {
                        out.push(Token::Zero);

                        si += 3;
                    }

                    continue;

                // [-
                // Add / Sub / Dup
                } else if let Some(Token::Goto(there)) = next {
                    let next = toks.get(si + 3);

                    // [->
                    // Add / Dup
                    if let Some(Token::Inc(mul)) = next {
                        if let Some(Token::Goto(back)) = toks.get(si + 4) {
                            let next = toks.get(si + 5);

                            // [->+ ><
                            // Add
                            if *there == -back {
                                // [->+<
                                if matches!(next, Some(Token::RBrack(_))) {
                                    // [->+<]
                                    out.push(Token::Add(*there, *mul));

                                    si += 6;
                                    continue;
                                }

                            // [->+ ><
                            // Dup
                            } else if let Some(Token::Inc(mul2)) = next {
                                // [->+ >< +
                                if let Some(Token::Goto(bi)) = toks.get(si + 6) {
                                    // [->+ >< + ><
                                    if bi + there + back == 0
                                        // [->+>+<<]
                                        && matches!(toks.get(si + 7), Some(Token::RBrack(_)))
                                    {
                                        out.push(Token::Dup(*there, *mul, *back, *mul2));

                                        si += 8;
                                        continue;
                                    }
                                }
                            }
                        }

                    // Sub
                    } else if matches!(next, Some(Token::Dec(1))) {
                        if let Some(Token::Goto(back)) = toks.get(si + 4) {
                            if *there == -back && matches!(toks.get(si + 5), Some(Token::RBrack(_)))
                            {
                                out.push(Token::Sub(*there));

                                si += 6;
                                continue;
                            }
                        }
                    }
                }

            // [
            // Add / Sub / Dup / Scan
            } else if let Some(Token::Goto(there)) = next {
                let next = toks.get(si + 2);

                // [>
                // Add / Dup
                if let Some(Token::Inc(mul)) = next {
                    if let Some(Token::Goto(back)) = toks.get(si + 3) {
                        let next = toks.get(si + 4);

                        // [>+ ><
                        // Add
                        if *there == -back
                            && matches!(next, Some(Token::Dec(1)))
                            && matches!(toks.get(si + 5), Some(Token::RBrack(_)))
                        {
                            // [>+<-]
                            out.push(Token::Add(*there, *mul));

                            si += 6;
                            continue;

                        // [>+ ><
                        // Dup
                        } else if let Some(Token::Inc(mul2)) = next {
                            // [>+ >< +
                            if let Some(Token::Goto(bi)) = toks.get(si + 5) {
                                if *bi == -(there + back)
                                    // [>+>+<<
                                    && matches!(toks.get(si + 6), Some(Token::Dec(1)))
                                    // [>+>+<<-
                                    && matches!(toks.get(si + 7), Some(Token::RBrack(_)))
                                // [>+>+<<-]
                                {
                                    out.push(Token::Dup(*there, *mul, *back, *mul2));

                                    si += 8;
                                    continue;
                                }
                            }
                        }
                    }

                // Sub
                } else if matches!(next, Some(Token::Dec(1))) {
                    if let Some(Token::Goto(back)) = toks.get(si + 3) {
                        if *there == -back
                            && matches!(toks.get(si + 4), Some(Token::Dec(1)))
                            && matches!(toks.get(si + 5), Some(Token::RBrack(_)))
                        {
                            out.push(Token::Sub(*there));

                            si += 6;
                            continue;
                        }
                    }

                // Scan
                } else if matches!(next, Some(Token::RBrack(_))) {
                    out.push(Token::Scan(*there));

                    si += 3;
                    continue;
                }

            // End
            } else if let Some(Token::RBrack(_)) = next {
                out.push(Token::End);

                si += 2;
                continue;
            }
        }

        match tok {
            Token::LBrack(_) => lbracks.push(out.len()),
            Token::RBrack(_) => {
                let lb = lbracks
                    .pop()
                    .unwrap_or_else(|| err("invalid input", "unmatched closing bracket"));
                out[lb] = Token::LBrack(out.len());
                tok = Token::RBrack(lb);
            }

            Token::Goto(0) | Token::Inc(0) | Token::Dec(0) => {
                si += 1;
                continue;
            }

            _ => {}
        }

        out.push(tok);
        si += 1;
    }

    if !lbracks.is_empty() {
        err("invalid input", "unmatched opening bracket");
    }

    out
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Inc(i) => write!(f, "{}", "+".repeat(*i as usize)),
            Token::Dec(i) => write!(f, "{}", "-".repeat(*i as usize)),
            Token::Goto(i) => write!(f, "{}", left_right(*i)),
            Token::Set(i) => write!(f, "[-]{}", "+".repeat(*i as usize)),
            Token::LBrack(_) => write!(f, "["),
            Token::RBrack(_) => write!(f, "]"),
            Token::Out => write!(f, "."),

            Token::In => write!(f, ","),
            Token::Zero => write!(f, "[-]"),
            Token::Add(i, mul) => write!(
                f,
                "[{}-{}{}]",
                left_right(*i),
                left_right(-i),
                "+".repeat(*mul as usize)
            ),
            Token::Sub(i) => write!(f, "[{}-{}-]", left_right(*i), left_right(-i)),
            Token::Dup(i1, mul1, i2, mul2) => write!(
                f,
                "[-{}{}{}{}{}]",
                left_right(*i1),
                "+".repeat(*mul1 as usize),
                left_right(*i2),
                "+".repeat(*mul2 as usize),
                left_right(-(i1 + i2)),
            ),
            Token::Scan(i) => write!(f, "[{}]", left_right(*i)),

            Token::End => write!(f, "[]"),

            #[cfg(any(debug_assertions, feature = "debug"))]
            Token::Dump => write!(f, ";"),
        }
    }
}
