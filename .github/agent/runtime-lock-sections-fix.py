from pathlib import Path

path = Path("core/rust/src/project.rs")
text = path.read_text()
old = '''        &project
            .runtime_target_path
            .as_ref()
            .map(portable_path)
            .unwrap_or_default(),'''
new = '''        &project
            .runtime_target_path
            .as_deref()
            .map(portable_path)
            .unwrap_or_default(),'''
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected one runtime target digest coercion, found {count}")
path.write_text(text.replace(old, new, 1))
