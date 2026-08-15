use std::collections::{HashMap, HashSet};

use super::Form;

const LIBRARIES: &[(&str, &str, &str)] = &[
    ("string", "std.foundation.string", "str"),
    ("promise", "std.foundation.promise", "promise"),
    ("bytes", "std.foundation.bytes", "bytes"),
    ("coroutine", "std.foundation.coroutine", "co"),
    ("pretty", "std.foundation.pretty", "pretty"),
];

pub(crate) fn foundation_library_aliases() -> impl Iterator<Item = (&'static str, &'static str)> {
    LIBRARIES
        .iter()
        .map(|(_, namespace, alias)| (*namespace, *alias))
}
#[path = "generated/rewrite.rs"]
mod rewrite;

const NATIVE_TYPES: &[&str] = &[
    "Maths",
    "Numbers",
    "Bits",
    "String",
    "Bytes",
    "File",
    "Socket",
    "Promise",
    "Coroutine",
    "Arr",
    "Obj",
    "Runtime",
    "Printer",
    "Document",
    "Edn",
    "Json",
    "Crypto",
    "Host",
    "Test",
    "Regex",
    "UUID",
    "Error",
    "Base",
    "Iter",
    "Kernel",
];

#[derive(Debug, Clone, Default)]
pub struct GeneratedNamespaceConfig {
    aliases: HashMap<String, String>,
    lazy_aliases: HashMap<String, String>,
    refers: HashMap<String, String>,
    macro_refers: HashMap<String, String>,
    required_namespaces: Vec<String>,
    used_namespaces: Vec<String>,
    used_exclusions: HashMap<String, HashSet<String>>,
    excluded_foundation: HashSet<String>,
    exposed_foundation: Option<HashSet<String>>,
    blank: bool,
}

impl GeneratedNamespaceConfig {
    pub fn defaults() -> Self {
        let mut aliases: HashMap<String, String> = LIBRARIES
            .iter()
            .map(|(_, namespace, alias)| ((*alias).into(), (*namespace).into()))
            .collect();
        for native_type in NATIVE_TYPES {
            aliases.insert((*native_type).into(), format!("std.native.{native_type}"));
        }
        Self {
            aliases,
            lazy_aliases: HashMap::new(),
            refers: HashMap::new(),
            macro_refers: HashMap::new(),
            required_namespaces: Vec::new(),
            used_namespaces: Vec::new(),
            used_exclusions: HashMap::new(),
            excluded_foundation: HashSet::new(),
            exposed_foundation: None,
            blank: false,
        }
    }

    pub fn configure(clauses: &[Form]) -> Result<Self, String> {
        Self::configure_with(clauses, known_namespace)
    }

    pub fn configure_with(
        clauses: &[Form],
        available: impl Fn(&str) -> bool,
    ) -> Result<Self, String> {
        let mut excluded = HashSet::new();
        let mut overrides = HashMap::new();
        let mut requires = Vec::new();
        let mut uses = Vec::new();
        let mut excluded_foundation = HashSet::new();
        let mut exposed_foundation = None;
        let mut override_seen = false;
        let mut blank = false;
        let mut intrinsics_seen = false;
        let mut config_seen = false;

        for clause in clauses {
            let values = list(clause, "ns clauses must be non-empty lists")?;
            let head = values.first().ok_or("ns clauses must be non-empty lists")?;
            let name = keyword(head, "ns clause must start with a keyword")?;
            match name {
                "config" => {
                    if config_seen {
                        return Err("ns accepts only one :config clause".into());
                    }
                    config_seen = true;
                    if values.len() != 2 {
                        return Err(":config expects one map".into());
                    }
                    parse_config(
                        &values[1],
                        &mut blank,
                        &mut excluded_foundation,
                        &mut exposed_foundation,
                        &mut override_seen,
                        &mut excluded,
                        &mut overrides,
                    )?;
                }
                "intrinsics" => {
                    // Legacy top-level :intrinsics is accepted for backward compatibility,
                    // but the spec places it inside :config.
                    if intrinsics_seen {
                        return Err("ns accepts only one :intrinsics clause".into());
                    }
                    intrinsics_seen = true;
                    if values.len() != 2 {
                        return Err(":intrinsics expects :all or an options map".into());
                    }
                    if !matches!(&values[1], Form::Keyword(name) if name == "all") {
                        parse_intrinsics(&values[1], &mut excluded, &mut overrides)?;
                    }
                }
                "require" => requires.extend(values[1..].iter().cloned()),
                "use" => uses.extend(values[1..].iter().cloned()),
                "flavor" | "import" => {}
                other => return Err(format!("Unsupported ns clause: :{other}")),
            }
        }

        if blank && override_seen {
            return Err(":config :blank true cannot be combined with :override".into());
        }
        if blank && exposed_foundation.is_some() {
            return Err(":config :blank true cannot be combined with :expose".into());
        }
        if override_seen && exposed_foundation.is_some() {
            return Err(":config :override cannot be combined with :expose".into());
        }

        for library in overrides.keys() {
            if excluded.contains(library) {
                return Err(format!(
                    "Intrinsic library cannot be both excluded and aliased: {library}"
                ));
            }
        }

        let mut config = Self::default();
        config.excluded_foundation = excluded_foundation;
        config.exposed_foundation = exposed_foundation;
        config.blank = blank;
        for native_type in NATIVE_TYPES {
            config.put_alias(native_type, &format!("std.native.{native_type}"))?;
        }
        for (library, namespace, default_alias) in LIBRARIES {
            if excluded.contains(*library) {
                continue;
            }
            let alias = overrides
                .get(*library)
                .map_or(*default_alias, String::as_str);
            config.put_alias(alias, namespace)?;
        }
        for require in requires {
            config.apply_require(&require, &available)?;
        }
        for use_form in uses {
            config.apply_use(&use_form, &available)?;
        }
        Ok(config)
    }

    pub fn required_namespaces(&self) -> &[String] {
        &self.required_namespaces
    }

    pub fn lazy_target(&self, alias: &str) -> Option<&str> {
        self.lazy_aliases.get(alias).map(String::as_str)
    }

    pub fn used_namespaces(&self) -> &[String] {
        &self.used_namespaces
    }

    pub fn used_symbol_excluded(&self, namespace: &str, symbol: &str) -> bool {
        self.used_exclusions
            .get(namespace)
            .is_some_and(|excluded| excluded.contains(symbol))
    }

    pub fn excluded_foundation(&self) -> &HashSet<String> {
        &self.excluded_foundation
    }

    pub fn exposed_foundation(&self) -> Option<&HashSet<String>> {
        self.exposed_foundation.as_ref()
    }

    pub fn blank(&self) -> bool {
        self.blank
    }

    pub fn aliases(&self) -> Vec<(String, String)> {
        self.aliases
            .iter()
            .map(|(alias, namespace)| (alias.clone(), namespace.clone()))
            .collect()
    }

    fn put_alias(&mut self, alias: &str, namespace: &str) -> Result<(), String> {
        if alias.is_empty() {
            return Err("Namespace alias cannot be empty".into());
        }
        if alias == "-" {
            return Err("Namespace alias is reserved: -".into());
        }
        if let Some(previous) = self.aliases.get(alias) {
            if previous != namespace {
                return Err(format!(
                    "Namespace alias already refers to {previous}: {alias}"
                ));
            }
            return Ok(());
        }
        self.aliases.insert(alias.into(), namespace.into());
        Ok(())
    }

    pub fn apply_require(
        &mut self,
        form: &Form,
        available: &impl Fn(&str) -> bool,
    ) -> Result<(), String> {
        let (target, options) = match form {
            Form::Vector(items) => {
                let target = match items.first() {
                    Some(Form::Symbol(target)) => target.as_str(),
                    _ => return Err(":require namespace must be a symbol".into()),
                };
                (normalize_namespace(target), &items[1..])
            }
            Form::List(items)
                if items.len() == 2
                    && matches!(&items[0], Form::Symbol(q) if q == "quote")
                    && matches!(&items[1], Form::Symbol(_)) =>
            {
                let target = match &items[1] {
                    Form::Symbol(target) => target.as_str(),
                    _ => unreachable!(),
                };
                (normalize_namespace(target), &[][..])
            }
            _ => return Err(":require expects vectors such as [hara.lib.string :as str]".into()),
        };
        if !known_namespace(target) && !available(target) {
            return Err(format!(
                "Cannot require missing generated namespace: {target}"
            ));
        }
        if options.len() % 2 != 0 {
            return Err(format!("Malformed :require options for {target}"));
        }
        let lazy = options.chunks(2).any(|option| {
            matches!(&option[0], Form::Keyword(name) if name == "lazy")
                && matches!(&option[1], Form::Bool(true))
        });
        let has_alias = options
            .chunks(2)
            .any(|option| matches!(&option[0], Form::Keyword(name) if name == "as"));
        if lazy && !has_alias {
            return Err(":require :lazy requires :as".into());
        }
        if !lazy && !self.required_namespaces.iter().any(|value| value == target) {
            self.required_namespaces.push(target.into());
        }
        for option in options.chunks(2) {
            let name = keyword(&option[0], "Malformed :require options")?;
            match name {
                "as" => {
                    let alias = symbol(&option[1], ":require :as expects an unqualified symbol")?;
                    if alias.contains('/') {
                        return Err(":require :as expects an unqualified symbol".into());
                    }
                    self.put_alias(alias, target)?;
                    if lazy {
                        self.lazy_aliases.insert(alias.into(), target.into());
                    }
                }
                "refer" => {
                    if lazy {
                        return Err(":require :lazy cannot be combined with :refer".into());
                    }
                    if matches!(&option[1], Form::Keyword(name) if name == "all") {
                        if !self.used_namespaces.iter().any(|value| value == target) {
                            self.used_namespaces.push(target.into());
                        }
                        continue;
                    }
                    let names = vector(
                        &option[1],
                        ":require :refer expects a vector of symbols or :all",
                    )?;
                    for value in names {
                        let name = symbol(value, ":require :refer expects unqualified symbols")?;
                        if qualified_symbol(name) {
                            return Err(":require :refer expects unqualified symbols".into());
                        }
                        let canonical = canonical(target, name);
                        if let Some(previous) = self.refers.insert(name.into(), canonical) {
                            return Err(format!(
                                "Referred symbol already exists: {name} ({previous})"
                            ));
                        }
                    }
                }
                "refer-macros" => {
                    if lazy {
                        return Err(":require :lazy cannot be combined with :refer-macros".into());
                    }
                    let names = vector(
                        &option[1],
                        ":require :refer-macros expects a vector of symbols",
                    )?;
                    for value in names {
                        let name =
                            symbol(value, ":require :refer-macros expects unqualified symbols")?;
                        if qualified_symbol(name) {
                            return Err(":require :refer-macros expects unqualified symbols".into());
                        }
                        let canonical = canonical(target, name);
                        if let Some(previous) = self.macro_refers.insert(name.into(), canonical) {
                            return Err(format!(
                                "Referred macro already exists: {name} ({previous})"
                            ));
                        }
                    }
                }
                "lazy" => {
                    if !matches!(&option[1], Form::Bool(true)) {
                        return Err(":require :lazy expects true".into());
                    }
                }
                "reload" => {
                    if !matches!(&option[1], Form::Bool(true)) {
                        return Err(":require :reload expects true".into());
                    }
                }
                "exclude" => {
                    let names =
                        vector(&option[1], ":require :exclude expects a vector of symbols")?;
                    for value in names {
                        let name = symbol(value, ":require :exclude expects unqualified symbols")?;
                        if qualified_symbol(name) {
                            return Err(":require :exclude expects unqualified symbols".into());
                        }
                        self.used_exclusions
                            .entry(target.into())
                            .or_default()
                            .insert(name.into());
                        if target == "std.foundation" {
                            self.excluded_foundation.insert(name.into());
                        }
                    }
                }
                other => return Err(format!("Unsupported :require option: :{other}")),
            }
        }
        Ok(())
    }

    pub fn apply_use(
        &mut self,
        form: &Form,
        available: &impl Fn(&str) -> bool,
    ) -> Result<(), String> {
        let target = match form {
            Form::Symbol(target) if !target.contains('/') => normalize_namespace(target),
            _ => return Err(":use expects unqualified namespace symbols".into()),
        };
        if !known_namespace(target) && !available(target) {
            return Err(format!("Cannot use missing generated namespace: {target}"));
        }
        if !self.required_namespaces.iter().any(|value| value == target) {
            self.required_namespaces.push(target.into());
        }
        if !self.used_namespaces.iter().any(|value| value == target) {
            self.used_namespaces.push(target.into());
        }
        Ok(())
    }
}

fn parse_config(
    form: &Form,
    blank: &mut bool,
    foundation_overrides: &mut HashSet<String>,
    foundation_exposure: &mut Option<HashSet<String>>,
    override_seen: &mut bool,
    excluded: &mut HashSet<String>,
    overrides: &mut HashMap<String, String>,
) -> Result<(), String> {
    let options = match form {
        Form::Map(options) => options,
        _ => return Err(":config expects one map".into()),
    };
    for (key, value) in options {
        match keyword(key, ":config keys must be unqualified keywords")? {
            "blank" => {
                *blank = match value {
                    Form::Bool(value) => *value,
                    _ => return Err(":config :blank expects a boolean".into()),
                };
            }
            "override" => {
                *override_seen = true;
                for item in vector(
                    value,
                    ":config :override expects a vector of unqualified symbols",
                )? {
                    let name = symbol(
                        item,
                        ":config :override expects a vector of unqualified symbols",
                    )?;
                    if qualified_symbol(name) {
                        return Err(
                            ":config :override expects a vector of unqualified symbols".into()
                        );
                    }
                    if !foundation_overrides.insert(name.into()) {
                        return Err(format!("Duplicate Foundation override: {name}"));
                    }
                }
            }
            "expose" => {
                let mut exposed = HashSet::new();
                for item in vector(
                    value,
                    ":config :expose expects a vector of unqualified symbols",
                )? {
                    let name = symbol(
                        item,
                        ":config :expose expects a vector of unqualified symbols",
                    )?;
                    if qualified_symbol(name) {
                        return Err(
                            ":config :expose expects a vector of unqualified symbols".into()
                        );
                    }
                    if !exposed.insert(name.into()) {
                        return Err(format!("Duplicate Foundation exposure: {name}"));
                    }
                }
                *foundation_exposure = Some(exposed);
            }
            "intrinsics" => {
                parse_intrinsics(value, excluded, overrides)?;
            }
            other => return Err(format!("Unsupported :config option: :{other}")),
        }
    }
    Ok(())
}

fn parse_intrinsics(
    form: &Form,
    excluded: &mut HashSet<String>,
    overrides: &mut HashMap<String, String>,
) -> Result<(), String> {
    let options = match form {
        Form::Map(options) => options,
        _ => return Err(":intrinsics expects :all or an options map".into()),
    };
    for (key, value) in options {
        match keyword(key, ":intrinsics option keys must be keywords")? {
            "exclude" => {
                for item in vector(
                    value,
                    ":intrinsics :exclude expects a vector of library symbols",
                )? {
                    let library = library(symbol(
                        item,
                        ":intrinsics :exclude expects unqualified library symbols",
                    )?)?;
                    if !excluded.insert(library.into()) {
                        return Err(format!("Duplicate intrinsic exclusion: {library}"));
                    }
                }
            }
            "aliases" => {
                let aliases = match value {
                    Form::Map(aliases) => aliases,
                    _ => return Err(":intrinsics :aliases expects a map".into()),
                };
                for (library_form, alias_form) in aliases {
                    let library = library(symbol(
                        library_form,
                        ":intrinsics :aliases expects library symbols",
                    )?)?;
                    let alias =
                        symbol(alias_form, "Intrinsic aliases must be unqualified symbols")?;
                    if alias.contains('/') {
                        return Err("Intrinsic aliases must be unqualified symbols".into());
                    }
                    if overrides.insert(library.into(), alias.into()).is_some() {
                        return Err(format!("Duplicate intrinsic alias: {library}"));
                    }
                }
            }
            other => return Err(format!("Unsupported :intrinsics option: :{other}")),
        }
    }
    Ok(())
}

fn list<'a>(form: &'a Form, error: &str) -> Result<&'a [Form], String> {
    match form {
        Form::List(values) => Ok(values),
        _ => Err(error.into()),
    }
}
fn vector<'a>(form: &'a Form, error: &str) -> Result<&'a [Form], String> {
    match form {
        Form::Vector(values) => Ok(values),
        _ => Err(error.into()),
    }
}
fn keyword<'a>(form: &'a Form, error: &str) -> Result<&'a str, String> {
    match form {
        Form::Keyword(value) => Ok(value),
        _ => Err(error.into()),
    }
}
fn symbol<'a>(form: &'a Form, error: &str) -> Result<&'a str, String> {
    match form {
        Form::Symbol(value) => Ok(value),
        _ => Err(error.into()),
    }
}
fn qualified_symbol(value: &str) -> bool {
    value != "/" && value.contains('/')
}
fn library(value: &str) -> Result<&str, String> {
    if value.contains('/') {
        return Err("Intrinsic library names must be unqualified symbols".into());
    }
    LIBRARIES
        .iter()
        .find(|(library, _, _)| *library == value)
        .map(|(library, _, _)| *library)
        .ok_or_else(|| format!("Unknown intrinsic library: {value}"))
}
pub(crate) fn normalize_namespace(value: &str) -> &str {
    match value {
        "core" | "hara.lib.core" => "std.foundation",
        "hara.lib.string" => "std.foundation.string",
        "hara.lib.promise" => "std.foundation.promise",
        "hara.lib.bytes" => "std.foundation.bytes",
        "hara.lib.socket" => "std.native.Socket",
        "hara.lib.file" => "std.native.File",
        value => value,
    }
}
fn known_namespace(value: &str) -> bool {
    let value = normalize_namespace(value);
    value == "std.foundation"
        || value == "std.foundation.coroutine"
        || value == "std.native"
        || value.starts_with("std.native.")
        || LIBRARIES
            .iter()
            .any(|(_, namespace, _)| *namespace == value)
}
fn canonical(namespace: &str, method: &str) -> String {
    if namespace == "std.foundation" {
        return format!("std.foundation/{method}");
    }
    // Coroutine operations are evaluator control forms, not ordinary HAL
    // function calls. Keep their canonical names so `co/yield` and
    // `co/await` remain visible to the fiber evaluator instead of routing
    // through the synchronous Foundation wrapper namespace.
    if normalize_namespace(namespace) == "std.foundation.coroutine" {
        return format!("std.foundation.coroutine/{method}");
    }
    if let Some((_, _, alias)) = LIBRARIES
        .iter()
        .find(|(_, library_namespace, _)| *library_namespace == normalize_namespace(namespace))
    {
        return format!("{alias}/{method}");
    }
    match (normalize_namespace(namespace), method) {
        ("std.foundation", method) => method.into(),
        ("std.native.Maths", method) => format!("std.native.Maths/{method}"),
        ("std.native.Numbers", method) => format!("std.native.Numbers/{method}"),
        ("std.native.Bits", method) => format!("std.native.Bits/{method}"),
        ("std.native.String", method) => format!("str/{method}"),
        ("std.native.Bytes", "new") => "bytes".into(),
        ("std.native.Bytes", "instance?") => "bytes?".into(),
        ("std.native.Bytes", method) => format!("bytes/{method}"),
        ("std.native.File", method) => format!("file/{method}"),
        ("std.native.Socket", method) => format!("socket/{method}"),
        ("std.native.Promise", "run") => "promise/run".into(),
        ("std.native.Promise", "instance?") => "promise?".into(),
        ("std.native.Promise", method) => format!("promise/{method}"),
        ("std.native.Coroutine", "instance?") => "std.foundation.coroutine/coroutine?".into(),
        ("std.native.Coroutine", method) => format!("std.foundation.coroutine/{method}"),
        ("std.native.Arr", "new") => "array".into(),
        ("std.native.Arr", "instance?") => "array?".into(),
        ("std.native.Obj", "new") => "object".into(),
        ("std.native.Obj", "instance?") => "object?".into(),
        ("std.native.Runtime", method) => method.into(),
        ("std.native.Printer", method) => method.into(),
        ("std.native.Edn", method) => format!("std.native.Edn/{method}"),
        ("std.native.Json", method) => format!("std.native.Json/{method}"),
        ("std.native.RegExp", "instance?") => "regexp?".into(),
        ("std.native.UUID", "instance?") => "uuid?".into(),
        ("std.native.Error", method) => format!("std.native.Error/{method}"),
        ("std.native.Iter", method) => format!("std.native.Iter/{method}"),
        ("std.lib.string", method) => format!("str/{method}"),
        ("std.lib.promise", "then") => "promise/then".into(),
        ("std.lib.promise", "catch") => "promise/catch".into(),
        ("std.lib.promise", method) => format!("promise/{method}"),
        ("std.lib.bytes", method) => format!("bytes/{method}"),
        ("std.lib.socket", method) => format!("socket/{method}"),
        ("std.lib.file", method) => format!("file/{method}"),
        (namespace, method) => format!("{namespace}/{method}"),
    }
}

pub(crate) fn canonical_native_call(name: &str) -> String {
    match name.rsplit_once('/') {
        Some((namespace, method)) if namespace.starts_with("std.native.") => {
            canonical(namespace, method)
        }
        _ => name.to_owned(),
    }
}

#[cfg(test)]
#[path = "generated/tests.rs"]
mod tests;
