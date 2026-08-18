from pathlib import Path
import sys

OLD = "           output (vec (keys (:registry/entries registry-value)))]"
NEW = "           output (vec (sort (keys (:registry/entries registry-value))))]"

for argument in sys.argv[1:]:
    path = Path(argument)
    text = path.read_text()
    count = text.count(OLD)
    if count != 1:
        raise SystemExit(f"{path}: expected one registry names marker, found {count}")
    path.write_text(text.replace(OLD, NEW, 1))
