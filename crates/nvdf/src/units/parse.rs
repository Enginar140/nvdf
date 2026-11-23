//! Recursive-descent parser for quantity strings.
//!
//! Grammar (whitespace allowed between tokens):
//!
//! ```text
//! quantity := number [unit]
//! number   := ['+'|'-'] digits ['.' digits] [('e'|'E') ['+'|'-'] digits]
//! unit     := affine | expr            ; affine units (degC) only standalone
//! expr     := term (('*'|'/') term)*   ; left-to-right
//! term     := atom ['^' ['+'|'-'] digits]
//! atom     := symbol | 'sqrt' '(' expr ')' | '(' expr ')' | '1'   ; '1' only as in "1/s"
//! ```

use super::{Dimension, UnitError, table};

/// Result of parsing: SI value and the unit's dimension (`None` for a plain number).
pub(super) struct Parsed {
    pub value: f64,
    pub unit: Option<Dimension>,
}

pub(super) fn parse(input: &str) -> Result<Parsed, UnitError> {
    let mut p = Parser {
        input,
        chars: input.chars().collect(),
        pos: 0,
    };
    p.skip_ws();
    if p.peek().is_none() {
        return Err(UnitError::Empty);
    }
    let number = p.number()?;
    p.skip_ws();
    if p.peek().is_none() {
        return Ok(Parsed {
            value: number,
            unit: None,
        });
    }

    let rest: String = p.chars[p.pos..].iter().collect();
    if let Some(offset) = table::affine_offset(rest.trim_end()) {
        return Ok(Parsed {
            value: number + offset,
            unit: Some(Dimension::TEMPERATURE),
        });
    }

    let (factor, dim) = p.expr()?;
    p.skip_ws();
    if let Some(c) = p.peek() {
        let message = if is_symbol_char(c) || c == '(' {
            format!("unexpected '{c}'; use '*' or '/' between units (e.g. \"N*m\")")
        } else {
            format!("unexpected '{c}'")
        };
        return Err(p.syntax(message));
    }
    let value = number * factor;
    if !value.is_finite() {
        return Err(UnitError::OutOfRange {
            text: input.trim().to_owned(),
        });
    }
    Ok(Parsed {
        value,
        unit: Some(dim),
    })
}

fn is_symbol_char(c: char) -> bool {
    c.is_alphabetic() || c == '%' || c == '\u{b0}'
}

struct Parser<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, c: char) -> bool {
        let hit = self.peek() == Some(c);
        if hit {
            self.pos += 1;
        }
        hit
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    fn digits(&mut self) -> usize {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        self.pos - start
    }

    fn syntax(&self, message: impl Into<String>) -> UnitError {
        UnitError::Syntax {
            column: self.pos + 1,
            message: message.into(),
        }
    }

    fn overflow(&self) -> UnitError {
        self.syntax("unit exponent out of range")
    }

    fn number(&mut self) -> Result<f64, UnitError> {
        let bad = || UnitError::BadNumber {
            text: self.input.trim().to_owned(),
        };
        let start = self.pos;
        if !self.eat('+') {
            self.eat('-');
        }
        let mut mantissa = self.digits();
        if self.eat('.') {
            mantissa += self.digits();
        }
        if mantissa == 0 {
            return Err(bad());
        }
        if self.eat('e') || self.eat('E') {
            if !self.eat('+') {
                self.eat('-');
            }
            if self.digits() == 0 {
                return Err(bad());
            }
        }
        if matches!(self.peek(), Some('.' | '0'..='9')) {
            return Err(bad());
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let value: f64 = text.parse().map_err(|_| bad())?;
        if !value.is_finite() {
            return Err(UnitError::OutOfRange {
                text: self.input.trim().to_owned(),
            });
        }
        Ok(value)
    }

    fn expr(&mut self) -> Result<(f64, Dimension), UnitError> {
        let (mut value, mut dim) = self.term(true)?;
        loop {
            self.skip_ws();
            let divide = match self.peek() {
                Some('*') => false,
                Some('/') => true,
                _ => return Ok((value, dim)),
            };
            self.pos += 1;
            let (v, d) = self.term(false)?;
            let combined = if divide {
                value /= v;
                dim.checked_div(d)
            } else {
                value *= v;
                dim.checked_mul(d)
            };
            dim = combined.ok_or_else(|| self.overflow())?;
        }
    }

    fn term(&mut self, first: bool) -> Result<(f64, Dimension), UnitError> {
        self.skip_ws();
        let (value, dim) = self.atom(first)?;
        self.skip_ws();
        if !self.eat('^') {
            return Ok((value, dim));
        }
        self.skip_ws();
        let start = self.pos;
        if !self.eat('+') {
            self.eat('-');
        }
        if self.digits() == 0 {
            return Err(self.syntax("expected an integer exponent after '^' (e.g. \"m/s^2\")"));
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let n: i32 = text.parse().map_err(|_| self.overflow())?;
        let dim = dim.checked_powi(n).ok_or_else(|| self.overflow())?;
        Ok((value.powi(n), dim))
    }

    fn atom(&mut self, first: bool) -> Result<(f64, Dimension), UnitError> {
        match self.peek() {
            None => Err(self.syntax("expected a unit")),
            Some('(') => {
                self.pos += 1;
                self.group()
            }
            Some('1') if first => {
                self.pos += 1;
                self.skip_ws();
                if self.peek() == Some('/') {
                    Ok((1.0, Dimension::DIMENSIONLESS))
                } else {
                    Err(self.syntax("expected '/' after '1' (e.g. \"1/s\")"))
                }
            }
            Some(c) if is_symbol_char(c) => self.symbol(),
            Some(c) => Err(self.syntax(format!("expected a unit, found '{c}'"))),
        }
    }

    /// Parses `expr ')'` after an opening parenthesis.
    fn group(&mut self) -> Result<(f64, Dimension), UnitError> {
        let inner = self.expr()?;
        self.skip_ws();
        if self.eat(')') {
            Ok(inner)
        } else {
            Err(self.syntax("expected ')'"))
        }
    }

    fn symbol(&mut self) -> Result<(f64, Dimension), UnitError> {
        let start = self.pos;
        while self.peek().is_some_and(is_symbol_char) {
            self.pos += 1;
        }
        let symbol: String = self.chars[start..self.pos].iter().collect();
        if symbol == "sqrt" {
            self.skip_ws();
            if !self.eat('(') {
                return Err(self.syntax("expected '(' after \"sqrt\" (e.g. \"sqrt(Hz)\")"));
            }
            let (value, dim) = self.group()?;
            let dim = dim.checked_sqrt().ok_or_else(|| {
                self.syntax("sqrt() of a unit that already has a half exponent is not supported")
            })?;
            return Ok((value.sqrt(), dim));
        }
        if let Some(def) = table::lookup(&symbol) {
            return Ok((def.factor, def.dimension));
        }
        if table::affine_offset(&symbol).is_some() {
            return Err(UnitError::AffineMisuse { unit: symbol });
        }
        let suggestion = table::suggest(&symbol);
        Err(UnitError::UnknownUnit {
            unit: symbol,
            suggestion,
        })
    }
}
