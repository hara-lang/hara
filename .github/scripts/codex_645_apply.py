from __future__ import annotations

import pathlib

ROOT = pathlib.Path.cwd()


def load(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def save(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


def replace_exact(path: str, old: str, new: str, count: int = 1) -> None:
    source = load(path)
    actual = source.count(old)
    if actual != count:
        raise RuntimeError(f"{path}: expected {count} occurrences, found {actual}: {old[:120]!r}")
    save(path, source.replace(old, new))


def matching_brace(source: str, opening: int) -> int:
    if source[opening] != "{":
        raise ValueError("opening index is not a brace")
    depth = 0
    state = "normal"
    escaped = False
    index = opening
    while index < len(source):
        ch = source[index]
        nxt = source[index + 1] if index + 1 < len(source) else ""
        if state == "line":
            if ch == "\n":
                state = "normal"
        elif state == "block":
            if ch == "*" and nxt == "/":
                state = "normal"
                index += 1
        elif state == "string":
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                state = "normal"
        elif state == "char":
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == "'":
                state = "normal"
        else:
            if ch == "/" and nxt == "/":
                state = "line"
                index += 1
            elif ch == "/" and nxt == "*":
                state = "block"
                index += 1
            elif ch == '"':
                state = "string"
            elif ch == "'":
                state = "char"
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return index
        index += 1
    raise RuntimeError("unterminated brace-delimited block")


def remove_method(path: str, signature: str) -> None:
    source = load(path)
    if source.count(signature) != 1:
        raise RuntimeError(f"{path}: expected one method signature {signature!r}")
    signature_index = source.index(signature)
    start = source.rfind("\n", 0, signature_index) + 1
    opening = source.index("{", signature_index + len(signature) - 1)
    closing = matching_brace(source, opening)
    end = closing + 1
    while end < len(source) and source[end] == "\n":
        end += 1
    save(path, source[:start] + source[end:])


def remove_xml_plugin(path: str, marker: str) -> None:
    source = load(path)
    if source.count(marker) != 1:
        raise RuntimeError(f"{path}: expected one XML marker {marker!r}")
    marker_index = source.index(marker)
    start = source.rfind("<plugin>", 0, marker_index)
    end = source.find("</plugin>", marker_index)
    if start < 0 or end < 0:
        raise RuntimeError(f"{path}: cannot find plugin containing {marker!r}")
    start = source.rfind("\n", 0, start) + 1
    end += len("</plugin>")
    if end < len(source) and source[end] == "\n":
        end += 1
    save(path, source[:start] + source[end:])


def patch_rust_bootstrap() -> None:
    path = "core/rust/src/lib.rs"
    source = load(path)
    marker = 'include_bytes!("../assets/std.foundation.hbx")'
    if source.count(marker) != 1:
        raise RuntimeError(f"{path}: expected one Foundation include_bytes bootstrap")
    marker_index = source.index(marker)
    cfg_text = '#[cfg(feature = "bytecode-vm")]'
    cfg_index = source.rfind(cfg_text, 0, marker_index)
    if cfg_index < 0:
        raise RuntimeError(f"{path}: missing bytecode bootstrap cfg")
    start = source.rfind("\n", 0, cfg_index) + 1
    opening = source.find("{", cfg_index + len(cfg_text), marker_index)
    if opening < 0:
        raise RuntimeError(f"{path}: missing bytecode bootstrap block")
    closing = matching_brace(source, opening)
    if not (opening < marker_index < closing):
        raise RuntimeError(f"{path}: Foundation bundle marker is outside selected cfg block")
    end = closing + 1
    if end < len(source) and source[end] == "\n":
        end += 1
    source = source[:start] + source[end:]

    source_cfg = '#[cfg(not(feature = "bytecode-vm"))]'
    source_cfg_index = source.find(source_cfg, start)
    if source_cfg_index < 0 or source_cfg_index - start > 256:
        raise RuntimeError(f"{path}: source fallback block did not follow bytecode bootstrap")
    line_end = source.find("\n", source_cfg_index)
    if line_end < 0:
        raise RuntimeError(f"{path}: malformed source fallback cfg")
    source = source[:source_cfg_index] + source[line_end + 1 :]
    if "std.foundation.hbx" in source:
        raise RuntimeError(f"{path}: stale Foundation HBX reference remains")
    save(path, source)


def patch_java_context() -> None:
    path = "core/java/src/main/java/hara/truffle/HaraContext.java"
    source = load(path)
    replacements = [
        ("import hara.truffle.bytecode.HbcProgram;\n", ""),
        (
            "  private final HaraLibraryLoader libraryLoader = new HaraLibraryLoader();\n"
            "  private final HbxBundleLibrary bytecodeLibrary =\n"
            "      new HbxBundleLibrary(HaraContext.class.getClassLoader());\n",
            "  private final HaraLibraryLoader libraryLoader = new HaraLibraryLoader();\n",
        ),
        (
            "    hideIteratorImplementationBindings();\n"
            "    for (String namespace : bytecodeLibrary.namespaces()) {\n"
            "      namespaceStates.put(namespace, NamespaceLoadState.UNLOADED);\n"
            "    }\n"
            "    currentNamespace = namespace(\"user\");\n",
            "    hideIteratorImplementationBindings();\n"
            "    currentNamespace = namespace(\"user\");\n",
        ),
        (
            "      if (bytecodeLibrary.available()) {\n"
            "        for (HbxBundleLibrary.Module module : bytecodeLibrary.eagerModules()) {\n"
            "          requiredNamespace(module.namespace());\n"
            "        }\n"
            "      } else {\n"
            "        libraryLoader.installEagerResources(this);\n"
            "      }\n",
            "      libraryLoader.installEagerResources(this);\n",
        ),
        (
            "        && (projectSource == null && bytecodeLibrary.provides(target)\n"
            "            || !libraryLoader.provides(target)\n"
            "            || sourceNamespaceLoaded(target)))",
            "        && (!libraryLoader.provides(target) || sourceNamespaceLoaded(target)))",
        ),
        ("      if (loaded == null) loaded = loadBytecodeNamespace(target);\n", ""),
        (
            "        && \"hara.lang\".equals(currentProject.name().display())\n"
            "        && bytecodeLibrary.provides(target)) {",
            "        && \"hara.lang\".equals(currentProject.name().display())\n"
            "        && libraryLoader.provides(target)) {",
        ),
        (
            "        && (bytecodeLibrary.provides(namespaceName) || libraryLoader.provides(namespaceName))\n",
            "        && libraryLoader.provides(namespaceName)\n",
        ),
    ]
    for old, new in replacements:
        if source.count(old) != 1:
            raise RuntimeError(
                f"{path}: expected one Java bootstrap fragment, found {source.count(old)}: {old[:100]!r}"
            )
        source = source.replace(old, new)
    save(path, source)
    remove_method(path, "  private HaraNamespace loadBytecodeNamespace(String target) {")
    source = load(path)
    stale = [
        marker
        for marker in ("HbxBundleLibrary", "bytecodeLibrary", "loadBytecodeNamespace", "HbcProgram")
        if marker in source
    ]
    if stale:
        raise RuntimeError(f"{path}: stale bytecode bootstrap markers remain: {stale}")


def patch_cargo() -> None:
    replace_exact(
        "core/rust/Cargo.toml",
        '[[bin]]\n'
        'name = "hara-foundation-artifact"\n'
        'path = "src/bin/hara-foundation-artifact.rs"\n'
        'required-features = ["bytecode-vm"]\n\n',
        "",
    )


def patch_java_tests() -> None:
    remove_method(
        "core/java/src/test/java/hara/truffle/HaraBytecodeToolTest.java",
        "  public void disassemblesTheRustProducedAlphaBundle() throws Exception {",
    )
    codec = "core/java/src/test/java/hara/truffle/bytecode/HbcCodecTest.java"
    remove_method(
        codec,
        "  public void decodesEveryArtifactInTheTrackedRustFoundationBundle() throws Exception {",
    )
    source = load(codec)
    for old, new in (
        ("import java.security.MessageDigest;\n", ""),
        (
            "  public void automaticallyLoadsEagerAndRequiredRustFoundationModules() throws Exception {",
            "  public void automaticallyLoadsEagerAndRequiredCanonicalHalSources() throws Exception {",
        ),
        ('(do (ns hbx.referral) (vector 1 2))', '(do (ns source.referral) (vector 1 2))'),
    ):
        if source.count(old) != 1:
            raise RuntimeError(f"{codec}: expected one test fragment {old!r}")
        source = source.replace(old, new)
    if "std.foundation.hbx" in source:
        raise RuntimeError(f"{codec}: stale Foundation bundle test remains")
    save(codec, source)


def patch_pom() -> None:
    remove_xml_plugin("core/java/pom.xml", "<id>embed-foundation-hbx-alpha</id>")
    if "std.foundation.hbx" in load("core/java/pom.xml"):
        raise RuntimeError("core/java/pom.xml: stale Foundation bundle resource remains")


def patch_lang_runtime_workflow() -> None:
    path = ".github/workflows/lang-runtime.yml"
    source = load(path)
    old_paths = (
        "      - 'core/rust/assets/std.foundation.hbx'\n"
        "      - 'core/rust/src/bin/hara-foundation-artifact.rs'\n"
    )
    new_paths = (
        "      - 'core/rust/src/lib.rs'\n"
        "      - 'core/rust/tests/source_only_foundation.rs'\n"
        "      - 'core/java/src/main/java/hara/truffle/HaraContext.java'\n"
        "      - 'core/java/src/main/java/hara/truffle/HaraLibraryLoader.java'\n"
        "      - 'scripts/runtime/source_only_foundation.py'\n"
        "      - 'scripts/runtime/source_only_foundation_test.py'\n"
    )
    if source.count(old_paths) != 2:
        raise RuntimeError(f"{path}: expected two legacy HBX path-filter pairs")
    source = source.replace(old_paths, new_paths)

    old_steps = """      - name: Generate the current standard-library HBX bundle
        run: cargo run --quiet --manifest-path core/rust/Cargo.toml --features bytecode-vm --bin hara-foundation-artifact -- generate
      - name: Upload the generated standard-library HBX bundle
        uses: actions/upload-artifact@v4
        with:
          name: std-foundation-hbx-${{ github.event.pull_request.head.sha || github.sha }}
          path: core/rust/assets/std.foundation.hbx
          if-no-files-found: error
"""
    new_steps = """      - name: Enforce source-only Foundation development runtimes
        run: |
          python3 scripts/runtime/source_only_foundation_test.py
          python3 scripts/runtime/source_only_foundation.py
"""
    if source.count(old_steps) != 1:
        raise RuntimeError(f"{path}: expected one generated Foundation HBX workflow block")
    source = source.replace(old_steps, new_steps)
    build_step = """      - name: Build Hara Truffle runtime
        run: mvn -B -Ptruffle -DskipTests package --file core/java/pom.xml
"""
    audited_build = build_step + """      - name: Verify the Java classpath remains source-only
        run: python3 scripts/runtime/source_only_foundation.py
"""
    if source.count(build_step) != 1:
        raise RuntimeError(f"{path}: expected one Truffle build step")
    source = source.replace(build_step, audited_build)
    if "hara-foundation-artifact" in source or "std-foundation-hbx-" in source:
        raise RuntimeError(f"{path}: stale generated bundle workflow remains")
    save(path, source)


def patch_main_workflow() -> None:
    path = ".github/workflows/main.yml"
    source = load(path)
    layout_step = """    - name: Check Rust module layout
      run: bash core/rust/scripts/check-layout.sh
"""
    audit_step = layout_step + """    - name: Enforce source-only Foundation development runtimes
      run: |
        python3 scripts/runtime/source_only_foundation_test.py
        python3 scripts/runtime/source_only_foundation.py
"""
    if source.count(layout_step) != 1:
        raise RuntimeError(f"{path}: expected one Rust layout step")
    source = source.replace(layout_step, audit_step)

    maven_step = """    - name: Build and test with Maven
      run: mvn -B -Ptruffle package --file core/java/pom.xml
"""
    maven_audit = maven_step + """    - name: Verify the JVM classpath remains source-only
      run: python3 scripts/runtime/source_only_foundation.py
"""
    if source.count(maven_step) != 1:
        raise RuntimeError(f"{path}: expected one Maven build step")
    source = source.replace(maven_step, maven_audit)

    native_build = """    - name: Build the Truffle Native Image
      env:
        HARA_NATIVE_USE_FALLBACK_RUNTIME: 'true'
      run: ./scripts/runtime/build-truffle-native
"""
    native_audit = native_build + """    - name: Verify the Native Image build remains source-only
      run: python3 scripts/runtime/source_only_foundation.py
"""
    if source.count(native_build) != 1:
        raise RuntimeError(f"{path}: expected one Native Image build step")
    source = source.replace(native_build, native_audit)

    bundle_lines = """        core/target/hara-truffle bytecode run core/rust/assets/std.foundation.hbx > /dev/null
        core/target/hara-truffle bytecode disassemble core/rust/assets/std.foundation.hbx > core/target/native-image-hbx-alpha-disassembly.txt
        test "$(grep -c '^module ' core/target/native-image-hbx-alpha-disassembly.txt)" -eq "$(wc -l < core/rust/standard-library.namespaces)"
        grep -q '^module std.foundation$' core/target/native-image-hbx-alpha-disassembly.txt
        grep -q '^module lang.core$' core/target/native-image-hbx-alpha-disassembly.txt
"""
    if source.count(bundle_lines) != 1:
        raise RuntimeError(f"{path}: expected one Native Image Foundation bundle block")
    source = source.replace(bundle_lines, "")
    if "core/rust/assets/std.foundation.hbx" in source:
        raise RuntimeError(f"{path}: stale development Foundation bundle reference remains")
    save(path, source)


def delete_retired_files() -> None:
    for path in (
        "core/rust/assets/std.foundation.hbx",
        "core/rust/src/bin/hara-foundation-artifact.rs",
        "core/java/src/main/java/hara/truffle/HbxBundleLibrary.java",
    ):
        target = ROOT / path
        if not target.exists():
            raise RuntimeError(f"{path}: expected retired file to exist")
        target.unlink()


def main() -> None:
    patch_rust_bootstrap()
    patch_java_context()
    patch_cargo()
    patch_pom()
    patch_java_tests()
    patch_lang_runtime_workflow()
    patch_main_workflow()
    delete_retired_files()

    forbidden_runtime_inputs = (
        "core/rust/src/lib.rs",
        "core/rust/Cargo.toml",
        "core/java/pom.xml",
        "core/java/src/main/java/hara/truffle/HaraContext.java",
        ".github/workflows/lang-runtime.yml",
        ".github/workflows/main.yml",
    )
    for path in forbidden_runtime_inputs:
        if "std.foundation.hbx" in load(path):
            raise RuntimeError(f"{path}: stale Foundation development bundle reference")
    print("source-only Foundation patch applied")


if __name__ == "__main__":
    main()
