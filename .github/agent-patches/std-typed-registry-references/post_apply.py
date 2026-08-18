from pathlib import Path

root = Path(__file__).resolve().parents[3]
payload = Path(__file__).resolve().parent

(root / ".github/workflows/std-typed-schema.yml").write_text(
    (payload / "std-typed-schema-final.yml").read_text()
)
(root / "core/lib/test/std/typed/registry_probe.hal").write_text(
    (payload / "registry_probe.hal").read_text()
)
