use std::{env, fs, path::PathBuf};

/// Install the man page so `man bahai-date` works after `cargo install`.
///
/// Cargo itself does not install man pages, so this best-effort copies
/// man/bahai-date.1 into the user man directories that `man` already
/// searches on common setups:
///   - $CARGO_HOME/share/man/man1  (in the default manpath on macOS configs
///     and easily added elsewhere)
///   - ~/.local/share/man/man1     (searched by default by man-db on Linux)
/// Failures are ignored: a read-only or unusual build environment must not
/// break compilation.
fn main() {
    println!("cargo:rerun-if-changed=man/bahai-date.1");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let man = fs::read_to_string(manifest_dir.join("man/bahai-date.1"))
        .expect("man/bahai-date.1 is missing from the repository");

    // Keep a copy next to the build artifacts for inspection and packaging.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let _ = fs::write(out_dir.join("bahai-date.1"), &man);

    let mut dirs: Vec<PathBuf> = Vec::new();
    let cargo_home = env::var("CARGO_HOME")
        .unwrap_or_else(|_| format!("{}/.cargo", env::var("HOME").unwrap_or_default()));
    if !cargo_home.is_empty() {
        dirs.push(PathBuf::from(cargo_home).join("share/man/man1"));
    }
    if let Ok(home) = env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/man/man1"));
    }

    for dir in dirs {
        if fs::create_dir_all(&dir).is_ok() {
            let _ = fs::write(dir.join("bahai-date.1"), &man);
        }
    }
}
