//! Embeds assets/gash.ico as the executable's icon on Windows (MSVC).
//!
//! Zero dependencies: rather than an .rc script and a resource compiler, this writes the
//! compiled resource format (.res) directly and passes it to the MSVC linker, which takes
//! .res files as ordinary inputs. Regenerate the .ico with `python assets/make_icon.py`.

use std::env;
use std::fs;
use std::path::PathBuf;

const ICO: &str = "assets/gash.ico";
const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;

fn main() {
    println!("cargo:rerun-if-changed={ICO}");
    println!("cargo:rerun-if-changed=build.rs");
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os != "windows" || target_env != "msvc" {
        return;
    }
    let ico = match fs::read(ICO) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("cargo:warning=not embedding an icon: {ICO}: {e}");
            return;
        }
    };
    let res = ico_to_res(&ico).unwrap_or_else(|| panic!("{ICO} is not a valid .ico file"));
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("gash-icon.res");
    fs::write(&out, res).unwrap();
    println!("cargo:rustc-link-arg-bins={}", out.display());
}

/// Turns an .ico file into a .res holding one RT_ICON per image plus the RT_GROUP_ICON
/// (id 1) that Explorer and the console window use.
fn ico_to_res(ico: &[u8]) -> Option<Vec<u8>> {
    let u16_at = |o: usize| Some(u16::from_le_bytes(ico.get(o..o + 2)?.try_into().ok()?));
    let u32_at = |o: usize| Some(u32::from_le_bytes(ico.get(o..o + 4)?.try_into().ok()?));
    if u16_at(0)? != 0 || u16_at(2)? != 1 {
        return None;
    }
    let count = u16_at(4)?;
    let mut res = Vec::new();
    entry(&mut res, 0, 0, 0, &[]); // every .res starts with an empty entry
    let mut group = Vec::new();
    for v in [0, 1, count] {
        group.extend(v.to_le_bytes()); // GRPICONDIR: reserved, type (1 = icon), count
    }
    for i in 0..count {
        let dir = 6 + 16 * i as usize; // ICONDIRENTRY
        let size = u32_at(dir + 8)? as usize;
        let offset = u32_at(dir + 12)? as usize;
        let image = ico.get(offset..offset.checked_add(size)?)?;
        let id = i + 1;
        entry(&mut res, RT_ICON, id, 0x1010, image); // MOVEABLE | DISCARDABLE
        // GRPICONDIRENTRY = the ICONDIRENTRY's first 12 bytes, then the resource id.
        group.extend_from_slice(ico.get(dir..dir + 12)?);
        group.extend(id.to_le_bytes());
    }
    entry(&mut res, RT_GROUP_ICON, 1, 0x1030, &group); // MOVEABLE | PURE | DISCARDABLE
    Some(res)
}

/// Appends one resource: a 32-byte header (numeric type and name, neutral language),
/// then the data, padded to a 4-byte boundary.
fn entry(res: &mut Vec<u8>, ty: u16, name: u16, flags: u16, data: &[u8]) {
    res.extend((data.len() as u32).to_le_bytes()); // DataSize
    res.extend(32u32.to_le_bytes()); // HeaderSize
    res.extend([0xFF, 0xFF]);
    res.extend(ty.to_le_bytes());
    res.extend([0xFF, 0xFF]);
    res.extend(name.to_le_bytes());
    res.extend(0u32.to_le_bytes()); // DataVersion
    res.extend(flags.to_le_bytes()); // MemoryFlags
    res.extend(0u16.to_le_bytes()); // LanguageId: neutral
    res.extend(0u32.to_le_bytes()); // Version
    res.extend(0u32.to_le_bytes()); // Characteristics
    res.extend_from_slice(data);
    while res.len() % 4 != 0 {
        res.push(0);
    }
}
