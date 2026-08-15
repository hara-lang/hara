# Mounted filesystem APIs

Hara exposes a mounted, capability-backed filesystem rather than ambient host filesystem access. The mounted filesystem root is the logical path `/`; it is never the host operating-system root.

The filesystem surface has three layers:

| Layer | Role |
| --- | --- |
| `std.native.File` / `File` | asynchronous provider boundary and compatibility operations |
| `std.fs.path` | synchronous, deterministic logical-path algebra |
| `std.fs` and `std.fs.walk` | portable promise-based filesystem operations and traversal |

## Logical paths

Every observable filesystem path is a canonical absolute logical path:

```clojure
(std.fs.path/normalise "src//./main.hal")
;; => "/src/main.hal"
```

The common path contract is:

- `/` is the attached provider root.
- Relative input is accepted for convenience and resolved from `/`.
- `/` is the only separator, on every host platform.
- Duplicate separators and `.` segments are removed.
- `..` is resolved, but escaping above `/` is rejected.
- `~`, host working directories, Windows drives, and host-specific separators are not expanded.
- Host paths are created only inside a provider implementation.

`std.fs.path` provides the pure operations `normalise`, `join`, `resolve`, `parent`, `root`, `file-name`, `segments`, `relativize`, `subpath`, `suffix`, `add-suffix`, `remove-suffix`, and `replace-suffix`.

## Promise and error contract

A valid filesystem effect call immediately returns a promise. Capability failures, missing entries, permission failures, and provider I/O failures reject that promise. Arity and argument-type errors remain synchronous.

Provider rejections are `ExceptionInfo` values with stable data:

```clojure
{:error/code     :file/not-found
 :file/operation :read
 :file/path      "/missing"
 :file/target    nil}
```

Portable code should branch on `:error/code`, not host error text. Stable codes include:

- `:file/not-found`
- `:file/already-exists`
- `:file/invalid-path`
- `:file/outside-root`
- `:file/not-directory`
- `:file/is-directory`
- `:file/directory-not-empty`
- `:file/permission-denied`
- `:file/unsupported`
- `:file/io`

`exists?` resolves `false` only for not-found. Other failures reject.

## Native provider boundary

The native static object is available as `File` and has the runtime identity `std.native.File`. Its effectful operations are:

```text
read write exists? stat entries list walk mkdir delete
copy move temp-file temp-directory
```

The native options are:

```clojure
write          {:mode :create|:replace|:append, :parents? false}
mkdir          {:parents? true, :exists-ok? true}
delete         {:missing-ok? false}
copy           {:replace? false, :parents? false, :preserve-modified? false}
move           {:replace? false, :parents? false, :atomic? false}
temp-file      {:prefix "tmp", :suffix ""}
temp-directory {:prefix "tmp"}
```

`stat` and `entries` return no-follow metadata maps:

```clojure
{:path        "/dir/item"
 :name        "item"
 :type        :file | :directory | :symlink | :other
 :size        42 | nil
 :modified-at 1780000000000
 :extensions  {}}
```

`:size` is present only for a regular file. `entries` returns immediate entries sorted by canonical path. `list` remains a sorted path-string projection of `entries`.

`parent`, `join`, `resolve`, `list`, and `walk` remain callable as compatibility operations. New portable code should use `std.fs.path`, `std.fs/entries`, and `std.fs.walk/walk` instead.

## Portable `std.fs`

Require the portable facade and path algebra explicitly:

```clojure
(ns example.files
  (:require [std.fs :as fs]
            [std.fs.path :as path]
            [std.fs.walk :as walk]))
```

The portable facade provides:

```text
stat entries list select
exists? file? directory? symlink?
read-bytes write-bytes
mkdir create-directory
temp-file temp-directory
copy-single copy copy-into move delete
```

Examples:

```clojure
(deref (fs/create-directory "/build" {:parents? true}))
;; => "/build"

(deref (fs/write-bytes "/build/output.bin" bytes
                       {:mode :create :parents? true}))
;; => "/build/output.bin"

(deref (fs/copy "/src" "/backup/src" {:parents? true}))
;; => {"/src" "/backup/src", ...}

(deref (fs/delete "/backup" {:recursive? true}))
;; => ["/backup/src/..." ... "/backup"]
```

Copy defaults are non-destructive:

```clojure
{:replace? false
 :parents? false
 :preserve-modified? false}
```

Delete defaults are also non-destructive:

```clojure
{:recursive? false
 :missing-ok? false}
```

A simple mutation resolves to its canonical logical path. Recursive copy resolves to a source-to-target map. Recursive delete resolves to deleted paths in post-order. Recursive operations reject on the first failure; completed provider mutations are not rolled back or returned as partial success.

`copy-into` copies the source beneath a destination directory using the source filename. The mounted root has no filename, so `(fs/copy-into "/" target)` rejects with `:file/invalid-path`.

## Deterministic traversal

`std.fs.walk/walk` traverses `File/entries` in canonical lexical order. It never follows symbolic links.

```clojure
(deref
 (walk/walk "/src"
            {:include-root? false
             :max-depth 3
             :include (fn [entry] (= (:type entry) :file))
             :exclude (fn [entry] (= (:name entry) ".cache"))}))
```

Options are:

```clojure
{:include-root? false
 :max-depth nil
 :include nil
 :exclude nil}
```

An excluded directory is pruned, not merely omitted from the result. `max-depth` limits descent relative to the requested root.

## Security and portability invariants

Providers must preserve these invariants:

- Logical `..` cannot escape the mounted root.
- Host absolute paths and drive syntax cannot select ambient host files.
- Metadata and traversal do not follow links.
- A link or ancestor link cannot be used to escape the provider root.
- Recursive copy and delete treat a symbolic link as an entry, never as a directory to traverse.
- Copying or moving a directory into itself or a descendant is rejected.
- The mounted root cannot be deleted.
- Temporary entries are created atomically beneath the explicit logical parent.

The JVM, Rust-native/WASI, memory, and unsupported providers are expected to expose the same logical results and stable error data even though their host implementations differ.
