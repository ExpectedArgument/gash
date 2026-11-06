//! Optional use of Microsoft's native "Coreutils for Windows" (github.com/microsoft/coreutils)
//! in place of Git for Windows' MSYS2 coreutils.
//!
//! When a command name resolves (via PATH) to one of Git's MSYS tools in `/usr/bin` and
//! Microsoft Coreutils provides the same utility, gash runs the native one instead. Because
//! the caller asked for an MSYS tool, gash translates MSYS paths in the arguments itself
//! (regardless of `MSYS_NO_PATHCONV`), using a per-utility policy so that strings such as
//! `tr` sets or `grep` patterns are never mistaken for paths.
//!
//! Located via `GASH_COREUTILS` (a path to coreutils.exe or its folder; `0` disables), else
//! `<gash.exe folder>\coreutils\coreutils.exe`, else `%ProgramFiles%\coreutils\coreutils.exe`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArgPolicy {
    /// Operands and `--opt=/path` values that look like MSYS paths are converted.
    Files,
    /// Arguments are plain strings: never converted.
    Strings,
    /// grep: the pattern operand (and `-e`) is a string; files (and `-f`) are converted.
    Grep,
}

pub struct Coreutils {
    pub exe: PathBuf,
    /// Folder of per-utility hard links (`cat.exe` -> coreutils.exe).
    pub links: PathBuf,
}

static CU: OnceLock<Option<Coreutils>> = OnceLock::new();

pub fn get() -> Option<&'static Coreutils> {
    CU.get_or_init(detect).as_ref()
}

fn detect() -> Option<Coreutils> {
    let setting = std::env::var("GASH_COREUTILS").unwrap_or_default();
    match setting.to_ascii_lowercase().as_str() {
        "0" | "off" | "false" | "no" => return None,
        "" | "1" | "on" | "true" | "yes" => {}
        _ => {
            let p = PathBuf::from(&setting);
            let exe = if p.is_dir() { p.join("coreutils.exe") } else { p };
            return if exe.is_file() { Some(make(exe)) } else { None };
        }
    }
    let mut cands = Vec::new();
    if let Ok(me) = std::env::current_exe() {
        if let Some(d) = me.parent() {
            cands.push(d.join("coreutils").join("coreutils.exe"));
        }
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        cands.push(PathBuf::from(pf).join("coreutils").join("coreutils.exe"));
    }
    cands.into_iter().find(|p| p.is_file()).map(make)
}

fn make(exe: PathBuf) -> Coreutils {
    let links = exe.parent().map(|d| d.join("bin")).unwrap_or_default();
    Coreutils { exe, links }
}

/// Utilities that gash routes to Microsoft Coreutils, with their argument policy.
///
/// Deliberately not routed (Git's GNU versions are kept):
/// `sort`, `find` (ambiguous with DOS commands; default to DOS behaviour without an
/// HKLM setting), `mktemp`, `realpath`, `readlink` (print Windows paths that scripts then
/// manipulate), `ln` (native symlinks need Developer Mode), `env`, `xargs` (can't start
/// shell scripts), and anything that is a gash builtin anyway (echo, printf, test, pwd, ...).
pub fn policy(util: &str) -> Option<ArgPolicy> {
    use ArgPolicy::*;
    Some(match util {
        "grep" => Grep,
        "basename" | "dirname" | "expr" | "seq" | "tr" | "yes" | "factor" | "numfmt" | "sleep" | "printenv"
        | "pathchk" | "arch" | "nproc" | "hostname" | "uptime" | "date" => Strings,
        "b2sum" | "base32" | "base64" | "basenc" | "cat" | "cksum" | "comm" | "cp" | "csplit" | "cut" | "df" | "du"
        | "fmt" | "fold" | "head" | "join" | "link" | "ls" | "md5sum" | "mkdir" | "mv" | "nl" | "od" | "paste" | "pr"
        | "ptx" | "rm" | "rmdir" | "sha1sum" | "sha224sum" | "sha256sum" | "sha384sum" | "sha512sum" | "shuf"
        | "split" | "stat" | "sum" | "tac" | "tail" | "tee" | "touch" | "truncate" | "tsort" | "unexpand" | "uniq"
        | "unlink" | "wc" => Files,
        _ => return None,
    })
}

impl Coreutils {
    /// Executable to run for `util`: a hard link named after it (created or refreshed on
    /// demand), or coreutils.exe itself if links can't be made (then pass `util` first).
    pub fn program(&self, util: &str) -> PathBuf {
        let link = self.links.join(format!("{}.exe", util));
        if same_file_data(&link, &self.exe) {
            return link;
        }
        let _ = std::fs::create_dir_all(&self.links);
        let _ = std::fs::remove_file(&link);
        if std::fs::hard_link(&self.exe, &link).is_ok() {
            return link;
        }
        self.exe.clone()
    }

    /// Is `p` one of our per-utility links? Returns the utility name.
    pub fn util_of(&self, p: &Path) -> Option<&'static str> {
        let parent = p.parent()?;
        if !parent.as_os_str().eq_ignore_ascii_case(self.links.as_os_str()) {
            return None;
        }
        let stem = p.file_stem()?.to_str()?.to_ascii_lowercase();
        static_name(&stem)
    }
}

/// Map a utility name to a `'static` str if it's routable.
pub fn static_name(name: &str) -> Option<&'static str> {
    const ALL: &[&str] = &[
        "grep", "basename", "dirname", "expr", "seq", "tr", "yes", "factor", "numfmt", "sleep", "printenv", "pathchk",
        "arch", "nproc", "hostname", "uptime", "date", "b2sum", "base32", "base64", "basenc", "cat", "cksum", "comm", "cp",
        "csplit", "cut", "df", "du", "fmt", "fold", "head", "join", "link", "ls", "md5sum", "mkdir", "mv", "nl", "od",
        "paste", "pr", "ptx", "rm", "rmdir", "sha1sum", "sha224sum", "sha256sum", "sha384sum", "sha512sum", "shuf",
        "split", "stat", "sum", "tac", "tail", "tee", "touch", "truncate", "tsort", "unexpand", "uniq", "unlink", "wc",
    ];
    ALL.iter().copied().find(|u| *u == name)
}

fn same_file_data(a: &Path, b: &Path) -> bool {
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(x), Ok(y)) => x.len() == y.len() && x.modified().ok() == y.modified().ok(),
        _ => false,
    }
}

/// Translate MSYS paths in `args` for a native utility according to its policy.
pub fn translate_args(policy: ArgPolicy, args: &[String]) -> Vec<String> {
    let conv = |a: &str| crate::path::convert_arg(a).into_owned();
    let conv_opt_value = |a: &str| -> String {
        // --opt=/path
        match a.find('=') {
            Some(i) if a.starts_with("--") => format!("{}{}", &a[..=i], conv(&a[i + 1..])),
            _ => a.to_string(),
        }
    };
    match policy {
        ArgPolicy::Strings => args.to_vec(),
        ArgPolicy::Files => {
            let mut out = Vec::with_capacity(args.len());
            let mut operands_only = false;
            for a in args {
                if !operands_only && a == "--" {
                    operands_only = true;
                    out.push(a.clone());
                } else if !operands_only && a.starts_with("--") {
                    out.push(conv_opt_value(a));
                } else if !operands_only && a.starts_with('-') && a.len() > 1 {
                    out.push(a.clone());
                } else {
                    out.push(conv(a));
                }
            }
            out
        }
        ArgPolicy::Grep => {
            // options whose value is a separate argument
            const SHORT_WITH_VALUE: &str = "efmABCdD";
            const LONG_WITH_VALUE: &[&str] = &[
                "--regexp", "--file", "--max-count", "--after-context", "--before-context", "--context", "--devices",
                "--directories", "--label", "--include", "--exclude", "--exclude-dir", "--exclude-from", "--binary-files",
                "--group-separator",
            ];
            let mut out = Vec::with_capacity(args.len());
            let mut pattern_given = args.iter().any(|a| {
                a == "-e" || a == "-f" || a.starts_with("--regexp") || a.starts_with("--file") || {
                    // -e/-f inside a short cluster like -ie or -rf
                    a.starts_with('-') && !a.starts_with("--") && a[1..].chars().any(|c| c == 'e' || c == 'f')
                }
            });
            let mut operands_only = false;
            let mut i = 0;
            while i < args.len() {
                let a = &args[i];
                if !operands_only && a == "--" {
                    operands_only = true;
                    out.push(a.clone());
                } else if !operands_only && a.starts_with("--") {
                    let name = a.split('=').next().unwrap_or(a);
                    let is_file = name == "--file" || name == "--exclude-from";
                    if a.contains('=') {
                        out.push(if is_file { conv_opt_value(a) } else { a.clone() });
                    } else {
                        out.push(a.clone());
                        if LONG_WITH_VALUE.contains(&name) && i + 1 < args.len() {
                            i += 1;
                            out.push(if is_file { conv(&args[i]) } else { args[i].clone() });
                        }
                    }
                } else if !operands_only && a.starts_with('-') && a.len() > 1 {
                    // short cluster; the first value-taking option consumes the rest or the next arg
                    let chars: Vec<char> = a[1..].chars().collect();
                    let mut consumed_next = None;
                    for (k, c) in chars.iter().enumerate() {
                        if SHORT_WITH_VALUE.contains(*c) {
                            if k + 1 == chars.len() && i + 1 < args.len() {
                                consumed_next = Some(*c);
                            }
                            break;
                        }
                    }
                    out.push(a.clone());
                    if let Some(c) = consumed_next {
                        i += 1;
                        out.push(if c == 'f' { conv(&args[i]) } else { args[i].clone() });
                    }
                } else if !pattern_given {
                    pattern_given = true;
                    out.push(a.clone());
                } else {
                    out.push(conv(a));
                }
                i += 1;
            }
            out
        }
    }
}

/// Quote an argument with MSVCRT rules. Quoted arguments are never re-globbed by
/// programs that expand wildcards themselves (such as Microsoft Coreutils via `wild`).
pub fn msvc_quote(a: &str) -> String {
    let mut s = String::with_capacity(a.len() + 2);
    s.push('"');
    let mut backslashes = 0;
    for c in a.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                s.extend(std::iter::repeat('\\').take(backslashes * 2 + 1));
                s.push('"');
                backslashes = 0;
            }
            c => {
                s.extend(std::iter::repeat('\\').take(backslashes));
                s.push(c);
                backslashes = 0;
            }
        }
    }
    s.extend(std::iter::repeat('\\').take(backslashes * 2));
    s.push('"');
    s
}

/// Does this argument contain characters a self-globbing native program would expand?
pub fn has_glob_chars(a: &str) -> bool {
    a.contains(['*', '?', '['])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn quoting() {
        assert_eq!(msvc_quote("a b"), "\"a b\"");
        assert_eq!(msvc_quote("*.txt"), "\"*.txt\"");
        assert_eq!(msvc_quote("say \"hi\""), "\"say \\\"hi\\\"\"");
        assert_eq!(msvc_quote(r"C:\dir\"), "\"C:\\dir\\\\\"");
        assert_eq!(msvc_quote(""), "\"\"");
    }

    #[test]
    fn grep_pattern_untouched() {
        crate::path::init();
        // the pattern looks like a path but must not be converted; the file must be
        let out = translate_args(ArgPolicy::Grep, &s(&["-rn", "/usr/bin", "/c/Windows/win.ini"]));
        assert_eq!(out[1], "/usr/bin");
        assert_eq!(out[2], "C:/Windows/win.ini");
        let out = translate_args(ArgPolicy::Grep, &s(&["-e", "/tmp/x", "/c/f"]));
        assert_eq!(out[1], "/tmp/x");
        assert_eq!(out[2], "C:/f");
        let out = translate_args(ArgPolicy::Grep, &s(&["-f", "/c/pats", "/c/f"]));
        assert_eq!(out[1], "C:/pats");
        assert_eq!(out[2], "C:/f");
        let out = translate_args(ArgPolicy::Grep, &s(&["-A", "2", "pat", "/c/f"]));
        assert_eq!(out, s(&["-A", "2", "pat", "C:/f"]));
    }

    #[test]
    fn files_and_strings() {
        crate::path::init();
        assert_eq!(translate_args(ArgPolicy::Files, &s(&["-n", "5", "/c/x/y"])), s(&["-n", "5", "C:/x/y"]));
        assert_eq!(translate_args(ArgPolicy::Files, &s(&["-d", "/", "rel/path"])), s(&["-d", "/", "rel/path"]));
        assert_eq!(translate_args(ArgPolicy::Strings, &s(&["/c/x"])), s(&["/c/x"]));
    }
}
