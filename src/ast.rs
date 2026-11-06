//! Abstract syntax tree for the shell language.

use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub parts: Vec<WordPart>,
}

impl Word {
    pub fn lit(s: &str) -> Word {
        Word { parts: vec![WordPart::Lit(s.to_string())] }
    }
    /// If the word consists solely of unquoted literal text, return it.
    pub fn as_plain(&self) -> Option<String> {
        let mut s = String::new();
        for p in &self.parts {
            match p {
                WordPart::Lit(t) => s.push_str(t),
                _ => return None,
            }
        }
        Some(s)
    }
    pub fn is_quoted(&self) -> bool {
        self.parts.iter().any(|p| matches!(p, WordPart::SQuoted(_) | WordPart::DQuoted(_) | WordPart::Escaped(_)))
    }
    /// Approximate source reconstruction (used for xtrace / error messages / `type`).
    pub fn to_source(&self) -> String {
        let mut s = String::new();
        for p in &self.parts {
            p.write_source(&mut s);
        }
        s
    }
}

#[derive(Debug, Clone)]
pub enum WordPart {
    /// Unquoted literal text (subject to brace expansion and globbing).
    Lit(String),
    /// Backslash-escaped character outside quotes.
    Escaped(String),
    /// Single-quoted or $'...' text.
    SQuoted(String),
    /// Double-quoted region.
    DQuoted(Vec<WordPart>),
    Param(Box<ParamExp>),
    CmdSubst(Arc<Program>),
    Backtick(Arc<Program>),
    Arith(Word),
    /// `~` or `~user` at word start (or after `:`/`=` in assignments).
    Tilde(String),
    ProcSubst { input: bool, prog: Arc<Program> },
    /// An assignment-shaped argument to a declaration builtin (`local x=(1 2)`, `export A=$b`).
    DeclAssign(Box<Assign>),
}

impl WordPart {
    pub fn write_source(&self, s: &mut String) {
        match self {
            WordPart::Lit(t) => s.push_str(t),
            WordPart::Escaped(t) => {
                s.push('\\');
                s.push_str(t)
            }
            WordPart::SQuoted(t) => {
                s.push('\'');
                s.push_str(&t.replace('\'', "'\\''"));
                s.push('\'');
            }
            WordPart::DQuoted(ps) => {
                s.push('"');
                for p in ps {
                    match p {
                        WordPart::Lit(t) => {
                            for c in t.chars() {
                                if matches!(c, '"' | '\\' | '$' | '`') {
                                    s.push('\\');
                                }
                                s.push(c);
                            }
                        }
                        other => other.write_source(s),
                    }
                }
                s.push('"');
            }
            WordPart::Param(p) => {
                s.push_str("${");
                s.push_str(&p.name);
                s.push('}');
            }
            WordPart::CmdSubst(_) => s.push_str("$(...)"),
            WordPart::Backtick(_) => s.push_str("`...`"),
            WordPart::Arith(w) => {
                s.push_str("$((");
                s.push_str(&w.to_source());
                s.push_str("))");
            }
            WordPart::Tilde(u) => {
                s.push('~');
                s.push_str(u)
            }
            WordPart::ProcSubst { input, .. } => s.push_str(if *input { "<(...)" } else { ">(...)" }),
            WordPart::DeclAssign(a) => {
                s.push_str(&a.name);
                s.push_str(if a.append { "+=" } else { "=" });
                match &a.value {
                    AssignValue::Scalar(w) => s.push_str(&w.to_source()),
                    AssignValue::Array(_) => s.push_str("(...)"),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamExp {
    /// Variable name or special parameter (`@`, `*`, `#`, `?`, `-`, `$`, `!`, `0`..`9`, digits).
    pub name: String,
    /// Subscript text for `name[...]` (raw word; evaluated arithmetically for indexed arrays).
    pub index: Option<Word>,
    /// `${!name}` indirection.
    pub indirect: bool,
    pub op: ParamOp,
}

#[derive(Debug, Clone)]
pub enum ParamOp {
    None,
    Length,
    /// `-`, `=`, `?`, `+` with optional colon
    Default { colon: bool, word: Word },
    Assign { colon: bool, word: Word },
    Error { colon: bool, word: Option<Word> },
    Alt { colon: bool, word: Word },
    TrimPrefix { longest: bool, pat: Word },
    TrimSuffix { longest: bool, pat: Word },
    Replace { mode: ReplMode, pat: Word, rep: Option<Word> },
    Substr { offset: Word, length: Option<Word> },
    Case { upper: bool, all: bool, pat: Option<Word> },
    /// `${x~}` / `${x~~}`: toggle case
    Toggle { all: bool, pat: Option<Word> },
    /// Unparseable expansion: reported as "bad substitution" when evaluated.
    Bad(String),
    Transform(char),
    /// `${!prefix*}` / `${!prefix@}`
    PrefixNames { star: bool },
    /// `${!arr[@]}` / `${!arr[*]}`
    Keys { star: bool },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplMode {
    First,
    All,
    Prefix,
    Suffix,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: String,
    pub index: Option<Word>,
    pub append: bool,
    pub value: AssignValue,
}

#[derive(Debug, Clone)]
pub enum AssignValue {
    Scalar(Word),
    /// `(a b [k]=v ...)`
    Array(Vec<(Option<Word>, Word)>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RedirOp {
    In,          // <
    Out,         // >
    Clobber,     // >|
    Append,      // >>
    ReadWrite,   // <>
    DupIn,       // <&
    DupOut,      // >&
    HereString,  // <<<
    HereDoc,     // << / <<-
    OutErr,      // &>
    AppendOutErr // &>>
}

#[derive(Debug)]
pub struct HereDocBody {
    pub quoted: bool,
    pub word: Word, // if quoted: single Lit/SQuoted; else parsed with $ ` \ active
}

#[derive(Debug, Clone)]
pub enum RedirTarget {
    Word(Word),
    HereDoc(Arc<OnceLock<HereDocBody>>),
}

#[derive(Debug, Clone)]
pub struct Redir {
    pub fd: Option<i32>,
    pub fd_var: Option<String>,
    pub op: RedirOp,
    pub target: RedirTarget,
}

#[derive(Debug, Clone, Default)]
pub struct SimpleCommand {
    pub assigns: Vec<Assign>,
    pub words: Vec<Word>,
    pub redirs: Vec<Redir>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CaseArm {
    pub patterns: Vec<Word>,
    pub body: List,
    pub term: CaseTerm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaseTerm {
    Break,       // ;;
    FallThrough, // ;&
    Continue,    // ;;&
}

#[derive(Debug, Clone)]
pub enum CondExpr {
    And(Box<CondExpr>, Box<CondExpr>),
    Or(Box<CondExpr>, Box<CondExpr>),
    Not(Box<CondExpr>),
    Unary(String, Word),
    Binary(Word, String, Word),
    Word(Word),
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub body: Command,
    pub source: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum CompoundKind {
    Subshell(List),
    Group(List),
    If { conds: Vec<(List, List)>, else_body: Option<List> },
    While { cond: List, body: List, until: bool },
    For { var: String, words: Option<Vec<Word>>, body: List },
    ArithFor { init: Option<Word>, cond: Option<Word>, step: Option<Word>, body: List },
    Select { var: String, words: Option<Vec<Word>>, body: List },
    Case { word: Word, arms: Vec<CaseArm> },
    Cond(CondExpr),
    Arith(Word),
}

#[derive(Debug, Clone)]
pub enum Command {
    Simple(SimpleCommand),
    Compound { kind: Box<CompoundKind>, redirs: Vec<Redir>, line: usize },
    FuncDef(Arc<FuncDef>),
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub negate: bool,
    pub time: bool,
    pub time_posix: bool,
    /// Each command and whether its stderr is also piped (`|&`).
    pub cmds: Vec<(Command, bool)>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct AndOr {
    pub first: Pipeline,
    pub rest: Vec<(AndOrOp, Pipeline)>,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub and_or: AndOr,
    pub background: bool,
    /// Source text of this item (for job display / history / `set -v`).
    pub text: String,
}

pub type List = Vec<ListItem>;

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub items: List,
}
