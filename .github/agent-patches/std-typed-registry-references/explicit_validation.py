from pathlib import Path
import sys

OLD = '''  ([schema value]
   (binding [*reference-trail* #{}]
     (validate-normal (normalize schema) value [])))
  ([schema value registry-value]
   (binding [*registry* (registry/ensure registry-value)
             *reference-trail* #{}]
     (validate-normal (normalize schema) value []))))'''

NEW = '''  ([schema value]
   (validate-normal (normalize schema) value []))
  ([schema value registry-value]
   (let [registry-value (registry/ensure registry-value)]
     (validate-normal-with
      (normalize-with schema registry-value)
      value
      []
      registry-value
      #{}))))'''

for argument in sys.argv[1:]:
    path = Path(argument)
    text = path.read_text()
    count = text.count(OLD)
    if count != 1:
        raise SystemExit(f"{path}: expected one validate marker, found {count}")
    path.write_text(text.replace(OLD, NEW, 1))
