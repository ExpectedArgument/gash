//! PS1/PS2/PS4 prompt expansion.

use crate::shell::*;

pub fn expand_prompt(sh: &mut Shell, ps: &str) -> String {
    let c: Vec<char> = ps.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch != '\\' || i + 1 >= c.len() {
            out.push(ch);
            i += 1;
            continue;
        }
        let e = c[i + 1];
        i += 2;
        match e {
            'a' => out.push('\x07'),
            'e' => out.push('\x1b'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            '\\' => out.push_str("\\\\"),
            '[' => out.push('\x01'),
            ']' => out.push('\x02'),
            '$' => out.push_str("\\$"),
            'u' => out.push_str(&sh.get_str("USER").unwrap_or_else(crate::sys::user_name)),
            'h' => {
                let h = sh.get_str("HOSTNAME").unwrap_or_else(crate::sys::host_name);
                out.push_str(h.split('.').next().unwrap_or(""));
            }
            'H' => out.push_str(&sh.get_str("HOSTNAME").unwrap_or_else(crate::sys::host_name)),
            'w' | 'W' => {
                let pwd = sh.get_str("PWD").unwrap_or_default();
                let home = sh.get_str("HOME").unwrap_or_default();
                let short = if !home.is_empty() && home != "/" && (pwd == home || pwd.starts_with(&format!("{}/", home))) {
                    format!("~{}", &pwd[home.len()..])
                } else {
                    pwd.clone()
                };
                let s = if e == 'W' {
                    if short == "~" || pwd == "/" {
                        short
                    } else {
                        pwd.rsplit('/').next().unwrap_or("").to_string()
                    }
                } else {
                    short
                };
                // escape so later expansion leaves it alone
                out.push_str(&s.replace('\\', "\\\\").replace('$', "\\$").replace('`', "\\`"));
            }
            's' => out.push_str(&sh.name),
            'v' => out.push_str("5.2"),
            'V' => out.push_str("5.2.37"),
            'j' => out.push_str(&sh.jobs.list.len().to_string()),
            'l' => out.push_str("cons0"),
            '!' => out.push_str(&(sh.history.lock().unwrap().len() + 1).to_string()),
            '#' => out.push_str(&(sh.history.lock().unwrap().len() + 1).to_string()),
            'd' => out.push_str(&crate::builtins_io::strftime("%a %b %d", crate::builtins_io::now_epoch())),
            't' => out.push_str(&crate::builtins_io::strftime("%H:%M:%S", crate::builtins_io::now_epoch())),
            'T' => out.push_str(&crate::builtins_io::strftime("%I:%M:%S", crate::builtins_io::now_epoch())),
            '@' => out.push_str(&crate::builtins_io::strftime("%I:%M %p", crate::builtins_io::now_epoch())),
            'A' => out.push_str(&crate::builtins_io::strftime("%H:%M", crate::builtins_io::now_epoch())),
            'D' => {
                if i < c.len() && c[i] == '{' {
                    let mut f = String::new();
                    i += 1;
                    while i < c.len() && c[i] != '}' {
                        f.push(c[i]);
                        i += 1;
                    }
                    i += 1;
                    let f = if f.is_empty() { "%X".to_string() } else { f };
                    out.push_str(&crate::builtins_io::strftime(&f, crate::builtins_io::now_epoch()));
                }
            }
            '0'..='7' => {
                let mut v = e.to_digit(8).unwrap();
                let mut n = 1;
                while n < 3 && i < c.len() && c[i].is_digit(8) {
                    v = v * 8 + c[i].to_digit(8).unwrap();
                    i += 1;
                    n += 1;
                }
                if let Some(ch) = char::from_u32(v) {
                    out.push(ch);
                }
            }
            other => {
                out.push('\\');
                out.push(other);
            }
        }
    }
    if sh.sh[sho::promptvars()] {
        let saved_status = sh.status;
        let saved_x = sh.o[O_XTRACE];
        sh.o[O_XTRACE] = false;
        let r = sh.expand_string(&out).unwrap_or(out);
        sh.o[O_XTRACE] = saved_x;
        sh.status = saved_status;
        r
    } else {
        out.replace("\\\\", "\\").replace("\\$", "$")
    }
}
