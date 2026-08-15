use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_hal(directory: &Path, output: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        .map(|entry| entry.expect("cannot read HAL resource entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_hal(&path, output);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("hal")
            && !is_editor_artifact(&path)
        {
            output.push(path);
        }
    }
}

/// Editor lock and autosave files (Emacs `.#name` / `#name#`) are not HAL
/// sources; lock symlinks are often dangling and must not break the build.
fn is_editor_artifact(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.starts_with(".#") || (name.starts_with('#') && name.ends_with('#'))
}

fn declared_namespace(source: &str, path: &Path) -> String {
    for line in source.lines() {
        let line = line.trim_start();
        let remainder = line
            .strip_prefix("(ns ")
            .or_else(|| line.strip_prefix("(ns+ "));
        if let Some(remainder) = remainder {
            let namespace = remainder
                .split(|character: char| character.is_whitespace() || character == ')')
                .next()
                .unwrap_or_default();
            if !namespace.is_empty() {
                return namespace.to_owned();
            }
        }
    }
    panic!(
        "{} does not declare an ns or ns+ namespace on its own line",
        path.display()
    );
}

fn standard_library_namespace(namespace: &str) -> bool {
    namespace == "db"
        || ["std.", "code.", "lang.", "db."]
            .iter()
            .any(|prefix| namespace.starts_with(prefix))
}

fn source_roots(manifest: &Path) -> Vec<(PathBuf, PathBuf)> {
    let canonical = manifest.join("../lib");
    let canonical_roots = [canonical.join("src"), canonical.join("src-lang")];
    if canonical_roots.iter().all(|root| root.is_dir()) {
        return vec![
            (canonical_roots[0].clone(), PathBuf::from("lib/src")),
            (canonical_roots[1].clone(), PathBuf::from("lib/src")),
        ];
    }

    let packaged = manifest.join("hal-src");
    if packaged.is_dir() {
        return vec![(packaged, PathBuf::from("lib/src"))];
    }

    panic!(
        "HAL sources are unavailable: build from the Hara repository or run \
         scripts/runtime/sync-rust-hal-src before packaging"
    );
}

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    // Repository builds embed canonical core/lib sources directly. Published
    // Cargo archives cannot include files above the crate root, so release
    // preparation materializes the same sources into the ignored hal-src
    // directory and explicitly includes that directory in the package.
    let source_roots = source_roots(&manifest);
    let inventory_path = manifest.join("standard-library.namespaces");
    let hta_path = manifest.join("src/hta.rs");
    for (source_root, _) in &source_roots {
        println!("cargo:rerun-if-changed={}", source_root.display());
    }
    println!("cargo:rerun-if-changed={}", inventory_path.display());
    println!("cargo:rerun-if-changed={}", hta_path.display());

    let mut resources = BTreeMap::new();
    let mut embedded_paths = BTreeMap::new();
    for (source_root, embedded_root) in &source_roots {
        let mut paths = Vec::new();
        collect_hal(source_root, &mut paths);
        for path in paths {
            println!("cargo:rerun-if-changed={}", path.display());
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            let namespace = declared_namespace(&source, &path);
            let relative = embedded_root.join(
                path.strip_prefix(source_root)
                    .expect("HAL resource must be inside its source root"),
            );
            if let Some(previous) = embedded_paths.insert(relative.clone(), path.clone()) {
                panic!(
                    "duplicate embedded HAL path {}: {} and {}",
                    relative.display(),
                    previous.display(),
                    path.display()
                );
            }
            if let Some((previous, _)) =
                resources.insert(namespace.clone(), (path.clone(), relative))
            {
                panic!(
                    "duplicate HAL namespace {namespace}: {} and {}",
                    previous.display(),
                    path.display()
                );
            }
        }
    }

    let expected_inventory = fs::read_to_string(&inventory_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", inventory_path.display()))
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let actual_inventory = resources
        .keys()
        .filter(|namespace| standard_library_namespace(namespace))
        .cloned()
        .collect::<Vec<_>>();
    if expected_inventory != actual_inventory {
        panic!(
            "{} is stale; expected exact embedded standard-library inventory:\n{}",
            inventory_path.display(),
            actual_inventory.join("\n")
        );
    }

    // Snapshot and bytecode artifacts use the pure HTA value codec in browser
    // builds too. Native builds keep the ordinary crate-root module; the
    // generated declaration supplies the same source only where lib.rs gates it.
    let hta_path = hta_path
        .canonicalize()
        .unwrap_or_else(|error| panic!("cannot resolve {}: {error}", hta_path.display()));
    let mut generated = format!(
        "#[cfg(target_arch = \"wasm32\")]\n#[path = {:?}]\npub mod hta;\n\npub(crate) static EMBEDDED_HAL_RESOURCES: &[(&str, &str, &str)] = &[\n",
        hta_path.to_string_lossy()
    );
    for (namespace, (path, relative)) in resources {
        let path = path
            .canonicalize()
            .unwrap_or_else(|error| panic!("cannot resolve {}: {error}", path.display()));
        let relative = relative.to_string_lossy().replace('\\', "/");
        generated.push_str(&format!(
            "    ({namespace:?}, {relative:?}, include_str!({path:?})),\n",
            namespace = namespace,
            relative = relative,
            path = path.to_string_lossy()
        ));
    }
    generated.push_str("];\n");
    generated
        .push_str("#[cfg(test)]\npub(crate) static STANDARD_LIBRARY_INVENTORY: &[&str] = &[\n");
    for namespace in expected_inventory {
        generated.push_str(&format!("    {namespace:?},\n"));
    }
    generated.push_str("];\n");

    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("embedded_hal.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", output.display()));
}
