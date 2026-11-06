//! Shell arithmetic evaluation (`$(( ))`, `(( ))`, `let`, array subscripts).

pub trait ArithEnv {
    /// Get a variable's value (`sub` = raw subscript text for `name[sub]`).
    fn arith_get(&mut self, name: &str, sub: Option<&str>) -> Result<String, String>;
    fn arith_set(&mut self, name: &str, sub: Option<&str>, v: i64) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq)]
enum T {
    Num(i64),
    Ident(String, Option<String>),
    Op(&'static str),
}

const OPS: &[&str] = &[
    "<<=", ">>=", "**", "++", "--", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=",
    "^=", "|=", "+", "-", "*", "/", "%", "<", ">", "!", "~", "&", "^", "|", "?", ":", "=", ",", "(", ")",
];

pub fn parse_number(s: &str) -> Result<i64, String> {
    let s = s.trim();
    let (neg, body) = if let Some(r) = s.strip_prefix('-') {
        (true, r.trim_start())
    } else if let Some(r) = s.strip_prefix('+') {
        (false, r.trim_start())
    } else {
        (false, s)
    };
    let v = parse_unsigned(body)?;
    Ok(if neg { v.wrapping_neg() } else { v })
}

fn digit_val(c: char, base: u32) -> Option<u32> {
    let v = match c {
        '0'..='9' => c as u32 - '0' as u32,
        'a'..='z' => c as u32 - 'a' as u32 + 10,
        'A'..='Z' => {
            if base <= 36 {
                c as u32 - 'A' as u32 + 10
            } else {
                c as u32 - 'A' as u32 + 36
            }
        }
        '@' => 62,
        '_' => 63,
        _ => return None,
    };
    if v < base {
        Some(v)
    } else {
        None
    }
}

fn parse_unsigned(s: &str) -> Result<i64, String> {
    if s.is_empty() {
        return Err("operand expected".into());
    }
    let too_great = || format!("value too great for base (error token is \"{}\")", s);
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        let mut v: i64 = 0;
        if h.is_empty() {
            return Err(too_great());
        }
        for c in h.chars() {
            let d = digit_val(c, 16).ok_or_else(too_great)?;
            v = v.wrapping_mul(16).wrapping_add(d as i64);
        }
        return Ok(v);
    }
    if let Some(p) = s.find('#') {
        let base: u32 = s[..p].parse().map_err(|_| format!("invalid number (error token is \"{}\")", s))?;
        if !(2..=64).contains(&base) {
            return Err(format!("invalid arithmetic base (error token is \"{}\")", s));
        }
        let mut v: i64 = 0;
        let digits = &s[p + 1..];
        if digits.is_empty() {
            return Err(format!("invalid number (error token is \"{}\")", s));
        }
        for c in digits.chars() {
            let d = digit_val(c, base).ok_or_else(too_great)?;
            v = v.wrapping_mul(base as i64).wrapping_add(d as i64);
        }
        return Ok(v);
    }
    if s.len() > 1 && s.starts_with('0') {
        let mut v: i64 = 0;
        for c in s[1..].chars() {
            let d = digit_val(c, 8).ok_or_else(too_great)?;
            v = v.wrapping_mul(8).wrapping_add(d as i64);
        }
        return Ok(v);
    }
    let mut v: i64 = 0;
    for c in s.chars() {
        let d = match c.to_digit(10) {
            Some(d) => d,
            None => return Err(too_great()),
        };
        v = v.wrapping_mul(10).wrapping_add(d as i64);
    }
    Ok(v)
}

fn tokenize(s: &str) -> Result<(Vec<T>, Vec<usize>), String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    let mut starts: Vec<usize> = Vec::new();
    let mut last_start = 0;
    while i < c.len() {
        while starts.len() < out.len() {
            starts.push(last_start);
        }
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        last_start = i;
        if ch.is_ascii_digit() {
            let mut t = String::new();
            while i < c.len() && (c[i].is_ascii_alphanumeric() || c[i] == '#' || c[i] == '@' || c[i] == '_') {
                t.push(c[i]);
                i += 1;
            }
            out.push(T::Num(parse_unsigned(&t)?));
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut t = String::new();
            while i < c.len() && (c[i].is_ascii_alphanumeric() || c[i] == '_') {
                t.push(c[i]);
                i += 1;
            }
            let mut sub = None;
            let mut j = i;
            while j < c.len() && c[j] == ' ' {
                j += 1;
            }
            if j < c.len() && c[j] == '[' {
                let mut depth = 0;
                let mut k = j;
                let mut st = String::new();
                loop {
                    if k >= c.len() {
                        return Err("bad array subscript".into());
                    }
                    let x = c[k];
                    if x == '[' {
                        depth += 1;
                        if depth > 1 {
                            st.push(x);
                        }
                    } else if x == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        st.push(x);
                    } else {
                        st.push(x);
                    }
                    k += 1;
                }
                sub = Some(st);
                i = k + 1;
            }
            out.push(T::Ident(t, sub));
            continue;
        }
        let mut matched = false;
        // 3--3 / x)++1: after an operand, ++/-- that isn't a postfix on a variable is two signs
        if (ch == '+' || ch == '-') && c.get(i + 1) == Some(&ch) && matches!(out.last(), Some(T::Num(_)) | Some(T::Op(")"))) {
            out.push(T::Op(if ch == '+' { "+" } else { "-" }));
            out.push(T::Op(if ch == '+' { "+" } else { "-" }));
            i += 2;
            continue;
        }
        for op in OPS {
            let n = op.len();
            if i + n <= c.len() && c[i..i + n].iter().collect::<String>() == *op {
                out.push(T::Op(op));
                i += n;
                matched = true;
                break;
            }
        }
        if !matched {
            let rest: String = c[i..].iter().collect();
            return Err(format!("arithmetic syntax error: invalid arithmetic operator (error token is \"{}\")", rest));
        }
    }
    while starts.len() < out.len() {
        starts.push(last_start);
    }
    Ok((out, starts))
}

struct Ev<'a> {
    toks: Vec<T>,
    pos: Vec<usize>,
    chars: Vec<char>,
    i: usize,
    env: &'a mut dyn ArithEnv,
    noeval: u32,
    depth: u32,
    src: String,
}

type R = Result<i64, String>;

impl<'a> Ev<'a> {
    fn peek(&self) -> Option<&T> {
        self.toks.get(self.i)
    }
    fn peek_op(&self) -> Option<&'static str> {
        match self.toks.get(self.i) {
            Some(T::Op(o)) => Some(o),
            _ => None,
        }
    }
    fn eat(&mut self, op: &str) -> bool {
        if self.peek_op() == Some(op) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn err_tok(&self) -> String {
        self.src_from(self.i)
    }
    fn src_from(&self, tok: usize) -> String {
        match self.pos.get(tok) {
            Some(&p) => self.chars[p.min(self.chars.len())..].iter().collect(),
            None => String::new(),
        }
    }

    fn var_value(&mut self, name: &str, sub: Option<&str>) -> R {
        if self.noeval > 0 {
            return Ok(0);
        }
        let s = self.env.arith_get(name, sub)?;
        let t = s.trim();
        if t.is_empty() {
            return Ok(0);
        }
        if let Ok(v) = parse_number(t) {
            return Ok(v);
        }
        if self.depth > 64 {
            return Err(format!("{}: expression recursion level exceeded", name));
        }
        eval_depth(t, self.env, self.depth + 1)
    }

    fn comma(&mut self) -> R {
        let mut v = self.assign()?;
        while self.eat(",") {
            v = self.assign()?;
        }
        Ok(v)
    }

    fn assign(&mut self) -> R {
        if let Some(T::Ident(name, sub)) = self.peek().cloned() {
            if let Some(T::Op(op)) = self.toks.get(self.i + 1).cloned() {
                if matches!(op, "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "<<=" | ">>=" | "&=" | "^=" | "|=") {
                    self.i += 2;
                    let rhs = self.assign()?;
                    if self.noeval > 0 {
                        return Ok(rhs);
                    }
                    let v = if op == "=" {
                        rhs
                    } else {
                        let cur = self.var_value(&name, sub.as_deref())?;
                        binop(&op[..op.len() - 1], cur, rhs)?
                    };
                    self.env.arith_set(&name, sub.as_deref(), v)?;
                    return Ok(v);
                }
            }
        }
        self.ternary()
    }

    fn ternary(&mut self) -> R {
        let c = self.lor()?;
        if self.eat("?") {
            if c != 0 {
                let a = self.assign()?;
                if !self.eat(":") {
                    return Err(format!("arithmetic syntax error: `:' expected for conditional expression (error token is \"{}\")", self.err_tok()));
                }
                self.noeval += 1;
                let _ = self.assign()?;
                self.noeval -= 1;
                Ok(a)
            } else {
                self.noeval += 1;
                let _ = self.assign()?;
                self.noeval -= 1;
                if !self.eat(":") {
                    return Err(format!("arithmetic syntax error: `:' expected for conditional expression (error token is \"{}\")", self.err_tok()));
                }
                self.assign()
            }
        } else {
            Ok(c)
        }
    }

    fn lor(&mut self) -> R {
        let mut l = self.land()?;
        while self.eat("||") {
            if l != 0 {
                self.noeval += 1;
                self.land()?;
                self.noeval -= 1;
                l = 1;
            } else {
                let r = self.land()?;
                l = (r != 0) as i64;
            }
        }
        Ok(l)
    }
    fn land(&mut self) -> R {
        let mut l = self.bor()?;
        while self.eat("&&") {
            if l == 0 {
                self.noeval += 1;
                self.bor()?;
                self.noeval -= 1;
            } else {
                let r = self.bor()?;
                l = (r != 0) as i64;
            }
        }
        Ok(l)
    }
    fn bor(&mut self) -> R {
        let mut l = self.bxor()?;
        while self.eat("|") {
            l |= self.bxor()?;
        }
        Ok(l)
    }
    fn bxor(&mut self) -> R {
        let mut l = self.band()?;
        while self.eat("^") {
            l ^= self.band()?;
        }
        Ok(l)
    }
    fn band(&mut self) -> R {
        let mut l = self.equality()?;
        while self.eat("&") {
            l &= self.equality()?;
        }
        Ok(l)
    }
    fn equality(&mut self) -> R {
        let mut l = self.relational()?;
        loop {
            if self.eat("==") {
                l = (l == self.relational()?) as i64;
            } else if self.eat("!=") {
                l = (l != self.relational()?) as i64;
            } else {
                return Ok(l);
            }
        }
    }
    fn relational(&mut self) -> R {
        let mut l = self.shift()?;
        loop {
            if self.eat("<=") {
                l = (l <= self.shift()?) as i64;
            } else if self.eat(">=") {
                l = (l >= self.shift()?) as i64;
            } else if self.eat("<") {
                l = (l < self.shift()?) as i64;
            } else if self.eat(">") {
                l = (l > self.shift()?) as i64;
            } else {
                return Ok(l);
            }
        }
    }
    fn shift(&mut self) -> R {
        let mut l = self.additive()?;
        loop {
            if self.eat("<<") {
                let r = self.additive()?;
                l = l.wrapping_shl(r as u32);
            } else if self.eat(">>") {
                let r = self.additive()?;
                l = l.wrapping_shr(r as u32);
            } else {
                return Ok(l);
            }
        }
    }
    fn additive(&mut self) -> R {
        let mut l = self.mult()?;
        loop {
            if self.eat("+") {
                l = l.wrapping_add(self.mult()?);
            } else if self.eat("-") {
                l = l.wrapping_sub(self.mult()?);
            } else {
                return Ok(l);
            }
        }
    }
    fn mult(&mut self) -> R {
        let mut l = self.power()?;
        loop {
            let op = if self.eat("*") {
                "*"
            } else if self.eat("/") {
                "/"
            } else if self.eat("%") {
                "%"
            } else {
                return Ok(l);
            };
            let didx = self.i;
            let r = self.power()?;
            if self.noeval > 0 && r == 0 && op != "*" {
                l = 0;
                continue;
            }
            l = binop(op, l, r).map_err(|e| format!("{} (error token is \"{}\")", e, self.src_from(didx)))?;
        }
    }
    fn err_tok_prev(&self, r: i64) -> String {
        let rest = self.err_tok();
        if rest.is_empty() {
            r.to_string()
        } else {
            format!("{} {}", r, rest)
        }
    }
    fn power(&mut self) -> R {
        let b = self.unary()?;
        if self.eat("**") {
            let e = self.power()?;
            if e < 0 {
                if self.noeval > 0 {
                    return Ok(0);
                }
                return Err("exponent less than 0".into());
            }
            return Ok(ipow(b, e));
        }
        Ok(b)
    }
    fn unary(&mut self) -> R {
        if self.eat("!") {
            return Ok((self.unary()? == 0) as i64);
        }
        if self.eat("~") {
            return Ok(!self.unary()?);
        }
        if self.eat("-") {
            return Ok(self.unary()?.wrapping_neg());
        }
        if self.eat("+") {
            return self.unary();
        }
        if self.peek_op() == Some("++") || self.peek_op() == Some("--") {
            let inc = self.peek_op() == Some("++");
            if let Some(T::Ident(name, sub)) = self.toks.get(self.i + 1).cloned() {
                self.i += 2;
                let v = self.var_value(&name, sub.as_deref())?;
                let nv = if inc { v.wrapping_add(1) } else { v.wrapping_sub(1) };
                if self.noeval == 0 {
                    self.env.arith_set(&name, sub.as_deref(), nv)?;
                }
                return Ok(nv);
            }
            // `--5` is double negation
            self.i += 1;
            let v = self.unary()?;
            return Ok(v);
        }
        self.postfix()
    }
    fn postfix(&mut self) -> R {
        match self.peek().cloned() {
            Some(T::Num(n)) => {
                self.i += 1;
                Ok(n)
            }
            Some(T::Ident(name, sub)) => {
                self.i += 1;
                if self.peek_op() == Some("++") || self.peek_op() == Some("--") {
                    let inc = self.peek_op() == Some("++");
                    self.i += 1;
                    let v = self.var_value(&name, sub.as_deref())?;
                    if self.noeval == 0 {
                        let nv = if inc { v.wrapping_add(1) } else { v.wrapping_sub(1) };
                        self.env.arith_set(&name, sub.as_deref(), nv)?;
                    }
                    return Ok(v);
                }
                self.var_value(&name, sub.as_deref())
            }
            Some(T::Op("(")) => {
                self.i += 1;
                let v = self.comma()?;
                if !self.eat(")") {
                    return Err(format!("arithmetic syntax error: missing `)' (error token is \"{}\")", self.err_tok()));
                }
                Ok(v)
            }
            _ => {
                let tok = if self.i >= self.toks.len() && self.i > 0 { self.src_from(self.i - 1) } else { self.err_tok() };
                Err(format!("arithmetic syntax error: operand expected (error token is \"{}\")", tok))
            }
        }
    }
    fn src_tail(&self) -> String {
        self.src.trim().to_string()
    }
}

fn ipow(mut b: i64, mut e: i64) -> i64 {
    let mut r: i64 = 1;
    while e > 0 {
        if e & 1 == 1 {
            r = r.wrapping_mul(b);
        }
        b = b.wrapping_mul(b);
        e >>= 1;
    }
    r
}

fn binop(op: &str, a: i64, b: i64) -> R {
    Ok(match op {
        "+" => a.wrapping_add(b),
        "-" => a.wrapping_sub(b),
        "*" => a.wrapping_mul(b),
        "/" => {
            if b == 0 {
                return Err("division by 0".into());
            }
            a.wrapping_div(b)
        }
        "%" => {
            if b == 0 {
                return Err("division by 0".into());
            }
            a.wrapping_rem(b)
        }
        "<<" => a.wrapping_shl(b as u32),
        ">>" => a.wrapping_shr(b as u32),
        "&" => a & b,
        "^" => a ^ b,
        "|" => a | b,
        _ => return Err(format!("bad operator {}", op)),
    })
}

pub fn eval(expr: &str, env: &mut dyn ArithEnv) -> R {
    eval_depth(expr, env, 0)
}

fn eval_depth(expr: &str, env: &mut dyn ArithEnv, depth: u32) -> R {
    if expr.trim().is_empty() {
        return Ok(0);
    }
    let (toks, pos) = tokenize(expr)?;
    let mut ev = Ev { toks, pos, chars: expr.chars().collect(), i: 0, env, noeval: 0, depth, src: expr.to_string() };
    let v = ev.comma()?;
    if ev.i < ev.toks.len() {
        return Err(format!("arithmetic syntax error in expression (error token is \"{}\")", ev.err_tok()));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    struct E(HashMap<String, String>);
    impl ArithEnv for E {
        fn arith_get(&mut self, n: &str, _s: Option<&str>) -> Result<String, String> {
            Ok(self.0.get(n).cloned().unwrap_or_default())
        }
        fn arith_set(&mut self, n: &str, _s: Option<&str>, v: i64) -> Result<(), String> {
            self.0.insert(n.into(), v.to_string());
            Ok(())
        }
    }
    #[test]
    fn ops() {
        let mut e = E(HashMap::new());
        assert_eq!(eval("1 + 2 * 3", &mut e).unwrap(), 7);
        assert_eq!(eval("(1 + 2) * 3", &mut e).unwrap(), 9);
        assert_eq!(eval("2 ** 10", &mut e).unwrap(), 1024);
        assert_eq!(eval("-2 ** 2", &mut e).unwrap(), 4);
        assert_eq!(eval("x = 5, x += 2, x", &mut e).unwrap(), 7);
        assert_eq!(eval("x++ + ++x", &mut e).unwrap(), 7 + 9);
        assert_eq!(eval("0x1f + 010 + 2#101", &mut e).unwrap(), 31 + 8 + 5);
        assert_eq!(eval("1 ? 2 : 3", &mut e).unwrap(), 2);
        assert_eq!(eval("0 && 1/0", &mut e).unwrap(), 0);
        assert!(eval("1/0", &mut e).is_err());
        assert_eq!(eval("7 % 3 == 1 && 5 > 3", &mut e).unwrap(), 1);
        assert_eq!(eval("~0", &mut e).unwrap(), -1);
        assert_eq!(eval("1 << 4 | 1", &mut e).unwrap(), 17);
    }
}
