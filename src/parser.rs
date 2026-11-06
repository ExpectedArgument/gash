//! Hand-written recursive-descent parser for bash syntax.

use crate::ast::*;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub enum ParseError {
    /// Input ended in the middle of a construct; more input is needed.
    Incomplete,
    Syntax { msg: String, line: usize },
}

pub type PResult<T> = Result<T, ParseError>;

struct PendingHeredoc {
    cell: Arc<OnceLock<HereDocBody>>,
    delim: String,
    strip: bool,
    quoted: bool,
}

#[derive(Clone, Copy, Default)]
struct Ctx {
    case_pat: bool,
    regex: bool,
    assign: bool,
    cond: bool,
}

#[derive(Clone, Copy)]
struct Mark {
    pos: usize,
    line: usize,
}

const KEYWORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "do", "done", "case", "esac", "while", "until", "for", "select",
    "function", "in", "time", "{", "}", "!", "[[", "]]", "coproc",
];

const DECL_BUILTINS: &[&str] = &["declare", "typeset", "local", "export", "readonly"];

pub struct Parser {
    s: Vec<char>,
    pos: usize,
    pub line: usize,
    heredocs: Vec<PendingHeredoc>,
    pub aliases: Option<Arc<HashMap<String, String>>>,
    alias_active: Vec<(String, usize)>,
    alias_next: bool,
    /// When true, an unterminated here-document at EOF is an error that asks for more input.
    pub interactive: bool,
}

fn is_meta(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | ';' | '&' | '|' | '<' | '>' | '(' | ')')
}

fn is_delim(c: Option<char>) -> bool {
    match c {
        None => true,
        Some(c) => is_meta(c),
    }
}

pub fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
pub fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
pub fn is_valid_name(s: &str) -> bool {
    let mut it = s.chars();
    match it.next() {
        Some(c) if is_name_start(c) => it.all(is_name_char),
        _ => false,
    }
}

impl Parser {
    pub fn new(src: &str) -> Parser {
        Parser {
            s: src.chars().collect(),
            pos: 0,
            line: 1,
            heredocs: Vec::new(),
            aliases: None,
            alias_active: Vec::new(),
            alias_next: false,
            interactive: false,
        }
    }

    pub fn with_aliases(mut self, a: Option<Arc<HashMap<String, String>>>) -> Parser {
        self.aliases = a;
        self
    }

    // ---------- low level ----------

    fn peek(&self) -> Option<char> {
        self.s.get(self.pos).copied()
    }
    fn peek_at(&self, n: usize) -> Option<char> {
        self.s.get(self.pos + n).copied()
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.s.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }
    fn eof(&self) -> bool {
        self.pos >= self.s.len()
    }
    fn starts_with(&self, t: &str) -> bool {
        let mut i = self.pos;
        for c in t.chars() {
            if self.s.get(i) != Some(&c) {
                return false;
            }
            i += 1;
        }
        true
    }
    fn advance(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }
    fn mark(&self) -> Mark {
        Mark { pos: self.pos, line: self.line }
    }
    fn reset(&mut self, m: Mark) {
        self.pos = m.pos;
        self.line = m.line;
    }
    fn text(&self, a: usize, b: usize) -> String {
        self.s[a.min(self.s.len())..b.min(self.s.len())].iter().collect()
    }

    fn syntax<T>(&self, msg: impl Into<String>) -> PResult<T> {
        Err(ParseError::Syntax { msg: msg.into(), line: self.line })
    }

    fn unexpected<T>(&self) -> PResult<T> {
        match self.peek() {
            None => Err(ParseError::Incomplete),
            Some('\n') => self.syntax("syntax error near unexpected token `newline'"),
            Some(_) => {
                let tok = self.peek_token_text();
                self.syntax(format!("syntax error near unexpected token `{}'", tok))
            }
        }
    }

    fn peek_token_text(&self) -> String {
        for op in [";;&", ";;", ";&", "&&", "||", "|&", ">>", "<<", "&>", ">&", "<&", "(", ")", ";", "&", "|", "<", ">"] {
            if self.starts_with(op) {
                return op.to_string();
            }
        }
        let (w, _) = self.peek_word_raw();
        if w.is_empty() {
            self.peek().map(|c| c.to_string()).unwrap_or_default()
        } else {
            w
        }
    }

    /// Skip blanks, line continuations and comments (not newlines).
    fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.pos += 1;
                }
                Some('\\') if self.peek_at(1) == Some('\n') => {
                    self.bump();
                    self.bump();
                }
                Some('#') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn consume_newline(&mut self) -> PResult<()> {
        self.bump();
        if !self.heredocs.is_empty() {
            self.read_heredocs()?;
        }
        Ok(())
    }

    fn skip_newlines(&mut self) -> PResult<()> {
        loop {
            self.skip_ws();
            if self.peek() == Some('\n') {
                self.consume_newline()?;
            } else {
                return Ok(());
            }
        }
    }

    /// Read a run of plain word characters (no quoting/expansion chars).
    fn peek_word_raw(&self) -> (String, usize) {
        let mut i = self.pos;
        let mut w = String::new();
        while let Some(&c) = self.s.get(i) {
            if is_meta(c) || matches!(c, '\'' | '"' | '\\' | '$' | '`') {
                break;
            }
            w.push(c);
            i += 1;
        }
        (w, i)
    }

    fn peek_keyword(&self) -> Option<&'static str> {
        let (w, end) = self.peek_word_raw();
        if w.is_empty() {
            return None;
        }
        let next = self.s.get(end).copied();
        let kw = KEYWORDS.iter().find(|k| **k == w)?;
        match *kw {
            "!" | "[[" => {
                if matches!(next, None | Some(' ') | Some('\t') | Some('\n')) {
                    Some(kw)
                } else {
                    None
                }
            }
            _ => {
                if is_delim(next) {
                    Some(kw)
                } else {
                    None
                }
            }
        }
    }

    fn consume_keyword(&mut self, kw: &str) {
        self.advance(kw.chars().count());
    }

    fn expect_keyword(&mut self, kw: &str) -> PResult<()> {
        self.skip_newlines()?;
        if self.peek_keyword() == Some(kw) {
            self.consume_keyword(kw);
            Ok(())
        } else if self.eof() {
            Err(ParseError::Incomplete)
        } else {
            self.unexpected()
        }
    }

    // ---------- here-documents ----------

    fn read_heredocs(&mut self) -> PResult<()> {
        let pending = std::mem::take(&mut self.heredocs);
        for hd in pending {
            let mut body = String::new();
            loop {
                if self.eof() {
                    if self.interactive {
                        return Err(ParseError::Incomplete);
                    }
                    break;
                }
                let mut line = String::new();
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    line.push(c);
                    self.pos += 1;
                }
                let had_nl = self.peek() == Some('\n');
                if had_nl {
                    self.bump();
                }
                let line_cmp = line.strip_suffix('\r').unwrap_or(&line);
                let check = if hd.strip { line_cmp.trim_start_matches('\t') } else { line_cmp };
                if check == hd.delim {
                    break;
                }
                if hd.strip {
                    body.push_str(line.trim_start_matches('\t'));
                } else {
                    body.push_str(&line);
                }
                if had_nl {
                    body.push('\n');
                }
            }
            let word = if hd.quoted {
                Word { parts: vec![WordPart::SQuoted(body)] }
            } else {
                parse_heredoc_text(&body, self.aliases.clone())?
            };
            let _ = hd.cell.set(HereDocBody { quoted: hd.quoted, word });
        }
        Ok(())
    }

    // ---------- top level ----------

    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut items = Vec::new();
        while let Some(mut cmd) = self.parse_next()? {
            items.append(&mut cmd);
        }
        Ok(Program { items })
    }

    /// Parse the next complete command (one logical line). Returns None at EOF.
    pub fn parse_next(&mut self) -> PResult<Option<List>> {
        loop {
            self.skip_ws();
            match self.peek() {
                None => return Ok(None),
                Some('\n') => self.consume_newline()?,
                _ => break,
            }
        }
        let mut items = Vec::new();
        loop {
            let start = self.pos;
            let ao = self.parse_and_or()?;
            self.skip_ws();
            let mut bg = false;
            match self.peek() {
                Some('&') => {
                    self.bump();
                    bg = true;
                }
                Some(';') => {
                    if self.peek_at(1) == Some(';') || self.peek_at(1) == Some('&') {
                        return self.unexpected();
                    }
                    self.bump();
                }
                Some('\n') | None => {}
                Some(_) => return self.unexpected(),
            }
            let text = self.text(start, self.pos).trim().trim_end_matches([';', '&']).trim().to_string();
            items.push(ListItem { and_or: ao, background: bg, text });
            self.skip_ws();
            match self.peek() {
                None => break,
                Some('\n') => {
                    self.consume_newline()?;
                    break;
                }
                _ => {}
            }
        }
        Ok(Some(items))
    }

    fn at_list_end(&self, terms: &[&str]) -> bool {
        match self.peek() {
            None => true,
            Some(')') => terms.contains(&")"),
            Some(';') => {
                terms.contains(&";;") && (self.peek_at(1) == Some(';') || self.peek_at(1) == Some('&'))
            }
            _ => match self.peek_keyword() {
                Some(kw) => terms.contains(&kw),
                None => false,
            },
        }
    }

    fn parse_list(&mut self, terms: &[&str]) -> PResult<List> {
        let mut items = Vec::new();
        loop {
            self.skip_newlines()?;
            if self.at_list_end(terms) {
                break;
            }
            let start = self.pos;
            let ao = self.parse_and_or()?;
            self.skip_ws();
            let mut bg = false;
            let mut sep = false;
            match self.peek() {
                Some('&') => {
                    self.bump();
                    bg = true;
                    sep = true;
                }
                Some(';') if self.peek_at(1) != Some(';') && self.peek_at(1) != Some('&') => {
                    self.bump();
                    sep = true;
                }
                Some('\n') => {
                    self.consume_newline()?;
                    sep = true;
                }
                _ => {}
            }
            let text = self.text(start, self.pos).trim().trim_end_matches([';', '&']).trim().to_string();
            items.push(ListItem { and_or: ao, background: bg, text });
            if !sep {
                self.skip_ws();
                if !self.at_list_end(terms) {
                    return self.unexpected();
                }
            }
        }
        Ok(items)
    }

    fn parse_and_or(&mut self) -> PResult<AndOr> {
        let first = self.parse_pipeline()?;
        let mut rest = Vec::new();
        loop {
            self.skip_ws();
            if self.starts_with("&&") {
                self.advance(2);
                self.skip_newlines()?;
                rest.push((AndOrOp::And, self.parse_pipeline()?));
            } else if self.starts_with("||") {
                self.advance(2);
                self.skip_newlines()?;
                rest.push((AndOrOp::Or, self.parse_pipeline()?));
            } else {
                break;
            }
        }
        Ok(AndOr { first, rest })
    }

    fn parse_pipeline(&mut self) -> PResult<Pipeline> {
        self.skip_ws();
        let line = self.line;
        let mut time = false;
        let mut time_posix = false;
        let mut negate = false;
        loop {
            match self.peek_keyword() {
                Some("time") => {
                    self.consume_keyword("time");
                    time = true;
                    self.skip_ws();
                    if self.starts_with("-p") && is_delim(self.peek_at(2)) {
                        self.advance(2);
                        time_posix = true;
                    }
                }
                Some("!") => {
                    self.consume_keyword("!");
                    negate = !negate;
                }
                _ => break,
            }
            self.skip_ws();
        }
        let mut cmds = Vec::new();
        if time || negate {
            // `time` / `!` alone
            match self.peek() {
                None | Some('\n') | Some(';') | Some('&') | Some(')') => {
                    if time {
                        return Ok(Pipeline { negate, time, time_posix, cmds, line });
                    }
                }
                _ => {}
            }
        }
        loop {
            let c = self.parse_command()?;
            self.skip_ws();
            if self.starts_with("|&") {
                self.advance(2);
                cmds.push((c, true));
                self.skip_newlines()?;
                continue;
            }
            if self.peek() == Some('|') && self.peek_at(1) != Some('|') {
                self.bump();
                cmds.push((c, false));
                self.skip_newlines()?;
                continue;
            }
            cmds.push((c, false));
            break;
        }
        Ok(Pipeline { negate, time, time_posix, cmds, line })
    }

    fn try_alias(&mut self) -> bool {
        let Some(aliases) = self.aliases.clone() else { return false };
        let (w, end) = self.peek_word_raw();
        if w.is_empty() || !is_delim(self.s.get(end).copied()) {
            return false;
        }
        let pos = self.pos;
        self.alias_active.retain(|(_, e)| *e > pos);
        if self.alias_active.iter().any(|(n, _)| *n == w) {
            return false;
        }
        let Some(val) = aliases.get(&w) else { return false };
        let chars: Vec<char> = val.chars().collect();
        let n = chars.len();
        let delta = n as isize - (end - self.pos) as isize;
        self.s.splice(self.pos..end, chars);
        for e in self.alias_active.iter_mut() {
            e.1 = (e.1 as isize + delta) as usize;
        }
        self.alias_active.push((w, self.pos + n));
        self.alias_next = val.ends_with(' ') || val.ends_with('\t');
        true
    }

    fn parse_command(&mut self) -> PResult<Command> {
        self.skip_ws();
        if self.aliases.is_some() && self.try_alias() {
            return self.parse_command();
        }
        let line = self.line;
        match self.peek() {
            None => return Err(ParseError::Incomplete),
            Some('(') => {
                if self.peek_at(1) == Some('(') {
                    if let Some(w) = self.try_double_paren()? {
                        let redirs = self.parse_trailing_redirs()?;
                        return Ok(Command::Compound { kind: Box::new(CompoundKind::Arith(w)), redirs, line });
                    }
                }
                self.bump();
                let body = self.parse_list(&[")"])?;
                self.skip_newlines()?;
                match self.peek() {
                    Some(')') => {
                        self.bump();
                    }
                    None => return Err(ParseError::Incomplete),
                    _ => return self.unexpected(),
                }
                let redirs = self.parse_trailing_redirs()?;
                return Ok(Command::Compound { kind: Box::new(CompoundKind::Subshell(body)), redirs, line });
            }
            Some(')') | Some(';') | Some('|') | Some('\n') => return self.unexpected(),
            Some('&') if self.peek_at(1) != Some('>') => return self.unexpected(),
            _ => {}
        }
        if let Some(kw) = self.peek_keyword() {
            let kind = match kw {
                "if" => {
                    self.consume_keyword(kw);
                    Some(self.parse_if()?)
                }
                "while" | "until" => {
                    self.consume_keyword(kw);
                    let cond = self.parse_list(&["do"])?;
                    self.expect_keyword("do")?;
                    let body = self.parse_list(&["done"])?;
                    self.expect_keyword("done")?;
                    Some(CompoundKind::While { cond, body, until: kw == "until" })
                }
                "for" => {
                    self.consume_keyword(kw);
                    Some(self.parse_for(false)?)
                }
                "select" => {
                    self.consume_keyword(kw);
                    Some(self.parse_for(true)?)
                }
                "case" => {
                    self.consume_keyword(kw);
                    Some(self.parse_case()?)
                }
                "{" => {
                    self.consume_keyword(kw);
                    let body = self.parse_list(&["}"])?;
                    self.expect_keyword("}")?;
                    Some(CompoundKind::Group(body))
                }
                "[[" => {
                    self.consume_keyword(kw);
                    let e = self.parse_cond_or()?;
                    self.cond_skip()?;
                    if self.starts_with("]]") && is_delim(self.peek_at(2)) {
                        self.advance(2);
                    } else if self.eof() {
                        return Err(ParseError::Incomplete);
                    } else {
                        return self.unexpected();
                    }
                    Some(CompoundKind::Cond(e))
                }
                "function" => {
                    let start = self.pos;
                    self.consume_keyword(kw);
                    self.skip_ws();
                    let (name, end) = self.peek_word_raw();
                    if name.is_empty() {
                        return self.unexpected();
                    }
                    self.advance(end - self.pos);
                    self.skip_ws();
                    if self.peek() == Some('(') {
                        let m = self.mark();
                        self.bump();
                        self.skip_ws();
                        if self.peek() == Some(')') {
                            self.bump();
                        } else {
                            self.reset(m);
                        }
                    }
                    return self.parse_function_body(name, start, line);
                }
                "then" | "else" | "elif" | "fi" | "do" | "done" | "esac" | "}" => {
                    return self.syntax(format!("syntax error near unexpected token `{}'", kw));
                }
                _ => None,
            };
            if let Some(kind) = kind {
                let redirs = self.parse_trailing_redirs()?;
                return Ok(Command::Compound { kind: Box::new(kind), redirs, line });
            }
        }
        self.parse_simple_command()
    }

    fn parse_function_body(&mut self, name: String, start: usize, line: usize) -> PResult<Command> {
        self.skip_newlines()?;
        let body_start = self.pos;
        let body = self.parse_command()?;
        if !matches!(body, Command::Compound { .. }) {
            return self.syntax(format!("syntax error near unexpected token `{}'", name));
        }
        let _ = body_start;
        let source = self.text(start, self.pos);
        Ok(Command::FuncDef(Arc::new(FuncDef { name, body, source, file: String::new(), line })))
    }

    fn parse_trailing_redirs(&mut self) -> PResult<Vec<Redir>> {
        let mut v = Vec::new();
        loop {
            self.skip_ws();
            match self.try_parse_redir()? {
                Some(r) => v.push(r),
                None => break,
            }
        }
        Ok(v)
    }

    fn parse_if(&mut self) -> PResult<CompoundKind> {
        let mut conds = Vec::new();
        let mut else_body = None;
        let c = self.parse_list(&["then"])?;
        self.expect_keyword("then")?;
        let b = self.parse_list(&["elif", "else", "fi"])?;
        conds.push((c, b));
        loop {
            self.skip_newlines()?;
            match self.peek_keyword() {
                Some("elif") => {
                    self.consume_keyword("elif");
                    let c = self.parse_list(&["then"])?;
                    self.expect_keyword("then")?;
                    let b = self.parse_list(&["elif", "else", "fi"])?;
                    conds.push((c, b));
                }
                Some("else") => {
                    self.consume_keyword("else");
                    else_body = Some(self.parse_list(&["fi"])?);
                    self.expect_keyword("fi")?;
                    break;
                }
                Some("fi") => {
                    self.consume_keyword("fi");
                    break;
                }
                _ => {
                    if self.eof() {
                        return Err(ParseError::Incomplete);
                    }
                    return self.unexpected();
                }
            }
        }
        Ok(CompoundKind::If { conds, else_body })
    }

    /// Look past blanks/newlines/comments to see whether the next word is `kw`.
    fn next_word_after_newlines_is(&self, kw: &str) -> bool {
        let mut i = self.pos;
        loop {
            match self.s.get(i) {
                Some(' ') | Some('\t') | Some('\n') | Some('\r') => i += 1,
                Some('#') => {
                    while let Some(&c) = self.s.get(i) {
                        if c == '\n' {
                            break;
                        }
                        i += 1;
                    }
                }
                _ => break,
            }
        }
        let n = kw.chars().count();
        let w: String = self.s.iter().skip(i).take(n).collect();
        w == kw && is_delim(self.s.get(i + n).copied())
    }

    fn parse_do_body(&mut self) -> PResult<List> {
        self.skip_newlines()?;
        if self.peek_keyword() == Some("{") {
            self.consume_keyword("{");
            let body = self.parse_list(&["}"])?;
            self.expect_keyword("}")?;
            return Ok(body);
        }
        self.expect_keyword("do")?;
        let body = self.parse_list(&["done"])?;
        self.expect_keyword("done")?;
        Ok(body)
    }

    fn parse_for(&mut self, select: bool) -> PResult<CompoundKind> {
        self.skip_ws();
        if !select && self.starts_with("((") {
            self.advance(2);
            let start = self.pos;
            let text = self.scan_arith_close()?;
            let _ = start;
            let mut parts: Vec<String> = Vec::new();
            let mut depth = 0i32;
            let mut cur = String::new();
            for c in text.chars() {
                match c {
                    '(' => {
                        depth += 1;
                        cur.push(c)
                    }
                    ')' => {
                        depth -= 1;
                        cur.push(c)
                    }
                    ';' if depth == 0 => parts.push(std::mem::take(&mut cur)),
                    _ => cur.push(c),
                }
            }
            parts.push(cur);
            if parts.len() != 3 {
                return self.syntax("syntax error: arithmetic expression required");
            }
            let mk = |s: &str, p: &Parser| -> PResult<Option<Word>> {
                if s.trim().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(parse_arith_text(s, p.aliases.clone())?))
                }
            };
            let init = mk(&parts[0], self)?;
            let cond = mk(&parts[1], self)?;
            let step = mk(&parts[2], self)?;
            self.skip_ws();
            if self.peek() == Some(';') {
                self.bump();
            }
            let body = self.parse_do_body()?;
            return Ok(CompoundKind::ArithFor { init, cond, step, body });
        }
        let (var, end) = self.peek_word_raw();
        if var.is_empty() {
            return self.unexpected();
        }
        if !is_valid_name(&var) {
            return self.syntax(format!("`{}': not a valid identifier", var));
        }
        self.advance(end - self.pos);
        self.skip_ws();
        let mut words = None;
        if self.next_word_after_newlines_is("in") {
            self.skip_newlines()?;
            self.consume_keyword("in");
            let mut ws = Vec::new();
            loop {
                self.skip_ws();
                match self.peek() {
                    None => return Err(ParseError::Incomplete),
                    Some('\n') | Some(';') => break,
                    _ => match self.parse_word()? {
                        Some(w) => ws.push(w),
                        None => return self.unexpected(),
                    },
                }
            }
            words = Some(ws);
        }
        self.skip_ws();
        if self.peek() == Some(';') {
            self.bump();
        }
        let body = self.parse_do_body()?;
        if select {
            Ok(CompoundKind::Select { var, words, body })
        } else {
            Ok(CompoundKind::For { var, words, body })
        }
    }

    fn parse_case(&mut self) -> PResult<CompoundKind> {
        self.skip_ws();
        let word = match self.parse_word()? {
            Some(w) => w,
            None => return self.unexpected(),
        };
        self.skip_newlines()?;
        self.expect_keyword("in")?;
        let mut arms = Vec::new();
        loop {
            self.skip_newlines()?;
            if self.peek_keyword() == Some("esac") {
                self.consume_keyword("esac");
                break;
            }
            if self.eof() {
                return Err(ParseError::Incomplete);
            }
            if self.peek() == Some('(') {
                self.bump();
            }
            let mut patterns = Vec::new();
            loop {
                self.skip_ws();
                let w = match self.parse_word_ctx(Ctx { case_pat: true, ..Default::default() })? {
                    Some(w) => w,
                    None => return self.unexpected(),
                };
                patterns.push(w);
                self.skip_ws();
                match self.peek() {
                    Some('|') => {
                        self.bump();
                    }
                    Some(')') => {
                        self.bump();
                        break;
                    }
                    None => return Err(ParseError::Incomplete),
                    _ => return self.unexpected(),
                }
            }
            let body = self.parse_list(&["esac", ";;"])?;
            self.skip_ws();
            let term = if self.starts_with(";;&") {
                self.advance(3);
                CaseTerm::Continue
            } else if self.starts_with(";;") {
                self.advance(2);
                CaseTerm::Break
            } else if self.starts_with(";&") {
                self.advance(2);
                CaseTerm::FallThrough
            } else {
                CaseTerm::Break
            };
            arms.push(CaseArm { patterns, body, term });
        }
        Ok(CompoundKind::Case { word, arms })
    }

    // ---------- [[ ]] ----------

    fn cond_skip(&mut self) -> PResult<()> {
        loop {
            self.skip_ws();
            if self.peek() == Some('\n') {
                self.consume_newline()?;
            } else {
                return Ok(());
            }
        }
    }

    fn parse_cond_or(&mut self) -> PResult<CondExpr> {
        let mut l = self.parse_cond_and()?;
        loop {
            self.cond_skip()?;
            if self.starts_with("||") {
                self.advance(2);
                let r = self.parse_cond_and()?;
                l = CondExpr::Or(Box::new(l), Box::new(r));
            } else {
                return Ok(l);
            }
        }
    }

    fn parse_cond_and(&mut self) -> PResult<CondExpr> {
        let mut l = self.parse_cond_not()?;
        loop {
            self.cond_skip()?;
            if self.starts_with("&&") {
                self.advance(2);
                let r = self.parse_cond_not()?;
                l = CondExpr::And(Box::new(l), Box::new(r));
            } else {
                return Ok(l);
            }
        }
    }

    fn parse_cond_not(&mut self) -> PResult<CondExpr> {
        self.cond_skip()?;
        if self.peek() == Some('!') && matches!(self.peek_at(1), Some(' ') | Some('\t') | Some('\n')) {
            self.bump();
            let e = self.parse_cond_not()?;
            return Ok(CondExpr::Not(Box::new(e)));
        }
        self.parse_cond_primary()
    }

    fn cond_token(&self) -> String {
        let mut i = self.pos;
        let mut w = String::new();
        while let Some(&c) = self.s.get(i) {
            if matches!(c, ' ' | '\t' | '\n' | ')' | '(' | '\'' | '"' | '$' | '`' | '\\' | ';') {
                break;
            }
            w.push(c);
            i += 1;
        }
        w
    }

    fn at_cond_end(&self) -> bool {
        (self.starts_with("]]") && is_delim(self.peek_at(2)))
            || self.starts_with("&&")
            || self.starts_with("||")
            || self.peek() == Some(')')
            || self.eof()
    }

    fn parse_cond_primary(&mut self) -> PResult<CondExpr> {
        self.cond_skip()?;
        if self.eof() {
            return Err(ParseError::Incomplete);
        }
        if self.peek() == Some('(') {
            self.bump();
            let e = self.parse_cond_or()?;
            self.cond_skip()?;
            if self.peek() == Some(')') {
                self.bump();
                return Ok(e);
            }
            return self.unexpected();
        }
        if self.at_cond_end() {
            return self.unexpected();
        }
        let tok = self.cond_token();
        const UNARY: &[&str] = &[
            "-a", "-b", "-c", "-d", "-e", "-f", "-g", "-h", "-k", "-p", "-r", "-s", "-t", "-u", "-w", "-x", "-G",
            "-L", "-N", "-O", "-S", "-z", "-n", "-o", "-v", "-R",
        ];
        if UNARY.contains(&tok.as_str()) {
            let m = self.mark();
            self.advance(tok.chars().count());
            self.skip_ws();
            if !self.at_cond_end() {
                let op_after = self.cond_token();
                // `-n == x`? treat tok as word if followed by a binary operator
                if !matches!(op_after.as_str(), "==" | "=" | "!=" | "=~") {
                    let w = match self.parse_word_ctx(Ctx { cond: true, ..Default::default() })? {
                        Some(w) => w,
                        None => return self.unexpected(),
                    };
                    return Ok(CondExpr::Unary(tok, w));
                }
            }
            self.reset(m);
        }
        let w1 = match self.parse_word_ctx(Ctx { cond: true, ..Default::default() })? {
            Some(w) => w,
            None => return self.unexpected(),
        };
        self.skip_ws();
        let op_tok = self.cond_token();
        let op = if self.peek() == Some('<') {
            Some("<".to_string())
        } else if self.peek() == Some('>') {
            Some(">".to_string())
        } else {
            match op_tok.as_str() {
                "==" | "=" | "!=" | "=~" | "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge" | "-nt" | "-ot" | "-ef" => {
                    Some(op_tok.clone())
                }
                _ => None,
            }
        };
        if let Some(op) = op {
            self.advance(op.chars().count());
            self.skip_ws();
            let ctx = if op == "=~" { Ctx { regex: true, cond: true, ..Default::default() } } else { Ctx { cond: true, ..Default::default() } };
            let w2 = match self.parse_word_ctx(ctx)? {
                Some(w) => w,
                None => return self.unexpected(),
            };
            return Ok(CondExpr::Binary(w1, op, w2));
        }
        Ok(CondExpr::Word(w1))
    }

    // ---------- simple commands ----------

    fn parse_simple_command(&mut self) -> PResult<Command> {
        let mut sc = SimpleCommand { line: self.line, ..Default::default() };
        let start = self.pos;
        let mut is_decl = false;
        loop {
            self.skip_ws();
            let c = match self.peek() {
                None => break,
                Some(c) => c,
            };
            if matches!(c, '\n' | ';' | '|' | ')') {
                break;
            }
            if c == '&' && self.peek_at(1) != Some('>') {
                break;
            }
            if c == '(' {
                return self.unexpected();
            }
            if let Some(r) = self.try_parse_redir()? {
                sc.redirs.push(r);
                continue;
            }
            if sc.words.is_empty() {
                if let Some(a) = self.try_parse_assignment()? {
                    sc.assigns.push(a);
                    continue;
                }
                if self.aliases.is_some() && (!sc.assigns.is_empty() || !sc.redirs.is_empty()) && self.try_alias() {
                    continue;
                }
            } else {
                if self.alias_next {
                    self.alias_next = false;
                    if self.try_alias() {
                        continue;
                    }
                }
                if is_decl {
                    if let Some(a) = self.try_parse_assignment()? {
                        sc.words.push(Word { parts: vec![WordPart::DeclAssign(Box::new(a))] });
                        continue;
                    }
                }
            }
            let w = match self.parse_word()? {
                Some(w) => w,
                None => return self.unexpected(),
            };
            sc.words.push(w);
            if sc.words.len() == 1 {
                let plain = sc.words[0].as_plain();
                if let Some(p) = &plain {
                    let base = p.as_str();
                    is_decl = DECL_BUILTINS.contains(&base);
                }
                // function definition `name ( )`
                if sc.assigns.is_empty() && sc.redirs.is_empty() {
                    if let Some(name) = plain {
                        let m = self.mark();
                        self.skip_ws();
                        if self.peek() == Some('(') {
                            self.bump();
                            self.skip_ws();
                            if self.peek() == Some(')') {
                                self.bump();
                                return self.parse_function_body(name, start, sc.line);
                            }
                            return self.unexpected();
                        }
                        self.reset(m);
                    }
                }
            } else if is_decl {
                // `declare -a` etc. Keep scanning.
            }
        }
        if sc.words.is_empty() && sc.assigns.is_empty() && sc.redirs.is_empty() {
            return self.unexpected();
        }
        Ok(Command::Simple(sc))
    }

    /// Scan from the current position (just after an opening `[`) to the matching `]`, returning the raw text.
    fn scan_bracket(&mut self) -> Option<String> {
        let m = self.mark();
        let mut depth = 0;
        let start = self.pos;
        loop {
            let c = match self.peek() {
                None => {
                    self.reset(m);
                    return None;
                }
                Some(c) => c,
            };
            match c {
                '[' => depth += 1,
                ']' => {
                    if depth == 0 {
                        let t = self.text(start, self.pos);
                        self.bump();
                        return Some(t);
                    }
                    depth -= 1;
                }
                '\\' => {
                    self.bump();
                }
                '\'' => {
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c == '\'' {
                            break;
                        }
                        self.bump();
                    }
                }
                '"' => {
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            break;
                        }
                        if c == '\\' {
                            self.bump();
                        }
                        self.bump();
                    }
                }
                '$' if self.peek_at(1) == Some('(') || self.peek_at(1) == Some('{') => {
                    let open = self.peek_at(1).unwrap();
                    let close = if open == '(' { ')' } else { '}' };
                    self.bump();
                    self.bump();
                    let mut d = 1;
                    while let Some(c) = self.peek() {
                        if c == open {
                            d += 1;
                        } else if c == close {
                            d -= 1;
                            if d == 0 {
                                break;
                            }
                        }
                        self.bump();
                    }
                }
                '\n' | ';' | '&' | '|' | '<' | '>' if depth == 0 => {
                    self.reset(m);
                    return None;
                }
                _ => {}
            }
            self.bump();
        }
    }

    fn try_parse_assignment(&mut self) -> PResult<Option<Assign>> {
        let m = self.mark();
        match self.peek() {
            Some(c) if is_name_start(c) => {}
            _ => return Ok(None),
        }
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if is_name_char(c) {
                name.push(c);
                self.bump();
            } else {
                break;
            }
        }
        let mut index = None;
        if self.peek() == Some('[') {
            self.bump();
            match self.scan_bracket() {
                Some(t) => index = Some(parse_operand_text(&t, self.aliases.clone())?),
                None => {
                    self.reset(m);
                    return Ok(None);
                }
            }
        }
        let append = if self.starts_with("+=") {
            self.advance(2);
            true
        } else if self.peek() == Some('=') {
            self.bump();
            false
        } else {
            self.reset(m);
            return Ok(None);
        };
        if self.peek() == Some('(') {
            self.bump();
            let mut elems = Vec::new();
            loop {
                self.skip_newlines()?;
                match self.peek() {
                    None => return Err(ParseError::Incomplete),
                    Some(')') => {
                        self.bump();
                        break;
                    }
                    Some('[') => {
                        let m2 = self.mark();
                        self.bump();
                        if let Some(k) = self.scan_bracket() {
                            let app = self.starts_with("+=");
                            if self.peek() == Some('=') || app {
                                self.advance(if app { 2 } else { 1 });
                                let key = parse_operand_text(&k, self.aliases.clone())?;
                                let v = self.parse_word_ctx(Ctx { assign: true, ..Default::default() })?.unwrap_or_default();
                                elems.push((Some(key), v));
                                continue;
                            }
                        }
                        self.reset(m2);
                        match self.parse_word()? {
                            Some(w) => elems.push((None, w)),
                            None => return self.unexpected(),
                        }
                    }
                    Some(_) => match self.parse_word()? {
                        Some(w) => elems.push((None, w)),
                        None => return self.unexpected(),
                    },
                }
            }
            return Ok(Some(Assign { name, index, append, value: AssignValue::Array(elems) }));
        }
        let w = self.parse_word_ctx(Ctx { assign: true, ..Default::default() })?.unwrap_or_default();
        Ok(Some(Assign { name, index, append, value: AssignValue::Scalar(w) }))
    }

    fn try_parse_redir(&mut self) -> PResult<Option<Redir>> {
        let m = self.mark();
        let mut fd = None;
        let mut fd_var = None;
        match self.peek() {
            Some(c) if c.is_ascii_digit() => {
                let mut n = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        n.push(c);
                        self.bump();
                    } else {
                        break;
                    }
                }
                if matches!(self.peek(), Some('<') | Some('>')) && self.peek_at(1) != Some('(') {
                    fd = n.parse::<i32>().ok();
                } else {
                    self.reset(m);
                    return Ok(None);
                }
            }
            Some('{') => {
                self.bump();
                let mut n = String::new();
                while let Some(c) = self.peek() {
                    if is_name_char(c) {
                        n.push(c);
                        self.bump();
                    } else {
                        break;
                    }
                }
                if self.peek() == Some('}') && is_valid_name(&n) {
                    self.bump();
                    if matches!(self.peek(), Some('<') | Some('>')) {
                        fd_var = Some(n);
                    } else {
                        self.reset(m);
                        return Ok(None);
                    }
                } else {
                    self.reset(m);
                    return Ok(None);
                }
            }
            _ => {}
        }
        let (op, n) = if self.starts_with("&>>") {
            (RedirOp::AppendOutErr, 3)
        } else if self.starts_with("&>") {
            (RedirOp::OutErr, 2)
        } else if self.starts_with("<<<") {
            (RedirOp::HereString, 3)
        } else if self.starts_with("<<-") {
            (RedirOp::HereDoc, 3)
        } else if self.starts_with("<<") {
            (RedirOp::HereDoc, 2)
        } else if self.starts_with("<>") {
            (RedirOp::ReadWrite, 2)
        } else if self.starts_with("<&") {
            (RedirOp::DupIn, 2)
        } else if self.starts_with("<(") {
            self.reset(m);
            return Ok(None);
        } else if self.starts_with("<") {
            (RedirOp::In, 1)
        } else if self.starts_with(">>") {
            (RedirOp::Append, 2)
        } else if self.starts_with(">|") {
            (RedirOp::Clobber, 2)
        } else if self.starts_with(">&") {
            (RedirOp::DupOut, 2)
        } else if self.starts_with(">(") {
            self.reset(m);
            return Ok(None);
        } else if self.starts_with(">") {
            (RedirOp::Out, 1)
        } else {
            self.reset(m);
            return Ok(None);
        };
        let strip = op == RedirOp::HereDoc && n == 3;
        self.advance(n);
        self.skip_ws();
        if op == RedirOp::HereDoc {
            let w = match self.parse_word()? {
                Some(w) => w,
                None => return self.unexpected(),
            };
            let quoted = w.is_quoted();
            let delim = heredoc_delim_text(&w);
            let cell = Arc::new(OnceLock::new());
            self.heredocs.push(PendingHeredoc { cell: cell.clone(), delim, strip, quoted });
            return Ok(Some(Redir { fd, fd_var, op, target: RedirTarget::HereDoc(cell) }));
        }
        let w = match self.parse_word()? {
            Some(w) => w,
            None => return self.unexpected(),
        };
        Ok(Some(Redir { fd, fd_var, op, target: RedirTarget::Word(w) }))
    }

    // ---------- words ----------

    pub fn parse_word(&mut self) -> PResult<Option<Word>> {
        self.parse_word_ctx(Ctx::default())
    }

    fn try_tilde(&mut self, assign: bool) -> Option<WordPart> {
        // at '~'
        let mut i = self.pos + 1;
        let mut user = String::new();
        while let Some(&c) = self.s.get(i) {
            if c == '/' || is_meta(c) || (assign && c == ':') {
                break;
            }
            if matches!(c, '\'' | '"' | '\\' | '$' | '`' | '*' | '?' | '[' | '{' | '}') {
                return None;
            }
            user.push(c);
            i += 1;
        }
        self.pos = i;
        Some(WordPart::Tilde(user))
    }

    fn parse_word_ctx(&mut self, ctx: Ctx) -> PResult<Option<Word>> {
        let start = self.pos;
        let mut parts: Vec<WordPart> = Vec::new();
        let mut lit = String::new();
        let mut depth = 0i32;
        macro_rules! flush {
            () => {
                if !lit.is_empty() {
                    parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                }
            };
        }
        if self.peek() == Some('~') && !ctx.regex {
            if let Some(t) = self.try_tilde(ctx.assign) {
                parts.push(t);
            }
        }
        // process substitution at word start
        loop {
            let c = match self.peek() {
                None => break,
                Some(c) => c,
            };
            match c {
                ' ' | '\t' | '\n' => {
                    if ctx.regex && depth > 0 && c != '\n' {
                        lit.push(c);
                        self.bump();
                        continue;
                    }
                    break;
                }
                '\r' => {
                    // stray CR (CRLF scripts) ends a word
                    if self.peek_at(1) == Some('\n') || self.peek_at(1).is_none() {
                        break;
                    }
                    lit.push(c);
                    self.bump();
                }
                ';' | '&' => {
                    if ctx.regex && depth > 0 {
                        lit.push(c);
                        self.bump();
                        continue;
                    }
                    break;
                }
                '|' => {
                    if ctx.regex {
                        lit.push(c);
                        self.bump();
                        continue;
                    }
                    break;
                }
                '<' | '>' => {
                    if ctx.regex && depth > 0 {
                        lit.push(c);
                        self.bump();
                        continue;
                    }
                    if self.peek_at(1) == Some('(') && !ctx.cond {
                        flush!();
                        let input = c == '<';
                        self.advance(2);
                        let prog = self.parse_subprogram()?;
                        parts.push(WordPart::ProcSubst { input, prog: Arc::new(prog) });
                        continue;
                    }
                    break;
                }
                '(' => {
                    if ctx.regex {
                        depth += 1;
                        lit.push(c);
                        self.bump();
                        continue;
                    }
                    if let Some(last) = lit.chars().last() {
                        if matches!(last, '?' | '*' | '+' | '@' | '!') {
                            let g = self.read_extglob_group()?;
                            lit.push_str(&g);
                            continue;
                        }
                    }
                    break;
                }
                ')' => {
                    if ctx.regex && depth > 0 {
                        depth -= 1;
                        lit.push(c);
                        self.bump();
                        continue;
                    }
                    break;
                }
                '\\' => {
                    self.bump();
                    match self.peek() {
                        Some('\n') => {
                            self.bump();
                        }
                        Some(n) => {
                            flush!();
                            self.bump();
                            parts.push(WordPart::Escaped(n.to_string()));
                        }
                        None => lit.push('\\'),
                    }
                }
                '\'' => {
                    flush!();
                    self.bump();
                    let mut s = String::new();
                    loop {
                        match self.bump() {
                            None => return Err(ParseError::Incomplete),
                            Some('\'') => break,
                            Some(c) => s.push(c),
                        }
                    }
                    parts.push(WordPart::SQuoted(s));
                }
                '"' => {
                    flush!();
                    self.bump();
                    parts.push(self.parse_dquoted()?);
                }
                '$' => {
                    flush!();
                    match self.parse_dollar(false)? {
                        Some(p) => parts.push(p),
                        None => lit.push('$'),
                    }
                }
                '`' => {
                    flush!();
                    parts.push(self.parse_backtick(false)?);
                }
                ':' if ctx.assign => {
                    lit.push(':');
                    self.bump();
                    if self.peek() == Some('~') {
                        flush!();
                        if let Some(t) = self.try_tilde(true) {
                            parts.push(t);
                        }
                    }
                }
                _ => {
                    lit.push(c);
                    self.bump();
                }
            }
            let _ = ctx.case_pat;
        }
        flush!();
        if self.pos == start {
            return Ok(None);
        }
        Ok(Some(Word { parts }))
    }

    fn read_extglob_group(&mut self) -> PResult<String> {
        // at '('
        let mut s = String::new();
        let mut depth = 0;
        loop {
            let c = match self.bump() {
                None => return Err(ParseError::Incomplete),
                Some(c) => c,
            };
            match c {
                '(' => {
                    depth += 1;
                    s.push(c)
                }
                ')' => {
                    depth -= 1;
                    s.push(c);
                    if depth == 0 {
                        return Ok(s);
                    }
                }
                '\\' => {
                    s.push('\\');
                    if let Some(n) = self.bump() {
                        s.push(n);
                    }
                }
                '\'' => {
                    while let Some(n) = self.bump() {
                        if n == '\'' {
                            break;
                        }
                        if "*?[]()|!@+\\".contains(n) {
                            s.push('\\');
                        }
                        s.push(n);
                    }
                }
                '"' => {
                    while let Some(n) = self.bump() {
                        if n == '"' {
                            break;
                        }
                        if "*?[]()|!@+\\".contains(n) {
                            s.push('\\');
                        }
                        s.push(n);
                    }
                }
                _ => s.push(c),
            }
        }
    }

    fn parse_dquoted(&mut self) -> PResult<WordPart> {
        let mut parts = Vec::new();
        let mut lit = String::new();
        loop {
            let c = match self.peek() {
                None => return Err(ParseError::Incomplete),
                Some(c) => c,
            };
            match c {
                '"' => {
                    self.bump();
                    break;
                }
                '\\' => {
                    self.bump();
                    match self.peek() {
                        Some(n @ ('$' | '`' | '"' | '\\')) => {
                            self.bump();
                            lit.push(n);
                        }
                        Some('\n') => {
                            self.bump();
                        }
                        _ => lit.push('\\'),
                    }
                }
                '$' => {
                    if !lit.is_empty() {
                        parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                    }
                    match self.parse_dollar(true)? {
                        Some(p) => parts.push(p),
                        None => lit.push('$'),
                    }
                }
                '`' => {
                    if !lit.is_empty() {
                        parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                    }
                    parts.push(self.parse_backtick(true)?);
                }
                _ => {
                    lit.push(c);
                    self.bump();
                }
            }
        }
        if !lit.is_empty() {
            parts.push(WordPart::Lit(lit));
        }
        Ok(WordPart::DQuoted(parts))
    }

    fn parse_backtick(&mut self, in_dq: bool) -> PResult<WordPart> {
        self.bump(); // `
        let mut text = String::new();
        loop {
            match self.bump() {
                None => return Err(ParseError::Incomplete),
                Some('`') => break,
                Some('\\') => match self.peek() {
                    Some(c) if c == '$' || c == '`' || c == '\\' || (in_dq && c == '"') => {
                        self.bump();
                        text.push(c);
                    }
                    Some('\n') => {
                        self.bump();
                    }
                    _ => text.push('\\'),
                },
                Some(c) => text.push(c),
            }
        }
        let mut p = Parser::new(&text).with_aliases(self.aliases.clone());
        p.line = self.line;
        let prog = p.parse_program().map_err(|e| match e {
            ParseError::Incomplete => ParseError::Syntax {
                msg: "unexpected EOF while looking for matching ``'".into(),
                line: self.line,
            },
            e => e,
        })?;
        Ok(WordPart::Backtick(Arc::new(prog)))
    }

    /// Parse a command list up to a closing `)` (consumed).
    fn parse_subprogram(&mut self) -> PResult<Program> {
        let saved_alias_next = self.alias_next;
        self.alias_next = false;
        let items = self.parse_list(&[")"])?;
        self.skip_newlines()?;
        match self.peek() {
            Some(')') => {
                self.bump();
            }
            None => return Err(ParseError::Incomplete),
            _ => return self.unexpected(),
        }
        self.alias_next = saved_alias_next;
        Ok(Program { items })
    }

    /// At `((`: scan for matching `))`. Returns the inner text, consuming through `))`,
    /// or None (position unchanged) if this isn't an arithmetic construct.
    fn try_double_paren(&mut self) -> PResult<Option<Word>> {
        let m = self.mark();
        self.advance(2);
        match self.scan_arith_close_opt()? {
            Some(text) => Ok(Some(parse_arith_text(&text, self.aliases.clone())?)),
            None => {
                self.reset(m);
                Ok(None)
            }
        }
    }

    fn scan_arith_close(&mut self) -> PResult<String> {
        match self.scan_arith_close_opt()? {
            Some(t) => Ok(t),
            None => self.syntax("syntax error: missing `))'"),
        }
    }

    /// Position is just after `((`. Find the `))` that closes it.
    fn scan_arith_close_opt(&mut self) -> PResult<Option<String>> {
        let start = self.pos;
        let mut depth = 0i32;
        loop {
            let c = match self.peek() {
                None => return Err(ParseError::Incomplete),
                Some(c) => c,
            };
            match c {
                '(' => depth += 1,
                ')' => {
                    if depth == 0 {
                        if self.peek_at(1) == Some(')') {
                            let t = self.text(start, self.pos);
                            self.advance(2);
                            return Ok(Some(t));
                        }
                        return Ok(None);
                    }
                    depth -= 1;
                }
                '\'' => {
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c == '\'' {
                            break;
                        }
                        self.bump();
                    }
                }
                '"' => {
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            break;
                        }
                        if c == '\\' {
                            self.bump();
                        }
                        self.bump();
                    }
                }
                '\\' => {
                    self.bump();
                }
                _ => {}
            }
            self.bump();
        }
    }

    fn read_name(&mut self) -> String {
        let mut n = String::new();
        while let Some(c) = self.peek() {
            if is_name_char(c) {
                n.push(c);
                self.bump();
            } else {
                break;
            }
        }
        n
    }

    /// At `$`. Returns None if the `$` is literal (position after the `$`).
    fn parse_dollar(&mut self, in_dq: bool) -> PResult<Option<WordPart>> {
        self.bump();
        let simple = |name: String| {
            WordPart::Param(Box::new(ParamExp { name, index: None, indirect: false, op: ParamOp::None }))
        };
        match self.peek() {
            Some('\'') if !in_dq => {
                self.bump();
                let s = self.read_ansi_c()?;
                Ok(Some(WordPart::SQuoted(s)))
            }
            Some('"') if !in_dq => {
                self.bump();
                Ok(Some(self.parse_dquoted()?))
            }
            Some('{') => {
                self.bump();
                Ok(Some(self.parse_brace_param(in_dq)?))
            }
            Some('(') => {
                if self.peek_at(1) == Some('(') {
                    if let Some(w) = self.try_double_paren()? {
                        return Ok(Some(WordPart::Arith(w)));
                    }
                }
                self.bump();
                let prog = self.parse_subprogram()?;
                Ok(Some(WordPart::CmdSubst(Arc::new(prog))))
            }
            Some('[') => {
                self.bump();
                let start = self.pos;
                let mut depth = 0;
                loop {
                    match self.peek() {
                        None => return Err(ParseError::Incomplete),
                        Some('[') => depth += 1,
                        Some(']') => {
                            if depth == 0 {
                                break;
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    self.bump();
                }
                let t = self.text(start, self.pos);
                self.bump();
                Ok(Some(WordPart::Arith(parse_arith_text(&t, self.aliases.clone())?)))
            }
            Some(c) if is_name_start(c) => {
                let n = self.read_name();
                Ok(Some(simple(n)))
            }
            Some(c) if c.is_ascii_digit() => {
                self.bump();
                Ok(Some(simple(c.to_string())))
            }
            Some(c) if "@*#?-$!".contains(c) => {
                self.bump();
                Ok(Some(simple(c.to_string())))
            }
            _ => Ok(None),
        }
    }

    fn read_ansi_c(&mut self) -> PResult<String> {
        let mut out = String::new();
        loop {
            let c = match self.bump() {
                None => return Err(ParseError::Incomplete),
                Some(c) => c,
            };
            match c {
                '\'' => break,
                '\\' => {
                    let e = match self.bump() {
                        None => return Err(ParseError::Incomplete),
                        Some(e) => e,
                    };
                    match e {
                        'a' => out.push('\x07'),
                        'b' => out.push('\x08'),
                        'e' | 'E' => out.push('\x1b'),
                        'f' => out.push('\x0c'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'v' => out.push('\x0b'),
                        '\\' => out.push('\\'),
                        '\'' => out.push('\''),
                        '"' => out.push('"'),
                        '?' => out.push('?'),
                        '0'..='7' => {
                            let mut v = e.to_digit(8).unwrap();
                            for _ in 0..2 {
                                match self.peek() {
                                    Some(d) if d.is_digit(8) => {
                                        v = v * 8 + d.to_digit(8).unwrap();
                                        self.bump();
                                    }
                                    _ => break,
                                }
                            }
                            push_byte_char(&mut out, v as u8);
                        }
                        'x' => {
                            let mut v = 0u32;
                            let mut n = 0;
                            while n < 2 {
                                match self.peek() {
                                    Some(d) if d.is_ascii_hexdigit() => {
                                        v = v * 16 + d.to_digit(16).unwrap();
                                        self.bump();
                                        n += 1;
                                    }
                                    _ => break,
                                }
                            }
                            if n == 0 {
                                out.push_str("\\x");
                            } else {
                                push_byte_char(&mut out, v as u8);
                            }
                        }
                        'u' | 'U' => {
                            let max = if e == 'u' { 4 } else { 8 };
                            let mut v = 0u32;
                            let mut n = 0;
                            while n < max {
                                match self.peek() {
                                    Some(d) if d.is_ascii_hexdigit() => {
                                        v = v.wrapping_mul(16) + d.to_digit(16).unwrap();
                                        self.bump();
                                        n += 1;
                                    }
                                    _ => break,
                                }
                            }
                            if n == 0 {
                                out.push('\\');
                                out.push(e);
                            } else if let Some(ch) = char::from_u32(v) {
                                out.push(ch);
                            }
                        }
                        'c' => {
                            if let Some(x) = self.bump() {
                                let v = (x.to_ascii_uppercase() as u32) ^ 0x40;
                                if let Some(ch) = char::from_u32(v) {
                                    out.push(ch);
                                }
                            }
                        }
                        other => {
                            out.push('\\');
                            out.push(other);
                        }
                    }
                }
                c => out.push(c),
            }
        }
        Ok(out)
    }

    fn try_param_name(&mut self) -> Option<String> {
        match self.peek() {
            Some(c) if is_name_start(c) => Some(self.read_name()),
            Some(c) if c.is_ascii_digit() => {
                let mut n = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        n.push(c);
                        self.bump();
                    } else {
                        break;
                    }
                }
                Some(n)
            }
            Some(c) if "@*#?-$!".contains(c) => {
                self.bump();
                Some(c.to_string())
            }
            _ => None,
        }
    }

    fn parse_subscript(&mut self) -> PResult<Word> {
        // at '['
        self.bump();
        let start = self.pos;
        let mut depth = 0;
        loop {
            match self.peek() {
                None => return Err(ParseError::Incomplete),
                Some('[') => depth += 1,
                Some(']') => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                Some('\\') => {
                    self.bump();
                }
                Some('\'') => {
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c == '\'' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('"') => {
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            break;
                        }
                        if c == '\\' {
                            self.bump();
                        }
                        self.bump();
                    }
                }
                _ => {}
            }
            self.bump();
        }
        let t = self.text(start, self.pos);
        self.bump();
        parse_operand_text(&t, self.aliases.clone())
    }

    fn expect_close_brace(&mut self) -> PResult<()> {
        match self.peek() {
            Some('}') => {
                self.bump();
                Ok(())
            }
            None => Err(ParseError::Incomplete),
            _ => self.syntax("bad substitution"),
        }
    }

    /// Skip to the matching `}` and produce a runtime "bad substitution" expansion.
    fn bad_subst(&mut self, name: String) -> PResult<WordPart> {
        let mut depth = 0;
        let mut text = name;
        loop {
            match self.bump() {
                None => return Err(ParseError::Incomplete),
                Some('{') => {
                    depth += 1;
                    text.push('{');
                }
                Some('}') => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    text.push('}');
                }
                Some(c) => text.push(c),
            }
        }
        Ok(WordPart::Param(Box::new(ParamExp { name: String::new(), index: None, indirect: false, op: ParamOp::Bad(text) })))
    }

    fn parse_brace_param(&mut self, in_dq: bool) -> PResult<WordPart> {
        let mut pe = ParamExp { name: String::new(), index: None, indirect: false, op: ParamOp::None };
        if self.peek() == Some('#') {
            let m = self.mark();
            self.bump();
            if let Some(name) = self.try_param_name() {
                let mut idx = None;
                if self.peek() == Some('[') {
                    idx = Some(self.parse_subscript()?);
                }
                if self.peek() == Some('}') {
                    self.bump();
                    pe.name = name;
                    pe.index = idx;
                    pe.op = ParamOp::Length;
                    return Ok(WordPart::Param(Box::new(pe)));
                }
            }
            self.reset(m);
        }
        if self.peek() == Some('!') {
            let m = self.mark();
            self.bump();
            if let Some(name) = self.try_param_name() {
                if matches!(self.peek(), Some('*') | Some('@')) && self.peek_at(1) == Some('}') && is_valid_name(&name) {
                    let star = self.peek() == Some('*');
                    self.advance(2);
                    pe.name = name;
                    pe.op = ParamOp::PrefixNames { star };
                    return Ok(WordPart::Param(Box::new(pe)));
                }
                if self.peek() == Some('[') {
                    let m2 = self.mark();
                    let idx = self.parse_subscript()?;
                    if let Some(p) = idx.as_plain() {
                        if (p == "@" || p == "*") && self.peek() == Some('}') {
                            self.bump();
                            pe.name = name;
                            pe.op = ParamOp::Keys { star: p == "*" };
                            return Ok(WordPart::Param(Box::new(pe)));
                        }
                    }
                    self.reset(m2);
                }
                pe.indirect = true;
                pe.name = name;
            } else {
                self.reset(m);
            }
        }
        if pe.name.is_empty() {
            match self.try_param_name() {
                Some(n) => pe.name = n,
                None => {
                    if self.eof() {
                        return Err(ParseError::Incomplete);
                    }
                    return self.bad_subst(String::new());
                }
            }
        }
        if self.peek() == Some('[') && pe.index.is_none() {
            pe.index = Some(self.parse_subscript()?);
        }
        let c = match self.peek() {
            None => return Err(ParseError::Incomplete),
            Some(c) => c,
        };
        match c {
            '}' => {
                self.bump();
            }
            ':' => {
                self.bump();
                match self.peek() {
                    Some('-') => {
                        self.bump();
                        let w = self.parse_operand(&['}'], in_dq, true)?;
                        pe.op = ParamOp::Default { colon: true, word: w };
                    }
                    Some('=') => {
                        self.bump();
                        let w = self.parse_operand(&['}'], in_dq, true)?;
                        pe.op = ParamOp::Assign { colon: true, word: w };
                    }
                    Some('?') => {
                        self.bump();
                        let w = self.parse_operand(&['}'], in_dq, true)?;
                        pe.op = ParamOp::Error { colon: true, word: if w.parts.is_empty() { None } else { Some(w) } };
                    }
                    Some('+') => {
                        self.bump();
                        let w = self.parse_operand(&['}'], in_dq, true)?;
                        pe.op = ParamOp::Alt { colon: true, word: w };
                    }
                    _ => {
                        let off = self.parse_operand(&[':', '}'], in_dq, false)?;
                        let mut len = None;
                        if self.peek() == Some(':') {
                            self.bump();
                            len = Some(self.parse_operand(&['}'], in_dq, false)?);
                        }
                        pe.op = ParamOp::Substr { offset: off, length: len };
                    }
                }
                self.expect_close_brace()?;
            }
            '-' | '=' | '?' | '+' => {
                self.bump();
                let w = self.parse_operand(&['}'], in_dq, true)?;
                pe.op = match c {
                    '-' => ParamOp::Default { colon: false, word: w },
                    '=' => ParamOp::Assign { colon: false, word: w },
                    '?' => ParamOp::Error { colon: false, word: if w.parts.is_empty() { None } else { Some(w) } },
                    _ => ParamOp::Alt { colon: false, word: w },
                };
                self.expect_close_brace()?;
            }
            '#' | '%' => {
                self.bump();
                let longest = if self.peek() == Some(c) {
                    self.bump();
                    true
                } else {
                    false
                };
                let pat = self.parse_operand(&['}'], in_dq, false)?;
                pe.op = if c == '#' { ParamOp::TrimPrefix { longest, pat } } else { ParamOp::TrimSuffix { longest, pat } };
                self.expect_close_brace()?;
            }
            '/' => {
                self.bump();
                let mode = match self.peek() {
                    Some('/') => {
                        self.bump();
                        ReplMode::All
                    }
                    Some('#') => {
                        self.bump();
                        ReplMode::Prefix
                    }
                    Some('%') => {
                        self.bump();
                        ReplMode::Suffix
                    }
                    _ => ReplMode::First,
                };
                let pat = self.parse_operand(&['/', '}'], in_dq, false)?;
                let mut rep = None;
                if self.peek() == Some('/') {
                    self.bump();
                    rep = Some(self.parse_operand(&['}'], in_dq, true)?);
                }
                pe.op = ParamOp::Replace { mode, pat, rep };
                self.expect_close_brace()?;
            }
            '^' | ',' => {
                self.bump();
                let all = if self.peek() == Some(c) {
                    self.bump();
                    true
                } else {
                    false
                };
                let pat = self.parse_operand(&['}'], in_dq, false)?;
                pe.op = ParamOp::Case { upper: c == '^', all, pat: if pat.parts.is_empty() { None } else { Some(pat) } };
                self.expect_close_brace()?;
            }
            '~' => {
                self.bump();
                let all = if self.peek() == Some('~') {
                    self.bump();
                    true
                } else {
                    false
                };
                let pat = self.parse_operand(&['}'], in_dq, false)?;
                pe.op = ParamOp::Toggle { all, pat: if pat.parts.is_empty() { None } else { Some(pat) } };
                self.expect_close_brace()?;
            }
            '@' => {
                self.bump();
                let t = match self.bump() {
                    None => return Err(ParseError::Incomplete),
                    Some(t) => t,
                };
                pe.op = ParamOp::Transform(t);
                if self.peek() != Some('}') {
                    return self.bad_subst(pe.name);
                }
                self.bump();
            }
            _ => {
                return self.bad_subst(pe.name);
            }
        }
        Ok(WordPart::Param(Box::new(pe)))
    }

    /// Parse an operand inside `${...}` up to (not including) one of `stops` at nesting depth 0.
    fn parse_operand(&mut self, stops: &[char], in_dq: bool, value_op: bool) -> PResult<Word> {
        let mut parts = Vec::new();
        let mut lit = String::new();
        let mut depth = 0;
        macro_rules! flush {
            () => {
                if !lit.is_empty() {
                    parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                }
            };
        }
        if self.peek() == Some('~') && value_op && !in_dq {
            if let Some(t) = self.try_tilde(false) {
                parts.push(t);
            }
        }
        loop {
            let c = match self.peek() {
                None => return Err(ParseError::Incomplete),
                Some(c) => c,
            };
            if depth == 0 && stops.contains(&c) {
                break;
            }
            match c {
                '\\' => {
                    self.bump();
                    match self.peek() {
                        None => return Err(ParseError::Incomplete),
                        Some('\n') => {
                            self.bump();
                        }
                        Some(n) => {
                            self.bump();
                            if in_dq && value_op && !matches!(n, '$' | '`' | '"' | '\\' | '}' | '/') {
                                lit.push('\\');
                                lit.push(n);
                            } else {
                                flush!();
                                parts.push(WordPart::Escaped(n.to_string()));
                            }
                        }
                    }
                }
                '\'' if !(in_dq && value_op) => {
                    flush!();
                    self.bump();
                    let mut s = String::new();
                    loop {
                        match self.bump() {
                            None => return Err(ParseError::Incomplete),
                            Some('\'') => break,
                            Some(c) => s.push(c),
                        }
                    }
                    parts.push(WordPart::SQuoted(s));
                }
                '"' => {
                    flush!();
                    self.bump();
                    parts.push(self.parse_dquoted()?);
                }
                '$' => {
                    flush!();
                    match self.parse_dollar(in_dq && value_op)? {
                        Some(p) => parts.push(p),
                        None => lit.push('$'),
                    }
                }
                '`' => {
                    flush!();
                    parts.push(self.parse_backtick(in_dq)?);
                }
                '{' => {
                    depth += 1;
                    lit.push(c);
                    self.bump();
                }
                '}' => {
                    depth -= 1;
                    lit.push(c);
                    self.bump();
                }
                _ => {
                    lit.push(c);
                    self.bump();
                }
            }
        }
        flush!();
        Ok(Word { parts })
    }

    /// Parse the entire remaining input as an operand word (quotes and expansions active,
    /// whitespace literal).
    fn parse_operand_to_end(&mut self) -> PResult<Word> {
        let mut parts = Vec::new();
        let mut lit = String::new();
        macro_rules! flush {
            () => {
                if !lit.is_empty() {
                    parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                }
            };
        }
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    self.bump();
                    match self.peek() {
                        Some('\n') => {
                            self.bump();
                        }
                        Some(n) => {
                            self.bump();
                            flush!();
                            parts.push(WordPart::Escaped(n.to_string()));
                        }
                        None => lit.push('\\'),
                    }
                }
                '\'' => {
                    flush!();
                    self.bump();
                    let mut s = String::new();
                    loop {
                        match self.bump() {
                            None => return Err(ParseError::Incomplete),
                            Some('\'') => break,
                            Some(c) => s.push(c),
                        }
                    }
                    parts.push(WordPart::SQuoted(s));
                }
                '"' => {
                    flush!();
                    self.bump();
                    parts.push(self.parse_dquoted()?);
                }
                '$' => {
                    flush!();
                    match self.parse_dollar(false)? {
                        Some(p) => parts.push(p),
                        None => lit.push('$'),
                    }
                }
                '`' => {
                    flush!();
                    parts.push(self.parse_backtick(false)?);
                }
                _ => {
                    lit.push(c);
                    self.bump();
                }
            }
        }
        flush!();
        Ok(Word { parts })
    }
}

fn push_byte_char(out: &mut String, b: u8) {
    // Bytes >= 0x80 can't be represented individually in a Rust String; map to the
    // corresponding Latin-1 char which round-trips well enough for practical use.
    out.push(b as char);
}

fn heredoc_delim_text(w: &Word) -> String {
    fn rec(parts: &[WordPart], s: &mut String) {
        for p in parts {
            match p {
                WordPart::Lit(t) | WordPart::SQuoted(t) | WordPart::Escaped(t) => s.push_str(t),
                WordPart::DQuoted(inner) => rec(inner, s),
                other => other.write_source(s),
            }
        }
    }
    let mut s = String::new();
    rec(&w.parts, &mut s);
    s
}

/// Parse text as an arithmetic expression word.
pub fn parse_arith_text(text: &str, aliases: Option<Arc<HashMap<String, String>>>) -> PResult<Word> {
    let mut p = Parser::new(text).with_aliases(aliases);
    p.parse_operand_to_end()
}

/// Parse text as an operand (subscript etc.).
pub fn parse_operand_text(text: &str, aliases: Option<Arc<HashMap<String, String>>>) -> PResult<Word> {
    let mut p = Parser::new(text).with_aliases(aliases);
    p.parse_operand_to_end()
}

/// Parse an unquoted here-document body: only `$`, backquote and backslash are special.
pub fn parse_heredoc_text(text: &str, aliases: Option<Arc<HashMap<String, String>>>) -> PResult<Word> {
    let mut p = Parser::new(text).with_aliases(aliases);
    let mut parts = Vec::new();
    let mut lit = String::new();
    while let Some(c) = p.peek() {
        match c {
            '\\' => {
                p.bump();
                match p.peek() {
                    Some(n @ ('$' | '`' | '\\')) => {
                        p.bump();
                        lit.push(n);
                    }
                    Some('\n') => {
                        p.bump();
                    }
                    _ => lit.push('\\'),
                }
            }
            '$' => {
                if !lit.is_empty() {
                    parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                }
                match p.parse_dollar(true)? {
                    Some(x) => parts.push(x),
                    None => lit.push('$'),
                }
            }
            '`' => {
                if !lit.is_empty() {
                    parts.push(WordPart::Lit(std::mem::take(&mut lit)));
                }
                parts.push(p.parse_backtick(false)?);
            }
            _ => {
                lit.push(c);
                p.bump();
            }
        }
    }
    if !lit.is_empty() {
        parts.push(WordPart::Lit(lit));
    }
    Ok(Word { parts: vec![WordPart::DQuoted(parts)] })
}

/// Parse a word from a string as if it appeared unquoted on a command line (used by `read`-like
/// contexts that need a pattern, and by tests).
pub fn parse_single_word(text: &str) -> PResult<Word> {
    let mut p = Parser::new(text);
    Ok(p.parse_word()?.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Program {
        Parser::new(s).parse_program().unwrap()
    }

    #[test]
    fn basic() {
        let p = parse("echo hello world | wc -l && ls; x=1 y=2 cmd >out 2>&1 &");
        assert_eq!(p.items.len(), 2);
        assert!(p.items[1].background);
    }

    #[test]
    fn compound() {
        parse("if true; then echo a; elif false; then echo b; else echo c; fi");
        parse("for i in 1 2 3; do echo $i; done");
        parse("for ((i=0;i<3;i++)); do echo $i; done");
        parse("while read -r line; do echo \"$line\"; done < file");
        parse("case $x in a|b) echo ab;; *) echo other;; esac");
        parse("f() { echo hi; }; function g { echo g; }");
        parse("[[ -f x && $a == b* || ! -z $c ]]");
        parse("(( x = 1 + 2 ))");
        parse("echo $(case x in x) echo y;; esac)");
        parse("a=(1 2 [5]=x \"y z\")");
        parse("echo ${x:-a b} ${#y} ${z//a/b} ${arr[@]:1:2} ${!p*}");
        parse("cat <<EOF\nhello $x\nEOF\necho after");
        parse("[[ $x =~ ^(a|b)+$ ]]");
        parse("echo @(a|b) !(x)");
        parse("local a=(1 2) b=$x");
    }

    #[test]
    fn incomplete() {
        assert!(matches!(Parser::new("if true; then").parse_program(), Err(ParseError::Incomplete)));
        assert!(matches!(Parser::new("echo 'abc").parse_program(), Err(ParseError::Incomplete)));
        assert!(matches!(Parser::new("echo a |").parse_program(), Err(ParseError::Incomplete)));
    }
}
