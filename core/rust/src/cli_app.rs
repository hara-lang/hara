//! Runtime-neutral CLI contracts shared by native Hara entrypoints.
//!
//! Command routing is implemented by `hara.cli.route`; Rust only embeds the
//! normative manifest and maps public outcomes to process exit codes.

pub const MANIFEST_SOURCE: &str = include_str!("../resources/hara-cli.edn");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliOutcome {
    Success,
    Failed,
    UsageError,
    ReadError,
    ResolutionError,
    Unavailable,
    InternalError,
    Interrupted,
}

impl CliOutcome {
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failed => 1,
            Self::UsageError
            | Self::ReadError
            | Self::ResolutionError
            | Self::Unavailable
            | Self::InternalError => 2,
            Self::Interrupted => 130,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CliOutcome, MANIFEST_SOURCE};

    fn repo_text(relative: &str) -> Option<String> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("hara-specs-registry")
            .join(relative);
        match std::fs::read_to_string(&path) {
            Ok(content) => Some(content),
            Err(_) => {
                eprintln!(
                    "skipping: {} is unavailable (hara-specs-registry sibling repo not present)",
                    path.display()
                );
                None
            }
        }
    }

    #[test]
    fn vendored_manifest_matches_specs_submodule_when_present() {
        let Some(submodule) = repo_text("00-unsorted/cli/draft/hara-cli.edn") else {
            return;
        };
        assert_eq!(
            submodule, MANIFEST_SOURCE,
            "rust/resources/hara-cli.edn is stale; refresh it from hara-specs-registry/00-unsorted/cli/draft/hara-cli.edn"
        );
    }

    #[test]
    fn public_outcomes_have_stable_exit_codes() {
        assert_eq!(CliOutcome::Success.exit_code(), 0);
        assert_eq!(CliOutcome::Failed.exit_code(), 1);
        assert_eq!(CliOutcome::ReadError.exit_code(), 2);
        assert_eq!(CliOutcome::Interrupted.exit_code(), 130);
    }
}
