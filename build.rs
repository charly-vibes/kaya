//! Pin the projection cache's generator identity to the render sources.
//!
//! The incremental skip predicate refuses to restore a contribution unless
//! the stored state's generator string equals this build's. Hashing the
//! projection sources makes that string change with any render-affecting
//! edit, so a stale cache can never skip a document the new code would
//! render differently — no manual bump to remember.

use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/projection");
    // FNV-1a 64-bit: deterministic across machines, no build dependency.
    let mut hash = 0xcbf29ce484222325u64;
    let mut paths = collect("src/projection");
    paths.sort();
    for path in &paths {
        let bytes = fs::read(path).expect("projection source is readable");
        mix_path(&mut hash, path);
        for byte in &bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    println!("cargo:rustc-env=RENDER_HASH={hash:016x}");
}

/// Fold one path's text into the hash, NUL-terminated, so file boundaries
/// and renames both register.
fn mix_path(hash: &mut u64, path: &str) {
    for byte in path.as_bytes().iter().chain(std::iter::once(&0u8)) {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

/// Every `.rs` file under `dir`, recursively, as sorted-stable strings.
fn collect(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(Path::new(dir)).expect("projection directory exists") {
        let path = entry.expect("entry is readable").path();
        if path.is_dir() {
            let nested = path.to_str().expect("utf-8 path").to_owned();
            out.extend(collect(&nested));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path.to_str().expect("utf-8 path").to_owned());
        }
    }
    out
}
