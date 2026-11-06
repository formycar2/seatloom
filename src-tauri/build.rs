// The build-embedded identity: the commit this binary was compiled from, written
// into the executable at build time.
//
// WHY THIS EXISTS. The sandbox banner's branch and commit are read from environment
// variables that the *launcher* sets. That makes them a statement by the installer
// about what it launched, not a statement by the binary about what it is. Measured
// consequence: `strings` on the sandbox binary finds the four env var NAMES and six
// hex-looking strings, none of which is a hash of any commit on the branch — so
// nothing in the binary can answer "what are you?".
//
// This is the smallest change that makes an acceptance falsifiable. An operator who
// clicks Accept and sees a merge cannot currently tell whether the guard refused for
// the right reason or the merge happened for the right reason, because the running
// code cannot state its own commit. With this, the banner's commit comes from inside
// the binary and can be compared against the commit the card names.
//
// WHAT IT DOES NOT CLAIM. A build-time capture describes the tree at compile time.
// It says nothing about the working directory of whoever launched it, and it is not
// a signature. If `git` is unavailable or the directory is not a repository, the
// capture is empty and the banner says "未记录" — the honest direction, and the same
// one `env_value` already takes.

use std::process::Command;

fn main() {
    // Rerun when the commit changes. A stale embedded hash is worse than none,
    // because it would make a binary claim to be a commit it is not.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs");

    let commit = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let dest = std::path::Path::new(&out_dir).join("build_identity.rs");
    // 🔴 `include!` 不能出现在表达式位置(`const X: T = include!(...)` 是语法错误,
    //    实测报 "expected expression, found keyword `pub`")。所以这里写的是一个**完整
    //    的模块文件**,由使用方 `include!` 在模块位置展开,再引用里面的常量。
    let body = match commit {
        Some(c) => format!(
            "/// The commit this binary was compiled from, captured by `build.rs`.\n\
             /// A literal: the value is compiled in, never read at runtime. A runtime `git`\n\
             /// call would make the banner depend on the launcher's working directory,\n\
             /// which is exactly what this replaces.\n\
             ///\n\
             /// 🔴 The label is not decoration. Measured: grepping a binary for a bare\n\
             /// 12-hex-digit run finds `dbdbdbdbdbdb` and other placeholders that are not\n\
             /// commits — Lyra measured six hex-looking strings in the sandbox binary,\n\
             /// none of them a commit. So the hash is emitted next to a fixed label, and\n\
             /// anything reading it must match the LABEL, never the bare hex.\n\
             pub const BUILD_COMMIT: Option<&str> = Some(\"seatloom-build-commit={c}\");\n"
        ),
        None => "/// No commit was captured (git unavailable, or not a repository).\n\
                  /// Stated as absent rather than guessed at.\n\
                  pub const BUILD_COMMIT: Option<&str> = None;\n"
            .to_string(),
    };
    let _ = std::fs::write(&dest, body);

    tauri_build::build()
}
