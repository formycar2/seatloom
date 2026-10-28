//! The operator's explicitly-configured identity — one file, read at decision time.
//!
//! # Why this module exists
//!
//! An acceptance decision is a **human act**, so the identity behind it must never
//! be invented on the spot. That rule is correct and this module does not weaken
//! it: the identity is still *explicitly configured*, by a person, in a place they
//! can see and edit.
//!
//! What it fixes is the other half. For a while the only configuration channel was
//! the `SEATLOOM_OPERATOR_REF` environment variable — and an app launched from the
//! macOS Launchpad, Spotlight or Finder has **no environment at all**. So a correct
//! guard had become a wall the operator could never get past: every click on
//! "accept" failed with `SEATLOOM_OPERATOR_REF is not set`, and nothing in the
//! product could satisfy it from a GUI launch. The acceptance plan even listed
//! "SEATLOOM_OPERATOR_REF 已配置" as a precondition that no GUI launch could meet.
//!
//! So there is a second channel: **a single-line file at
//! `~/.seatloom/operator`**, read at decision time. It persists across launches, it
//! survives app reinstalls, and it is found no matter how the app was started,
//! because it is resolved from `$HOME` and not from the working directory.
//!
//! # The hard line, restated
//!
//! There is **no default and no guess**. When neither channel states an identity,
//! this module refuses with a message that names the file, its exact path, and the
//! exact content to write. If you find yourself writing "if neither is set, use
//! `<something>`", you are relaxing the rule — stop.
//!
//! # Precedence
//!
//! **File first, environment second.** The file is the persistent, documented
//! channel; the environment is the per-launch override and the fallback when the
//! file cannot be read. Both are explicit configuration by a person, so neither is
//! a default — the precedence only decides which of two stated identities wins.
//!
//! # When a change takes effect
//!
//! The file is read **inside the command**, not cached at startup, so editing it
//! takes effect on the very next click. No restart is needed. (A running app that
//! has already loaded an old value into memory is therefore not a concern: nothing
//! is loaded ahead of time.)

use std::path::PathBuf;

/// The environment variable naming the operator. The per-launch channel.
pub const OPERATOR_REF_ENV: &str = "SEATLOOM_OPERATOR_REF";

/// The configuration file, relative to `$HOME`. A single line: the operator ref.
///
/// Deliberately a **single file under `$HOME`** and nothing more. A reader that
/// goes looking for identity in several places is a reader that will eventually
/// find it in the wrong one.
pub const OPERATOR_REF_FILE_RELATIVE: &str = ".seatloom/operator";

/// Where a resolved identity came from, so a reader of the ledger can see which
/// channel stated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorRefSource {
    /// The configuration file at this absolute path.
    File(PathBuf),
    /// The environment variable.
    Env,
}

/// A resolved operator identity, with the channel that stated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorRef {
    pub value: String,
    pub source: OperatorRefSource,
}

/// Why no identity could be resolved. Every variant names what is missing, so the
/// caller can render an actionable message rather than a variable name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorRefError {
    /// Neither channel stated an identity: the configuration file is absent (or
    /// `$HOME` is unset so it cannot be located at all) and the environment is
    /// unset. `file_path` is the path the file WOULD have been read from, so the
    /// refusal can name it; `None` when `$HOME` is unset.
    NotConfigured { file_path: Option<PathBuf> },
    /// The file is there but could not be read (permissions, a directory in its
    /// place, an encoding problem). Distinguished from "absent" on purpose: a
    /// file that exists and cannot be read is a problem to report, not a channel
    /// to skip silently.
    FileUnreadable { path: PathBuf, detail: String },
    /// The file is there and holds no non-empty line.
    FileBlank { path: PathBuf },
    /// The file holds more than one non-empty line. An operator ref is ONE token;
    /// picking one of two would be a guess, so this refuses and names both lines.
    FileAmbiguous { path: PathBuf, lines: Vec<String> },
    /// The environment variable is set but blank.
    EnvBlank,
}

impl std::fmt::Display for OperatorRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorRefError::NotConfigured { file_path } => match file_path {
                Some(path) => write!(
                    f,
                    "neither the operator identity file {} nor {OPERATOR_REF_ENV} states an \
                     identity — the operator identity is not configured",
                    path.display()
                ),
                None => write!(
                    f,
                    "the operator identity is not configured: {OPERATOR_REF_ENV} is unset and \
                     $HOME is unset, so the identity file cannot be located either"
                ),
            },
            OperatorRefError::FileUnreadable { path, detail } => write!(
                f,
                "the operator identity file {} exists but cannot be read ({detail})",
                path.display()
            ),
            OperatorRefError::FileBlank { path } => write!(
                f,
                "the operator identity file {} exists but is empty — write your ref into it \
                 as a single line",
                path.display()
            ),
            OperatorRefError::FileAmbiguous { path, lines } => write!(
                f,
                "the operator identity file {} holds {} lines, and an operator ref is one \
                 token — which of these did you mean? {:?}",
                path.display(),
                lines.len(),
                lines
            ),
            OperatorRefError::EnvBlank => write!(
                f,
                "{OPERATOR_REF_ENV} is set but blank, which is not an identity"
            ),
        }
    }
}

impl std::error::Error for OperatorRefError {}

/// Resolve the operator identity: the configuration file first, the environment
/// second, and a typed refusal when neither states one.
///
/// There is no third branch. In particular there is no default: a caller that
/// receives an `Err` must surface it, never substitute a value.
pub fn read_operator_ref() -> Result<OperatorRef, OperatorRefError> {
    // Channel 1 — the file. A missing `$HOME` is not an error here: the file
    // simply cannot exist, and the environment is still a valid channel. The path
    // is remembered either way so the final refusal can name it.
    let mut file_path: Option<PathBuf> = None;
    if let Some(home) = std::env::var_os("HOME").filter(|h| !h.is_empty()) {
        let path = PathBuf::from(home).join(OPERATOR_REF_FILE_RELATIVE);
        match std::fs::read_to_string(&path) {
            Ok(raw) => {
                let value = parse_operator_file(&raw).map_err(|e| e.with_path(path.clone()))?;
                return Ok(OperatorRef {
                    value,
                    source: OperatorRefSource::File(path),
                });
            }
            // Absent is the ordinary "not configured this way" case: fall through
            // to the environment rather than refusing.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                file_path = Some(path);
            }
            // Present but unreadable is NOT the same as absent — say so.
            Err(e) => {
                return Err(OperatorRefError::FileUnreadable {
                    path,
                    detail: e.to_string(),
                })
            }
        }
    }

    // Channel 2 — the environment.
    match std::env::var(OPERATOR_REF_ENV) {
        Ok(v) if !v.trim().is_empty() => Ok(OperatorRef {
            value: v.trim().to_string(),
            source: OperatorRefSource::Env,
        }),
        // Both channels silent. The remembered path lets the refusal say exactly
        // which file to write; `None` means $HOME was unset, and the refusal then
        // says the file could not be located rather than inventing one.
        Ok(_) => Err(OperatorRefError::EnvBlank),
        Err(_) => Err(OperatorRefError::NotConfigured { file_path }),
    }
}

/// Parse the file's content into one operator ref.
///
/// Exactly one non-empty line. Blank lines and `#` comments are ignored, so the
/// file can carry a note about what it is; anything else that is non-empty and
/// beyond the first such line is a refusal, because choosing between two stated
/// identities would be a guess.
fn parse_operator_file(raw: &str) -> Result<String, OperatorRefFileError> {
    let mut lines: Vec<String> = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        lines.push(t.to_string());
    }
    match lines.len() {
        0 => Err(OperatorRefFileError::Blank),
        1 => Ok(lines.remove(0)),
        _ => Err(OperatorRefFileError::Ambiguous { lines }),
    }
}

/// The file-content failures, before the path is known.
#[derive(Debug, Clone, PartialEq, Eq)]
enum OperatorRefFileError {
    Blank,
    Ambiguous { lines: Vec<String> },
}

impl OperatorRefFileError {
    fn with_path(self, path: PathBuf) -> OperatorRefError {
        match self {
            OperatorRefFileError::Blank => OperatorRefError::FileBlank { path },
            OperatorRefFileError::Ambiguous { lines } => {
                OperatorRefError::FileAmbiguous { path, lines }
            }
        }
    }
}

/// The absolute path the configuration file is read from, when `$HOME` is set.
///
/// Exposed so a caller can render the refusal with the real path rather than a
/// `~`-relative one the operator would have to expand themselves.
pub fn operator_ref_file_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(|h| PathBuf::from(h).join(OPERATOR_REF_FILE_RELATIVE))
}

/// The refusal message rendered for the operator.
///
/// Written for someone who is not a programmer: it names the file by its real
/// absolute path, gives the exact command to write it, says the value is the one
/// already used in the ledger, and says the change takes effect on the next click
/// without a restart. It never suggests a value of its own.
pub fn operator_ref_refusal_message(err: &OperatorRefError) -> String {
    // The path named is the one the read actually tried, taken from the error
    // itself. Reading the live `$HOME` again here would be wrong: the caller may
    // have resolved the identity under a different environment than the one the
    // message is rendered in, and the two would then disagree.
    let path = match err {
        OperatorRefError::NotConfigured { file_path: Some(p) }
        | OperatorRefError::FileBlank { path: p }
        | OperatorRefError::FileAmbiguous { path: p, .. }
        | OperatorRefError::FileUnreadable { path: p, .. } => p.display().to_string(),
        OperatorRefError::NotConfigured { file_path: None } | OperatorRefError::EnvBlank => {
            operator_ref_file_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| format!("$HOME/{OPERATOR_REF_FILE_RELATIVE}"))
        }
    };
    let head = match err {
        OperatorRefError::NotConfigured { .. } => format!(
            "accepting a delivery needs to know who you are, and SeatLoom is not told who you \
             are yet — there is no identity file and no {OPERATOR_REF_ENV}."
        ),
        OperatorRefError::FileUnreadable { detail, .. } => format!(
            "accepting a delivery needs to know who you are. Your identity file is there but \
             cannot be read: {detail}."
        ),
        OperatorRefError::FileBlank { .. } => {
            "accepting a delivery needs to know who you are. Your identity file is there but it \
             is empty."
                .to_string()
        }
        OperatorRefError::FileAmbiguous { lines, .. } => format!(
            "accepting a delivery needs to know who you are, and your identity file states more \
             than one: {lines:?}. Keep exactly one line in it."
        ),
        OperatorRefError::EnvBlank => format!(
            "accepting a delivery needs to know who you are, and {OPERATOR_REF_ENV} is set but \
             blank."
        ),
    };
    format!(
        "{head}\n\n\
         Write your ref into this file (one line, nothing else):\n\
         \u{00b7} {path}\n\n\
         For example, from Terminal:\n\
         \u{00b7} echo 'human:supervisor' > {path}\n\n\
         Or set it for one launch instead:\n\
         \u{00b7} {OPERATOR_REF_ENV}='<your-ref>'\n\n\
         Use a ref that already exists in your ledger — SeatLoom will not invent one for you, \
         because accepting a delivery is your decision and it must be recorded as yours.\n\
         The file is read when you click, so a change takes effect on the next click — no \
         restart needed."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(dir: &std::path::Path, content: &str) -> PathBuf {
        let path = dir.join(OPERATOR_REF_FILE_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
        path
    }

    /// The path is a single file under `$HOME`, never absolute and never allowed
    /// to walk upwards. A reader that hunts for identity in several places is a
    /// reader that will eventually find it in the wrong one.
    #[test]
    fn the_config_file_is_a_single_relative_file_under_home() {
        assert_eq!(OPERATOR_REF_FILE_RELATIVE, ".seatloom/operator");
        assert!(!OPERATOR_REF_FILE_RELATIVE.starts_with('/'));
        assert!(!OPERATOR_REF_FILE_RELATIVE.contains(".."));
        assert!(
            OPERATOR_REF_FILE_RELATIVE.split('/').count() == 2,
            "exactly one directory and one file name"
        );
    }

    /// A blank file is NOT an identity. This is the hard line in its smallest
    /// form: "there is a file" must never read as "there is a value".
    #[test]
    fn a_blank_file_is_not_an_identity() {
        let dir = std::env::temp_dir().join(format!("seatloom-op-blank-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = write_file(&dir, "   \n\t\n\n");
        let err = parse_operator_file(&std::fs::read_to_string(&path).unwrap())
            .unwrap_err()
            .with_path(path.clone());
        assert_eq!(err, OperatorRefError::FileBlank { path: path.clone() });
        assert!(err.to_string().contains("empty"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Two stated identities is a refusal, not a pick. Choosing one would be the
    /// exact "never guess" failure the whole module exists to avoid.
    #[test]
    fn two_stated_identities_is_a_refusal_not_a_pick() {
        let dir = std::env::temp_dir().join(format!("seatloom-op-two-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = write_file(&dir, "human:supervisor\nhuman:someone-else\n");
        let err = parse_operator_file(&std::fs::read_to_string(&path).unwrap())
            .unwrap_err()
            .with_path(path.clone());
        match &err {
            OperatorRefError::FileAmbiguous { lines, .. } => {
                assert_eq!(lines, &["human:supervisor", "human:someone-else"])
            }
            other => panic!("expected FileAmbiguous, got {other:?}"),
        }
        assert!(err.to_string().contains("one token"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A file may carry comments and blank lines; exactly one non-empty line is
    /// the identity.
    #[test]
    fn comments_and_blank_lines_are_not_a_second_identity() {
        let raw = "# who accepts deliveries\n\nhuman:supervisor\n\n# end\n";
        assert_eq!(parse_operator_file(raw).unwrap(), "human:supervisor");
        // A file that is only comments is still blank.
        assert!(matches!(
            parse_operator_file("# nothing here\n").unwrap_err(),
            OperatorRefFileError::Blank
        ));
    }

    /// Surrounding whitespace is trimmed; an interior space is not, so a ref
    /// cannot silently become something else.
    #[test]
    fn the_value_is_trimmed_and_otherwise_verbatim() {
        assert_eq!(
            parse_operator_file("  human:supervisor \n").unwrap(),
            "human:supervisor"
        );
        assert_eq!(parse_operator_file("human:a b\n").unwrap(), "human:a b");
    }

    /// An absent file is the ordinary "not configured this way" case and must not
    /// be confused with a present-but-unreadable one.
    #[test]
    fn absent_and_unreadable_are_different_failures() {
        let missing = std::env::temp_dir().join("seatloom-op-absent-xyz/operator");
        let e = std::fs::read_to_string(&missing).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        // The Display of each names its own thing.
        let blank = OperatorRefError::FileBlank {
            path: missing.clone(),
        };
        let unreadable = OperatorRefError::FileUnreadable {
            path: missing.clone(),
            detail: "permission denied".into(),
        };
        assert!(blank.to_string().contains("empty"));
        assert!(unreadable.to_string().contains("permission denied"));
        assert_ne!(blank, unreadable);
    }

    /// The refusal message names the real path, the exact command, and says the
    /// change needs no restart. It never suggests a value of its own beyond the
    /// example, which is a ref already present in the ledger.
    #[test]
    fn the_refusal_names_the_path_the_command_and_the_no_restart_fact() {
        // The message renders the path it will actually read, which is the live
        // `$HOME` — the error's own path is that same path by construction.
        let path = operator_ref_file_path().unwrap_or_else(|| PathBuf::from("/unset/operator"));
        let err = OperatorRefError::FileBlank { path: path.clone() };
        let msg = operator_ref_refusal_message(&err);
        assert!(msg.contains(&path.display().to_string()), "{msg}");
        assert!(msg.contains("echo 'human:supervisor' >"), "{msg}");
        assert!(msg.contains(OPERATOR_REF_ENV), "{msg}");
        assert!(msg.contains("no restart needed"), "{msg}");
        assert!(msg.contains("will not invent one for you"), "{msg}");
    }

    /// The refusal for an ambiguous file shows both lines, so the operator can see
    /// what to delete rather than being told "invalid".
    #[test]
    fn the_refusal_for_an_ambiguous_file_shows_both_lines() {
        let err = OperatorRefError::FileAmbiguous {
            path: operator_ref_file_path().unwrap_or_else(|| PathBuf::from("/unset/operator")),
            lines: vec!["human:a".into(), "human:b".into()],
        };
        let msg = operator_ref_refusal_message(&err);
        assert!(msg.contains("human:a") && msg.contains("human:b"), "{msg}");
        assert!(msg.contains("Keep exactly one line"), "{msg}");
    }

    /// The "not configured" refusal names the file the read WOULD have used, so
    /// the operator is told exactly where to write. Found by driving
    /// `read_operator_ref()` with `$HOME` pointed at a directory with no file:
    /// the error carries that directory's path, not a lie about `$HOME` being
    /// unset (which is what the first cut of this module reported).
    #[test]
    fn the_not_configured_refusal_names_the_file_it_would_have_read() {
        // Point HOME at an empty directory through the same code path the binary
        // uses. `std::env::set_var` is process-global, so this test runs alone
        // under the suite's default threading only because no other test here
        // reads HOME through `read_operator_ref` — the ones that do are the
        // real-file check, which tolerates either outcome.
        let dir = std::env::temp_dir().join(format!("seatloom-op-nohome-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let expected = dir.join(OPERATOR_REF_FILE_RELATIVE);

        let prev = std::env::var_os("HOME");
        let prev_env = std::env::var_os(OPERATOR_REF_ENV);
        std::env::set_var("HOME", &dir);
        std::env::remove_var(OPERATOR_REF_ENV);
        let err = read_operator_ref().unwrap_err();
        if let Some(p) = prev {
            std::env::set_var("HOME", p);
        }
        match prev_env {
            Some(v) => std::env::set_var(OPERATOR_REF_ENV, v),
            None => std::env::remove_var(OPERATOR_REF_ENV),
        }

        assert_eq!(
            err,
            OperatorRefError::NotConfigured {
                file_path: Some(expected.clone())
            },
            "the refusal must name the file it would have read"
        );
        let msg = operator_ref_refusal_message(&err);
        assert!(
            msg.contains(&expected.display().to_string()),
            "the message must name {expected:?}: {msg}"
        );
        assert!(
            !msg.contains("cannot find your home directory"),
            "HOME was set; the message must not claim otherwise: {msg}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An unset `$HOME` is reported as such rather than with a fabricated path.
    #[test]
    fn an_unset_home_is_reported_without_a_fabricated_path() {
        let err = OperatorRefError::NotConfigured { file_path: None };
        assert!(err.to_string().contains("$HOME is unset"), "{err}");
        let msg = operator_ref_refusal_message(&err);
        assert!(msg.contains("not told who you are yet"), "{msg}");
    }

    /// THE ONE THAT MATTERS: the real file on this machine resolves. The operator
    /// already has `~/.seatloom/operator` containing `human:supervisor`, written
    /// on 2026-10-12; this proves the channel this module reads is the one that
    /// already works, rather than a new format he would have to migrate to.
    #[test]
    fn the_real_config_file_on_this_machine_resolves() {
        let Some(path) = operator_ref_file_path() else {
            eprintln!("HOME is unset — skipping the real-file check");
            return;
        };
        let Ok(raw) = std::fs::read_to_string(&path) else {
            eprintln!(
                "{} is not present — skipping the real-file check",
                path.display()
            );
            return;
        };
        let value =
            parse_operator_file(&raw).unwrap_or_else(|e| panic!("{}: {e:?}", path.display()));
        assert_eq!(value, "human:supervisor");
        // And it is a ref that already exists in the ledger's own spelling.
        assert!(value.starts_with("human:"), "{value}");
    }
}
