from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one match in {path}, found {count}: {old!r}")
    file.write_text(text.replace(old, new, 1))


replace_once(
    "core/java/src/main/java/hara/truffle/SessionKernel.java",
    "    private static Object transferValue(Value value) {",
    "    static Object transferValue(Value value) {",
)

replace_once(
    "core/java/src/main/java/hara/truffle/Main.java",
    "          Object results = Parser.LispReader.readString(value.asString(), null);",
    '''          Object results =
              value.isString()
                  ? Parser.LispReader.readString(value.asString(), null)
                  : SessionKernel.Session.transferValue(value);''',
)
