use crate::kernel::{parse, Form};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildPlan {
    pub project_id: String,
    pub project_version: String,
    pub profile: String,
    pub language: String,
    pub main: String,
    pub entrypoints: Vec<String>,
    pub keep_vars: Vec<String>,
    pub keep_namespaces: Vec<String>,
    pub output_bundle: String,
    pub output_report: String,
}

impl BuildPlan {
    pub fn parse(source: &str) -> Result<Self, String> {
        let form = parse(source).map_err(|error| format!("invalid production build plan: {error}"))?;
        let entries = map_entries(&form, "production build plan must be an EDN map")?;
        let project_id = scalar(required(entries, "project/id")?, ":project/id")?;
        let project_version = string(required(entries, "project/version")?, ":project/version")?;
        let profile = identifier(required(entries, "profile/name")?, ":profile/name")?;
        let language = identifier(required(entries, "profile/language")?, ":profile/language")?;
        let main = scalar(required(entries, "profile/main")?, ":profile/main")?;
        let entrypoints = symbol_vector(required(entries, "build/entrypoints")?, ":build/entrypoints", true)?;
        let keep_vars = symbol_vector(required(entries, "build/keep-vars")?, ":build/keep-vars", true)?;
        let keep_namespaces = symbol_vector(
            required(entries, "build/keep-namespaces")?,
            ":build/keep-namespaces",
            false,
        )?;
        let output_bundle = string(
            required(entries, "build/output-bundle")?,
            ":build/output-bundle",
        )?;
        let output_report = string(
            required(entries, "build/output-report")?,
            ":build/output-report",
        )?;
        if language != "hara" {
            return Err("production build plan must use :profile/language :hara".into());
        }
        if entrypoints.is_empty() {
            return Err("production build plan has no entrypoints".into());
        }
        Ok(Self {
            project_id,
            project_version,
            profile,
            language,
            main,
            entrypoints,
            keep_vars,
            keep_namespaces,
            output_bundle,
            output_report,
        })
    }

    pub fn report_path(&self, root: &std::path::Path) -> PathBuf {
        root.join(&self.output_report)
    }
}

fn map_entries<'a>(form: &'a Form, message: &str) -> Result<&'a [(Form, Form)], String> {
    match form {
        Form::Map(entries) => Ok(entries),
        _ => Err(message.into()),
    }
}

fn required<'a>(entries: &'a [(Form, Form)], key: &str) -> Result<&'a Form, String> {
    entries
        .iter()
        .find_map(|(candidate, value)| {
            matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
        })
        .ok_or_else(|| format!("production build plan is missing :{key}"))
}

fn scalar(form: &Form, label: &str) -> Result<String, String> {
    match form {
        Form::Symbol(value) | Form::String(value) => Ok(value.clone()),
        _ => Err(format!("{label} must be a symbol or string")),
    }
}

fn string(form: &Form, label: &str) -> Result<String, String> {
    match form {
        Form::String(value) => Ok(value.clone()),
        _ => Err(format!("{label} must be a string")),
    }
}

fn identifier(form: &Form, label: &str) -> Result<String, String> {
    match form {
        Form::Keyword(value) | Form::Symbol(value) | Form::String(value) => Ok(value.clone()),
        _ => Err(format!("{label} must be a keyword, symbol, or string")),
    }
}

fn symbol_vector(form: &Form, label: &str, qualified: bool) -> Result<Vec<String>, String> {
    let Form::Vector(values) = form else {
        return Err(format!("{label} must be a vector"));
    };
    let mut output = values
        .iter()
        .map(|value| scalar(value, label))
        .collect::<Result<Vec<_>, _>>()?;
    if qualified && output.iter().any(|value| !qualified_var(value)) {
        return Err(format!("{label} must contain qualified Var symbols"));
    }
    if !qualified && output.iter().any(|value| value.contains('/')) {
        return Err(format!("{label} must contain namespace symbols"));
    }
    output.sort();
    output.dedup();
    Ok(output)
}

fn qualified_var(value: &str) -> bool {
    let Some((namespace, name)) = value.split_once('/') else {
        return false;
    };
    !namespace.is_empty() && !name.is_empty() && !name.contains('/')
}
