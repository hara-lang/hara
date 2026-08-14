from pathlib import Path

path = Path(".github/agent/runtime-aware-projects.py")
text = path.read_text()
needle = 'org.postgresql/postgresql {:version "42.7.7"}'
replacement = '"org.postgresql/postgresql" {:version "42.7.7"}'
count = text.count(needle)
if count != 2:
    raise SystemExit(f"expected two portable Maven fixture keys, found {count}")
path.write_text(text.replace(needle, replacement))
