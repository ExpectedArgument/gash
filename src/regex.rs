//! Small POSIX-ERE-style regex engine for `[[ str =~ re ]]`.
//! Backtracking VM with a visited-state bitmap (linear-ish time, leftmost-first semantics).

#[derive(Debug, Clone)]
enum CItem {
    Ch(char),
    Range(char, char),
    Named(String),
}

#[derive(Debug, Clone)]
enum Node {
    Char(char),
    Any,
    Class(bool, Vec<CItem>),
    Bol,
    Eol,
    WordB,
    NotWordB,
    WordStart,
    WordEnd,
    Group(Box<Node>, usize),
    Concat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat(Box<Node>, u32, Option<u32>),
}

#[derive(Debug, Clone)]
enum Inst {
    Char(char),
    Any,
    Class(usize),
    Split(usize, usize),
    Jmp(usize),
    Save(usize),
    Bol,
    Eol,
    WordB,
    NotWordB,
    WordStart,
    WordEnd,
    Match,
}

pub struct Regex {
    prog: Vec<Inst>,
    classes: Vec<(bool, Vec<CItem>)>,
    ngroups: usize,
    nocase: bool,
    anchored: bool,
}

struct P<'a> {
    c: &'a [char],
    i: usize,
    ngroups: usize,
}

impl<'a> P<'a> {
    fn alt(&mut self) -> Result<Node, String> {
        let mut alts = vec![self.concat()?];
        while self.i < self.c.len() && self.c[self.i] == '|' {
            self.i += 1;
            alts.push(self.concat()?);
        }
        Ok(if alts.len() == 1 { alts.pop().unwrap() } else { Node::Alt(alts) })
    }
    fn concat(&mut self) -> Result<Node, String> {
        let mut v = Vec::new();
        while self.i < self.c.len() && self.c[self.i] != '|' && self.c[self.i] != ')' {
            let a = self.atom()?;
            let a = self.quant(a)?;
            v.push(a);
        }
        Ok(Node::Concat(v))
    }
    fn quant(&mut self, mut a: Node) -> Result<Node, String> {
        loop {
            if self.i >= self.c.len() {
                return Ok(a);
            }
            match self.c[self.i] {
                '*' => {
                    self.i += 1;
                    a = Node::Repeat(Box::new(a), 0, None);
                }
                '+' => {
                    self.i += 1;
                    a = Node::Repeat(Box::new(a), 1, None);
                }
                '?' => {
                    self.i += 1;
                    a = Node::Repeat(Box::new(a), 0, Some(1));
                }
                '{' => {
                    // interval?
                    let save = self.i;
                    self.i += 1;
                    let mut n1 = String::new();
                    while self.i < self.c.len() && self.c[self.i].is_ascii_digit() {
                        n1.push(self.c[self.i]);
                        self.i += 1;
                    }
                    let mut n2: Option<String> = None;
                    if self.i < self.c.len() && self.c[self.i] == ',' {
                        self.i += 1;
                        let mut s = String::new();
                        while self.i < self.c.len() && self.c[self.i].is_ascii_digit() {
                            s.push(self.c[self.i]);
                            self.i += 1;
                        }
                        n2 = Some(s);
                    }
                    if self.i < self.c.len() && self.c[self.i] == '}' && !n1.is_empty() {
                        self.i += 1;
                        let min: u32 = n1.parse().map_err(|_| "bad interval")?;
                        let max = match n2 {
                            None => Some(min),
                            Some(s) if s.is_empty() => None,
                            Some(s) => Some(s.parse().map_err(|_| "bad interval")?),
                        };
                        if min > 1000 || max.map(|m| m > 1000 || m < min).unwrap_or(false) {
                            return Err("invalid repetition count".into());
                        }
                        a = Node::Repeat(Box::new(a), min, max);
                    } else {
                        self.i = save;
                        return Ok(a);
                    }
                }
                _ => return Ok(a),
            }
        }
    }
    fn atom(&mut self) -> Result<Node, String> {
        let ch = self.c[self.i];
        self.i += 1;
        Ok(match ch {
            '(' => {
                self.ngroups += 1;
                let g = self.ngroups;
                let inner = if self.i < self.c.len() && self.c[self.i] == ')' { Node::Concat(vec![]) } else { self.alt()? };
                if self.i >= self.c.len() || self.c[self.i] != ')' {
                    return Err("unmatched ( or \\(".into());
                }
                self.i += 1;
                Node::Group(Box::new(inner), g)
            }
            '.' => Node::Any,
            '^' => Node::Bol,
            '$' => Node::Eol,
            '[' => self.class()?,
            '\\' => {
                if self.i >= self.c.len() {
                    return Err("trailing backslash".into());
                }
                let e = self.c[self.i];
                self.i += 1;
                match e {
                    'w' => Node::Class(false, vec![CItem::Named("word".into())]),
                    'W' => Node::Class(true, vec![CItem::Named("word".into())]),
                    's' => Node::Class(false, vec![CItem::Named("space".into())]),
                    'S' => Node::Class(true, vec![CItem::Named("space".into())]),
                    'd' => Node::Class(false, vec![CItem::Named("digit".into())]),
                    'D' => Node::Class(true, vec![CItem::Named("digit".into())]),
                    'b' => Node::WordB,
                    'B' => Node::NotWordB,
                    '<' => Node::WordStart,
                    '>' => Node::WordEnd,
                    'n' => Node::Char('\n'),
                    't' => Node::Char('\t'),
                    c => Node::Char(c),
                }
            }
            '*' | '+' | '?' => {
                // leading quantifier: literal in ERE
                Node::Char(ch)
            }
            c => Node::Char(c),
        })
    }
    fn class(&mut self) -> Result<Node, String> {
        let mut neg = false;
        if self.i < self.c.len() && self.c[self.i] == '^' {
            neg = true;
            self.i += 1;
        }
        let mut items = Vec::new();
        let mut first = true;
        loop {
            if self.i >= self.c.len() {
                return Err("unmatched [".into());
            }
            let ch = self.c[self.i];
            if ch == ']' && !first {
                self.i += 1;
                break;
            }
            first = false;
            if ch == '[' && self.i + 1 < self.c.len() && matches!(self.c[self.i + 1], ':' | '.' | '=') {
                let kind = self.c[self.i + 1];
                let mut j = self.i + 2;
                let mut name = String::new();
                while j + 1 < self.c.len() && !(self.c[j] == kind && self.c[j + 1] == ']') {
                    name.push(self.c[j]);
                    j += 1;
                }
                if j + 1 >= self.c.len() {
                    return Err("unmatched [".into());
                }
                self.i = j + 2;
                if kind == ':' {
                    items.push(CItem::Named(name));
                } else {
                    for c in name.chars() {
                        items.push(CItem::Ch(c));
                    }
                }
                continue;
            }
            self.i += 1;
            if self.i + 1 < self.c.len() && self.c[self.i] == '-' && self.c[self.i + 1] != ']' {
                let hi = self.c[self.i + 1];
                self.i += 2;
                items.push(CItem::Range(ch, hi));
            } else {
                items.push(CItem::Ch(ch));
            }
        }
        Ok(Node::Class(neg, items))
    }
}

struct Compiler {
    prog: Vec<Inst>,
    classes: Vec<(bool, Vec<CItem>)>,
}

impl Compiler {
    fn emit(&mut self, i: Inst) -> usize {
        self.prog.push(i);
        self.prog.len() - 1
    }
    fn node(&mut self, n: &Node) {
        match n {
            Node::Char(c) => {
                self.emit(Inst::Char(*c));
            }
            Node::Any => {
                self.emit(Inst::Any);
            }
            Node::Class(neg, items) => {
                self.classes.push((*neg, items.clone()));
                let k = self.classes.len() - 1;
                self.emit(Inst::Class(k));
            }
            Node::Bol => {
                self.emit(Inst::Bol);
            }
            Node::Eol => {
                self.emit(Inst::Eol);
            }
            Node::WordB => {
                self.emit(Inst::WordB);
            }
            Node::NotWordB => {
                self.emit(Inst::NotWordB);
            }
            Node::WordStart => {
                self.emit(Inst::WordStart);
            }
            Node::WordEnd => {
                self.emit(Inst::WordEnd);
            }
            Node::Group(inner, g) => {
                self.emit(Inst::Save(g * 2));
                self.node(inner);
                self.emit(Inst::Save(g * 2 + 1));
            }
            Node::Concat(v) => {
                for x in v {
                    self.node(x);
                }
            }
            Node::Alt(alts) => {
                let mut jumps = Vec::new();
                for (k, a) in alts.iter().enumerate() {
                    if k + 1 < alts.len() {
                        let split = self.emit(Inst::Split(0, 0));
                        self.node(a);
                        jumps.push(self.emit(Inst::Jmp(0)));
                        let next = self.prog.len();
                        self.prog[split] = Inst::Split(split + 1, next);
                    } else {
                        self.node(a);
                    }
                }
                let end = self.prog.len();
                for j in jumps {
                    self.prog[j] = Inst::Jmp(end);
                }
            }
            Node::Repeat(inner, min, max) => {
                for _ in 0..*min {
                    self.node(inner);
                }
                match max {
                    None => {
                        let split = self.emit(Inst::Split(0, 0));
                        self.node(inner);
                        self.emit(Inst::Jmp(split));
                        let end = self.prog.len();
                        self.prog[split] = Inst::Split(split + 1, end);
                    }
                    Some(mx) => {
                        let mut splits = Vec::new();
                        for _ in *min..*mx {
                            splits.push(self.emit(Inst::Split(0, 0)));
                            self.node(inner);
                        }
                        let end = self.prog.len();
                        for s in splits {
                            self.prog[s] = Inst::Split(s + 1, end);
                        }
                    }
                }
            }
        }
    }
}

fn named(name: &str, c: char) -> bool {
    match name {
        "alpha" => c.is_alphabetic(),
        "digit" => c.is_ascii_digit(),
        "alnum" => c.is_alphanumeric(),
        "upper" => c.is_uppercase(),
        "lower" => c.is_lowercase(),
        "space" => c.is_whitespace(),
        "blank" => c == ' ' || c == '\t',
        "punct" => c.is_ascii_punctuation(),
        "print" => !c.is_control(),
        "graph" => !c.is_control() && !c.is_whitespace(),
        "cntrl" => c.is_control(),
        "xdigit" => c.is_ascii_hexdigit(),
        "word" => c.is_alphanumeric() || c == '_',
        _ => false,
    }
}

fn is_word(c: Option<&char>) -> bool {
    c.map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false)
}

impl Regex {
    pub fn new(re: &str, nocase: bool) -> Result<Regex, String> {
        let chars: Vec<char> = re.chars().collect();
        let mut p = P { c: &chars, i: 0, ngroups: 0 };
        let node = p.alt()?;
        if p.i < chars.len() {
            return Err("unmatched ( or \\(".into());
        }
        let anchored = matches!(&node, Node::Concat(v) if matches!(v.first(), Some(Node::Bol)));
        let mut c = Compiler { prog: Vec::new(), classes: Vec::new() };
        c.emit(Inst::Save(0));
        c.node(&node);
        c.emit(Inst::Save(1));
        c.emit(Inst::Match);
        Ok(Regex { prog: c.prog, classes: c.classes, ngroups: p.ngroups, nocase, anchored })
    }

    fn ceq(&self, a: char, b: char) -> bool {
        a == b || (self.nocase && a.to_lowercase().eq(b.to_lowercase()))
    }

    fn class_hit(&self, k: usize, c: char) -> bool {
        let (neg, items) = &self.classes[k];
        let mut hit = false;
        for it in items {
            let h = match it {
                CItem::Ch(x) => self.ceq(*x, c),
                CItem::Range(lo, hi) => {
                    (*lo <= c && c <= *hi)
                        || (self.nocase && {
                            let l = c.to_ascii_lowercase();
                            let u = c.to_ascii_uppercase();
                            (*lo <= l && l <= *hi) || (*lo <= u && u <= *hi)
                        })
                }
                CItem::Named(n) => named(n, c),
            };
            if h {
                hit = true;
                break;
            }
        }
        hit != *neg
    }

    /// Search; returns capture groups (0 = whole match) as optional char ranges, mapped to strings.
    pub fn captures(&self, s: &str) -> Option<Vec<Option<String>>> {
        let input: Vec<char> = s.chars().collect();
        let n = input.len();
        let np = self.prog.len();
        let mut visited = vec![0u64; (np * (n + 1)).div_ceil(64)];
        let nslots = (self.ngroups + 1) * 2;
        enum Job {
            Try(usize, usize),
            Restore(usize, Option<usize>),
        }
        let last_start = if self.anchored { 0 } else { n };
        for start in 0..=last_start {
            let mut caps: Vec<Option<usize>> = vec![None; nslots];
            let mut stack = vec![Job::Try(0, start)];
            while let Some(job) = stack.pop() {
                match job {
                    Job::Restore(slot, v) => caps[slot] = v,
                    Job::Try(mut pc, mut pos) => loop {
                        let key = pc * (n + 1) + pos;
                        if visited[key / 64] & (1 << (key % 64)) != 0 {
                            break;
                        }
                        visited[key / 64] |= 1 << (key % 64);
                        match &self.prog[pc] {
                            Inst::Char(c) => {
                                if pos < n && self.ceq(*c, input[pos]) {
                                    pc += 1;
                                    pos += 1;
                                } else {
                                    break;
                                }
                            }
                            Inst::Any => {
                                if pos < n {
                                    pc += 1;
                                    pos += 1;
                                } else {
                                    break;
                                }
                            }
                            Inst::Class(k) => {
                                if pos < n && self.class_hit(*k, input[pos]) {
                                    pc += 1;
                                    pos += 1;
                                } else {
                                    break;
                                }
                            }
                            Inst::Split(a, b) => {
                                stack.push(Job::Try(*b, pos));
                                pc = *a;
                            }
                            Inst::Jmp(a) => pc = *a,
                            Inst::Save(s) => {
                                stack.push(Job::Restore(*s, caps[*s]));
                                caps[*s] = Some(pos);
                                pc += 1;
                            }
                            Inst::Bol => {
                                if pos == 0 {
                                    pc += 1
                                } else {
                                    break;
                                }
                            }
                            Inst::Eol => {
                                if pos == n {
                                    pc += 1
                                } else {
                                    break;
                                }
                            }
                            Inst::WordB | Inst::NotWordB => {
                                let a = pos > 0 && is_word(input.get(pos - 1));
                                let b = is_word(input.get(pos));
                                let atb = a != b;
                                if atb == matches!(self.prog[pc], Inst::WordB) {
                                    pc += 1
                                } else {
                                    break;
                                }
                            }
                            Inst::WordStart => {
                                let a = pos > 0 && is_word(input.get(pos - 1));
                                if !a && is_word(input.get(pos)) {
                                    pc += 1
                                } else {
                                    break;
                                }
                            }
                            Inst::WordEnd => {
                                let a = pos > 0 && is_word(input.get(pos - 1));
                                if a && !is_word(input.get(pos)) {
                                    pc += 1
                                } else {
                                    break;
                                }
                            }
                            Inst::Match => {
                                let mut out = Vec::with_capacity(self.ngroups + 1);
                                for g in 0..=self.ngroups {
                                    match (caps[g * 2], caps[g * 2 + 1]) {
                                        (Some(a), Some(b)) if a <= b => out.push(Some(input[a..b].iter().collect())),
                                        _ => out.push(None),
                                    }
                                }
                                return Some(out);
                            }
                        }
                    },
                }
            }
        }
        None
    }

    pub fn is_match(&self, s: &str) -> bool {
        self.captures(s).is_some()
    }
}

/// Escape regex metacharacters (for quoted parts of `=~` operands).
pub fn escape(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        if "\\.[]()*+?{}|^$".contains(c) {
            o.push('\\');
        }
        o.push(c);
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let r = Regex::new("^([a-z]+)-([0-9]{2,3})$", false).unwrap();
        let c = r.captures("abc-123").unwrap();
        assert_eq!(c[1].as_deref(), Some("abc"));
        assert_eq!(c[2].as_deref(), Some("123"));
        assert!(r.captures("abc-1").is_none());
        let r = Regex::new("b+", false).unwrap();
        assert_eq!(r.captures("aabbbc").unwrap()[0].as_deref(), Some("bbb"));
        let r = Regex::new("(a|ab)(c|bcd)", false).unwrap();
        assert!(r.is_match("abcd"));
        let r = Regex::new("[[:digit:]]+\\.[0-9]*", false).unwrap();
        assert!(r.is_match("v12.5"));
        let r = Regex::new("(a*)*b", false).unwrap();
        assert!(!r.is_match(&"a".repeat(200)));
    }
}
