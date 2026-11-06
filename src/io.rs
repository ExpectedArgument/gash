//! File-descriptor table and handle abstraction. The shell never relies on process-wide
//! stdio state, so subshells can run in threads with their own fd tables.

use crate::sys;
use std::fs::File;
use std::io::{self, BufRead, PipeReader, PipeWriter, Read, Seek, SeekFrom, Write};
use std::mem::ManuallyDrop;
use std::os::windows::io::{AsHandle, AsRawHandle, FromRawHandle};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Debug, Default)]
pub struct MemReader {
    pub data: Vec<u8>,
    pub pos: usize,
}

#[derive(Clone)]
pub enum Handle {
    Stdin,
    Stdout,
    Stderr,
    File(Arc<File>),
    PipeR(Arc<PipeReader>),
    PipeW(Arc<PipeWriter>),
    Buf(Arc<Mutex<Vec<u8>>>),
    Mem(Arc<Mutex<MemReader>>),
    Null,
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Handle::Stdin => "stdin",
            Handle::Stdout => "stdout",
            Handle::Stderr => "stderr",
            Handle::File(_) => "file",
            Handle::PipeR(_) => "pipe-r",
            Handle::PipeW(_) => "pipe-w",
            Handle::Buf(_) => "buf",
            Handle::Mem(_) => "mem",
            Handle::Null => "null",
        };
        f.write_str(s)
    }
}

fn raw_std(n: sys::DWORD) -> ManuallyDrop<File> {
    unsafe { ManuallyDrop::new(File::from_raw_handle(sys::std_handle(n) as _)) }
}

pub fn console_stdin() -> bool {
    sys::is_console(sys::STD_INPUT_HANDLE)
}

/// Write bytes to the process stdout/stderr. Console output goes through Rust's std which
/// converts UTF-8 to UTF-16; invalid UTF-8 is written lossily.
fn write_std(err: bool, data: &[u8]) -> io::Result<()> {
    let r = if err {
        let mut e = io::stderr().lock();
        e.write_all(data).and_then(|_| e.flush())
    } else {
        let mut o = io::stdout().lock();
        o.write_all(data).and_then(|_| o.flush())
    };
    match r {
        Err(e) if e.kind() == io::ErrorKind::InvalidData => {
            let s = String::from_utf8_lossy(data).into_owned();
            if err {
                let mut e = io::stderr().lock();
                e.write_all(s.as_bytes())?;
                e.flush()
            } else {
                let mut o = io::stdout().lock();
                o.write_all(s.as_bytes())?;
                o.flush()
            }
        }
        r => r,
    }
}

impl Handle {
    pub fn write_all(&self, data: &[u8]) -> io::Result<()> {
        match self {
            Handle::Stdout => write_std(false, data),
            Handle::Stderr => write_std(true, data),
            Handle::File(f) => (&**f).write_all(data),
            Handle::PipeW(p) => (&**p).write_all(data),
            Handle::Buf(b) => {
                b.lock().unwrap().extend_from_slice(data);
                Ok(())
            }
            Handle::Null => Ok(()),
            Handle::Stdin | Handle::PipeR(_) | Handle::Mem(_) => {
                Err(io::Error::new(io::ErrorKind::Other, "Bad file descriptor"))
            }
        }
    }

    pub fn is_console(&self) -> bool {
        match self {
            Handle::Stdin => sys::is_console(sys::STD_INPUT_HANDLE),
            Handle::Stdout => sys::is_console(sys::STD_OUTPUT_HANDLE),
            Handle::Stderr => sys::is_console(sys::STD_ERROR_HANDLE),
            _ => false,
        }
    }

    pub fn readable(&self) -> bool {
        matches!(self, Handle::Stdin | Handle::File(_) | Handle::PipeR(_) | Handle::Mem(_) | Handle::Null)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Handle::Stdin => {
                if console_stdin() {
                    io::stdin().lock().read(buf)
                } else {
                    let f = raw_std(sys::STD_INPUT_HANDLE);
                    match (&*f).read(buf) {
                        Err(e) if e.raw_os_error() == Some(109) => Ok(0),
                        r => r,
                    }
                }
            }
            Handle::File(f) => (&**f).read(buf),
            Handle::PipeR(p) => match (&**p).read(buf) {
                Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(0),
                r => r,
            },
            Handle::Mem(m) => {
                let mut m = m.lock().unwrap();
                let n = buf.len().min(m.data.len() - m.pos);
                let p = m.pos;
                buf[..n].copy_from_slice(&m.data[p..p + n]);
                m.pos += n;
                Ok(n)
            }
            Handle::Null => Ok(0),
            _ => Err(io::Error::new(io::ErrorKind::Other, "Bad file descriptor")),
        }
    }

    pub fn read_to_end(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut buf = vec![0u8; 65536];
        loop {
            let n = self.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
        }
        Ok(out)
    }

    /// Read up to and including `delim` without consuming input beyond it (so that
    /// child processes sharing the handle see the rest). Returns (data without delim, found_delim).
    pub fn read_until(&self, delim: u8, max: Option<usize>) -> io::Result<(Vec<u8>, bool)> {
        let mut out = Vec::new();
        let limit = max.unwrap_or(usize::MAX);
        match self {
            Handle::Mem(m) => {
                let mut m = m.lock().unwrap();
                while m.pos < m.data.len() && out.len() < limit {
                    let b = m.data[m.pos];
                    m.pos += 1;
                    if b == delim {
                        return Ok((out, true));
                    }
                    out.push(b);
                }
                Ok((out, false))
            }
            Handle::File(f) => read_until_seekable(&f, delim, limit),
            Handle::Stdin if console_stdin() => {
                let stdin = io::stdin();
                let mut lock = stdin.lock();
                if max.is_none() {
                    let n = lock.read_until(delim, &mut out)?;
                    if n > 0 && out.last() == Some(&delim) {
                        out.pop();
                        return Ok((out, true));
                    }
                    return Ok((out, false));
                }
                let mut b = [0u8; 1];
                while out.len() < limit {
                    if lock.read(&mut b)? == 0 {
                        return Ok((out, false));
                    }
                    if b[0] == delim {
                        return Ok((out, true));
                    }
                    out.push(b[0]);
                }
                Ok((out, false))
            }
            Handle::Stdin if sys::file_type(sys::STD_INPUT_HANDLE) == sys::FILE_TYPE_DISK => {
                let f = raw_std(sys::STD_INPUT_HANDLE);
                read_until_seekable(&f, delim, limit)
            }
            _ => {
                let mut b = [0u8; 1];
                while out.len() < limit {
                    if self.read(&mut b)? == 0 {
                        return Ok((out, false));
                    }
                    if b[0] == delim {
                        return Ok((out, true));
                    }
                    out.push(b[0]);
                }
                Ok((out, false))
            }
        }
    }

    /// Convert to a Stdio for a child process (`msys`: an MSYS2/Cygwin program). Returns an
    /// optional helper thread (pump) that must be joined after the child exits.
    pub fn to_stdio(&self, fd: i32, msys: bool) -> io::Result<(Stdio, Option<JoinHandle<()>>)> {
        if msys {
            if let Some(f) = self.msys_writable() {
                return Ok((Stdio::from(f), None));
            }
        }
        Ok(match self {
            Handle::Stdin => {
                if fd == 0 {
                    (Stdio::inherit(), None)
                } else {
                    (dup_std(io::stdin().as_handle())?, None)
                }
            }
            Handle::Stdout => {
                if fd == 1 {
                    (Stdio::inherit(), None)
                } else {
                    (dup_std(io::stdout().as_handle())?, None)
                }
            }
            Handle::Stderr => {
                if fd == 2 {
                    (Stdio::inherit(), None)
                } else {
                    (dup_std(io::stderr().as_handle())?, None)
                }
            }
            Handle::File(f) => (Stdio::from(f.try_clone()?), None),
            Handle::PipeR(p) => (Stdio::from(p.try_clone()?), None),
            Handle::PipeW(p) => (Stdio::from(p.try_clone()?), None),
            Handle::Null => (Stdio::null(), None),
            Handle::Buf(b) => {
                let (mut r, w) = io::pipe()?;
                let b = b.clone();
                let h = std::thread::spawn(move || {
                    let mut tmp = vec![0u8; 65536];
                    loop {
                        match r.read(&mut tmp) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => b.lock().unwrap().extend_from_slice(&tmp[..n]),
                        }
                    }
                });
                (Stdio::from(w), Some(h))
            }
            Handle::Mem(m) => {
                let data = {
                    let mut m = m.lock().unwrap();
                    let d = m.data[m.pos..].to_vec();
                    m.pos = m.data.len();
                    d
                };
                let (r, mut w) = io::pipe()?;
                let h = std::thread::spawn(move || {
                    let _ = w.write_all(&data);
                });
                (Stdio::from(r), Some(h))
            }
        })
    }
}

impl Handle {
    /// MSYS programs can't write through an append-only handle, which is how `>>` opens files
    /// (see `sys::reopen_append_only_writable`). For such a file, a second handle that can
    /// write, positioned at the end of the file. The MSYS program then writes from there on,
    /// not always at the current end as with O_APPEND, so another process appending to the
    /// same file at the same time can overwrite its output or have its output overwritten.
    fn msys_writable(&self) -> Option<File> {
        let h = match self {
            Handle::Stdin => sys::std_handle(sys::STD_INPUT_HANDLE),
            Handle::Stdout => sys::std_handle(sys::STD_OUTPUT_HANDLE),
            Handle::Stderr => sys::std_handle(sys::STD_ERROR_HANDLE),
            Handle::File(f) => f.as_raw_handle() as sys::HANDLE,
            _ => return None,
        };
        let nh = sys::reopen_append_only_writable(h)?;
        let mut f = unsafe { File::from_raw_handle(nh as _) };
        f.seek(SeekFrom::End(0)).ok()?;
        Some(f)
    }
}

fn dup_std(h: std::os::windows::io::BorrowedHandle<'_>) -> io::Result<Stdio> {
    match h.try_clone_to_owned() {
        Ok(o) => Ok(Stdio::from(o)),
        Err(_) => Ok(Stdio::null()),
    }
}

fn read_until_seekable(f: &File, delim: u8, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut f = f;
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let want = chunk.len().min(limit.saturating_sub(out.len()).max(1));
        let n = f.read(&mut chunk[..want])?;
        if n == 0 {
            return Ok((out, false));
        }
        if let Some(i) = chunk[..n].iter().position(|&b| b == delim) {
            out.extend_from_slice(&chunk[..i]);
            let back = (n - i - 1) as i64;
            if back > 0 {
                f.seek(SeekFrom::Current(-back))?;
            }
            return Ok((out, true));
        }
        let take = n.min(limit - out.len());
        out.extend_from_slice(&chunk[..take]);
        if take < n {
            f.seek(SeekFrom::Current(-((n - take) as i64)))?;
        }
        if out.len() >= limit {
            return Ok((out, false));
        }
    }
}

#[derive(Clone)]
pub struct Fds {
    pub v: Vec<Option<Handle>>,
}

impl Fds {
    pub fn std() -> Fds {
        Fds { v: vec![Some(Handle::Stdin), Some(Handle::Stdout), Some(Handle::Stderr)] }
    }
    pub fn get(&self, fd: i32) -> Option<&Handle> {
        if fd < 0 {
            return None;
        }
        self.v.get(fd as usize).and_then(|h| h.as_ref())
    }
    pub fn set(&mut self, fd: i32, h: Option<Handle>) -> Option<Handle> {
        let i = fd as usize;
        if self.v.len() <= i {
            self.v.resize(i + 1, None);
        }
        std::mem::replace(&mut self.v[i], h)
    }
    pub fn free_fd(&self, from: i32) -> i32 {
        let mut fd = from;
        while self.get(fd).is_some() {
            fd += 1;
        }
        fd
    }
}
