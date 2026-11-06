//! MSYS programs that gash starts must resolve `/tmp` to the same folder gash does.
//!
//! The MSYS runtime builds its `/tmp` mount (`usertemp` in /etc/fstab) from the Windows
//! TMP/TEMP of the first MSYS process of an installation, and keeps the mount table in
//! per-user shared memory for every MSYS process that starts while one is still running.
//! Against Git's own `/usr/bin` the outcome therefore depends on whether, say, a Git Bash
//! window is open. This test copies the runtime and a few tools into a private root, which
//! MSYS treats as a separate installation, so the program gash starts is always the one
//! that sets up the mount table.
//!
//! Skipped when Git for Windows is not installed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const RUNTIME: &[&str] = &[
    "msys-2.0.dll",
    "msys-iconv-2.dll",
    "msys-intl-8.dll",
    "cat.exe",
    "cygpath.exe",
    "mktemp.exe",
];

const FSTAB: &str = "none / cygdrive binary,posix=0,noacl,user 0 0\n\
                     none /tmp usertemp binary,posix=0,noacl 0 0\n";

const SCRIPT: &str = r#"
echo made-by-gash > /tmp/from-gash
/usr/bin/cat /tmp/from-gash
/usr/bin/cat <(echo procsub)
d=$(/usr/bin/mktemp -d) && cd "$d" && echo cd-ok
/usr/bin/cygpath -w /tmp
"#;

fn git_usr_bin() -> Option<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(r) = std::env::var("GASH_ROOT") {
        roots.push(PathBuf::from(r));
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        roots.push(Path::new(&pf).join("Git"));
    }
    roots.push(PathBuf::from(r"C:\Program Files\Git"));
    roots
        .into_iter()
        .map(|r| r.join("usr").join("bin"))
        .find(|b| RUNTIME.iter().all(|f| b.join(f).is_file()))
}

/// Removes the scratch folder even when an assertion fails.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn msys_children_share_gash_tmp() {
    let Some(git_bin) = git_usr_bin() else {
        eprintln!("skipping: Git for Windows' MSYS runtime not found");
        return;
    };
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let scratch = Scratch(std::env::temp_dir().join(format!("gash-msys-tmp-{}-{nanos}", std::process::id())));
    let root = scratch.0.join("root");
    let temp = scratch.0.join("temp");
    let bin = root.join("usr").join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(root.join("etc")).unwrap();
    fs::create_dir_all(&temp).unwrap();
    for f in RUNTIME {
        fs::copy(git_bin.join(f), bin.join(f)).unwrap_or_else(|e| panic!("copy {f}: {e}"));
    }
    fs::write(root.join("etc").join("fstab"), FSTAB).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_gash"))
        .arg("-c")
        .arg(SCRIPT)
        .current_dir(&scratch.0)
        .env("GASH_ROOT", &root)
        .env("TEMP", &temp)
        .env("TMP", &temp)
        .env("GASH_COREUTILS", "0")
        .stdin(Stdio::null())
        .output()
        .expect("run gash");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let context = format!("exit: {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}", out.status.code());

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4, "{context}");
    assert_eq!(lines[..3], ["made-by-gash", "procsub", "cd-ok"], "{context}");
    let msys_tmp = fs::canonicalize(lines[3].trim()).unwrap_or_else(|e| panic!("{e}: {}\n{context}", lines[3]));
    assert_eq!(msys_tmp, fs::canonicalize(&temp).unwrap(), "MSYS /tmp differs from gash's\n{context}");
    assert_eq!(out.status.code(), Some(0), "{context}");
}
