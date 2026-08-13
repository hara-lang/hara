//! Shared CLI contracts for Hara runtimes.

mod manifest;
mod outcome;
#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::OnceLock;

pub use outcome::CliOutcome;

pub const BASE_MANIFEST_SOURCE: &str = include_str!("../resources/hara-cli.edn");
pub const PROJECT_BUILD_MANIFEST_SOURCE: &str =
    include_str!("../resources/hara-cli-project-build.edn");

#[derive(Clone, Copy)]
pub struct ManifestSource;

pub const MANIFEST_SOURCE: ManifestSource = ManifestSource;

static MERGED_MANIFEST_SOURCE: OnceLock<String> = OnceLock::new();

impl fmt::Debug for ManifestSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(merged_manifest_source(), formatter)
    }
}

pub fn merged_manifest_source() -> &'static str {
    MERGED_MANIFEST_SOURCE
        .get_or_init(|| {
            manifest::merge_sources(BASE_MANIFEST_SOURCE, PROJECT_BUILD_MANIFEST_SOURCE)
                .expect("embedded CLI manifest extensions must be valid")
        })
        .as_str()
}
