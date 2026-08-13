//! Runtime-neutral CLI contracts shared by native Hara entrypoints.
//!
//! Command routing is implemented by `tool.cli.route`; Rust embeds the
//! normative manifest plus focused versioned extensions and maps public
//! outcomes to process exit codes.

use crate::kernel::{parse, Form};
use std::fmt;
use std::sync::LazyLock;

pub const BASE_MANIFEST_SOURCE: &str = include_str!("../resources/hara-cli.edn");
pub const PROJECT_BUILD_MANIFEST_SOURCE: &str =
    include_str!("../resources/hara-cli-project-build.edn");

#[derive(Clone, Copy)]
pub struct ManifestSource;

pub const MANIFEST_SOURCE: ManifestSource = ManifestSource;

static MERGED_MANIFEST_SOURCE: LazyLock<String> = LazyLock::new(|| {
    merge_manifest_sources(BASE_MANIFEST_SOURCE, PROJECT_BUILD_MANIFEST_SOURCE)
        .expect("embedded CLI manifest extensions must be valid")
});

impl fmt::Debug for ManifestSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(merged_manifest_source(), formatter)
    }
}

pub fn merged_manifest_source() -> &'static str {
    MERGED_MANIFEST_SOURCE.as_str()
}

fn map_entries(form: &Form) -> Result<&[(Form, Form)], String> {
    match form {
        Form::Map(entries) => Ok(entries),
        _ => Err("CLI manifest value must be an EDN map".into()),
    }
}

fn map_entries_mut(form: &mut Form) -> Result<&mut Vec<(Form, Form)>, String> {
    match form {
        Form::Map(entries) => Ok(entries),
        _ => Err("CLI manifest value must be an EDN map".into()),
    }
}

fn map_value<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    entries.iter().find_map(|(candidate, value)| {
        matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
    })
}

fn map_value_mut<'a>(entries: &'a mut [(Form, Form)], key: &str) -> Option<&'a mut Form> {
    entries.iter_mut().find_map(|(candidate, value)| {
        matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
    })
}

fn set_map_value(entries: &mut Vec<(Form, Form)>, key: &str, value: Form) {
    if let Some(current) = map_value_mut(entries, key) {
        *current = value;
    } else {
        entries.push((Form::Keyword(key.into()), value));
    }
}

fn keyword_value(form: &Form) -> Option<&str> {
    match form {
        Form::Keyword(value) => Some(value),
        _ => None,
    }
}

fn map_keyword<'a>(form: &'a Form, key: &str) -> Option<&'a str> {
    map_entries(form)
        .ok()
        .and_then(|entries| map_value(entries, key))
        .and_then(keyword_value)
}

fn vector_mut(form: &mut Form) -> Result<&mut Vec<Form>, String> {
    match form {
        Form::Vector(values) => Ok(values),
        _ => Err("CLI manifest collection must be an EDN vector".into()),
    }
}

fn append_unique_entry(values: &mut Vec<Form>, field: &str, id: &str, entry: Form) {
    if !values
        .iter()
        .any(|candidate| map_keyword(candidate, field) == Some(id))
    {
        values.push(entry);
    }
}

fn merge_manifest_sources(base: &str, extension: &str) -> Result<String, String> {
    let mut manifest = parse(base)?;
    let extension = parse(extension)?;
    let extension_entries = map_entries(&extension)?;

    let app_id = map_value(extension_entries, "app/id")
        .and_then(keyword_value)
        .ok_or("CLI manifest extension is missing keyword :app/id")?
        .to_owned();
    let app_summary = map_value(extension_entries, "app/summary")
        .cloned()
        .ok_or("CLI manifest extension is missing :app/summary")?;
    let route = map_value(extension_entries, "route")
        .cloned()
        .ok_or("CLI manifest extension is missing :route")?;
    let handler = map_value(extension_entries, "handler")
        .cloned()
        .ok_or("CLI manifest extension is missing :handler")?;

    let route_id = map_keyword(&route, "route/id")
        .ok_or("CLI route is missing keyword :route/id")?
        .to_owned();
    let route_handler = map_keyword(&route, "route/handler")
        .ok_or("CLI route is missing keyword :route/handler")?
        .to_owned();
    let handler_id = map_keyword(&handler, "handler/id")
        .ok_or("CLI handler is missing keyword :handler/id")?
        .to_owned();
    if route_handler != handler_id {
        return Err("CLI route and handler ids do not match".into());
    }

    let manifest_entries = map_entries_mut(&mut manifest)?;
    let apps = map_value_mut(manifest_entries, "cli/apps")
        .ok_or("CLI manifest is missing :cli/apps")
        .and_then(vector_mut)?;
    let mut app_found = false;
    for app in apps {
        if map_keyword(app, "app/id") == Some(app_id.as_str()) {
            app_found = true;
            let app_entries = map_entries_mut(app)?;
            set_map_value(app_entries, "app/summary", app_summary.clone());
            let routes = map_value_mut(app_entries, "app/routes")
                .ok_or("CLI app is missing :app/routes")
                .and_then(vector_mut)?;
            if !routes
                .iter()
                .any(|candidate| keyword_value(candidate) == Some(route_id.as_str()))
            {
                routes.push(Form::Keyword(route_id.clone()));
            }
        }
    }
    if !app_found {
        return Err(format!("CLI manifest is missing app :{app_id}"));
    }

    let manifest_entries = map_entries_mut(&mut manifest)?;
    let routes = map_value_mut(manifest_entries, "cli/routes")
        .ok_or("CLI manifest is missing :cli/routes")
        .and_then(vector_mut)?;
    append_unique_entry(routes, "route/id", &route_id, route);

    let manifest_entries = map_entries_mut(&mut manifest)?;
    let handlers = map_value_mut(manifest_entries, "cli/handlers")
        .ok_or("CLI manifest is missing :cli/handlers")
        .and_then(vector_mut)?;
    append_unique_entry(handlers, "handler/id", &handler_id, handler);

    Ok(manifest.to_string())
}

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
    use super::{
        map_entries, map_keyword, map_value, merge_manifest_sources, merged_manifest_source,
        CliOutcome, BASE_MANIFEST_SOURCE, MANIFEST_SOURCE, PROJECT_BUILD_MANIFEST_SOURCE,
    };
    use crate::kernel::{parse, Form};

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
            submodule, BASE_MANIFEST_SOURCE,
            "rust/resources/hara-cli.edn is stale; refresh it from hara-specs-registry/00-unsorted/cli/draft/hara-cli.edn"
        );
    }

    #[test]
    fn production_project_build_contract_is_composed_once() {
        parse(BASE_MANIFEST_SOURCE).expect("CLI manifest must remain valid EDN");
        parse(PROJECT_BUILD_MANIFEST_SOURCE)
            .expect("project build manifest extension must remain valid EDN");
        let merged = parse(merged_manifest_source()).expect("merged CLI manifest must be valid EDN");
        let entries = map_entries(&merged).unwrap();

        let Form::Vector(routes) = map_value(entries, "cli/routes").unwrap() else {
            panic!(":cli/routes must be a vector");
        };
        assert_eq!(
            routes
                .iter()
                .filter(|route| {
                    map_keyword(route, "route/id") == Some("tool.cli.route/project-build")
                })
                .count(),
            1
        );

        let Form::Vector(handlers) = map_value(entries, "cli/handlers").unwrap() else {
            panic!(":cli/handlers must be a vector");
        };
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| {
                    map_keyword(handler, "handler/id")
                        == Some("tool.cli.handler/project-build")
                })
                .count(),
            1
        );

        assert_eq!(
            merge_manifest_sources(merged_manifest_source(), PROJECT_BUILD_MANIFEST_SOURCE)
                .unwrap(),
            merged_manifest_source()
        );
        assert_eq!(
            parse(&format!("{MANIFEST_SOURCE:?}")).unwrap(),
            Form::String(merged_manifest_source().into())
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
