package hara.truffle;

import hara.lang.data.Keyword;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.Deque;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

/** Provider-neutral projections used by the Java std.native.File runtime boundary. */
final class FilesystemFileEffects {
  private FilesystemFileEffects() {}

  static CompletionStage<Object> read(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    return filesystem.read(context, path).thenApply(bytes -> bytes.clone());
  }

  static CompletionStage<Object> exists(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    CompletableFuture<Object> result = new CompletableFuture<>();
    filesystem
        .stat(context, path)
        .whenComplete(
            (entry, error) -> {
              if (error == null) {
                result.complete(Boolean.TRUE);
                return;
              }
              Throwable failure = unwrap(error);
              if (failure instanceof FilesystemException filesystemError
                  && "not-found".equals(filesystemError.code())) {
                result.complete(Boolean.FALSE);
              } else {
                result.completeExceptionally(failure);
              }
            });
    return result;
  }

  static CompletionStage<Object> stat(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    return filesystem.stat(context, path).thenApply(FilesystemFileEffects::entryValue);
  }

  static CompletionStage<Object> entries(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    return collectEntries(filesystem, context, path)
        .thenApply(
            values ->
                HaraPersistentValues.normalize(
                    values.stream().map(FilesystemFileEffects::entryValue).toList()));
  }

  static CompletionStage<Object> list(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    return collectEntries(filesystem, context, path)
        .thenApply(
            values ->
                HaraPersistentValues.normalize(
                    values.stream().map(IFilesystem.Entry::path).toList()));
  }

  static CompletionStage<Object> walk(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    return filesystem
        .stat(context, path)
        .thenCompose(
            root -> {
              if (root.type() != IFilesystem.EntryType.DIRECTORY) {
                return CompletableFuture.completedFuture(
                    HaraPersistentValues.normalize(List.of(root.path())));
              }
              return walkDirectories(
                  filesystem,
                  context,
                  new ArrayDeque<>(List.of(root.path())),
                  new ArrayList<>());
            });
  }

  static CompletionStage<Object> write(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      String path,
      byte[] bytes,
      IFilesystem.WriteOptions options,
      IFilesystem.MutationContext mutation) {
    Objects.requireNonNull(bytes, "filesystem bytes");
    return filesystem
        .write(context, path, bytes.clone(), options, mutation)
        .thenApply(IFilesystem.Mutation::path);
  }

  static CompletionStage<Object> mkdir(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      String path,
      IFilesystem.MkdirOptions options,
      IFilesystem.MutationContext mutation) {
    return filesystem
        .mkdir(context, path, options, mutation)
        .thenApply(IFilesystem.Mutation::path);
  }

  static CompletionStage<Object> delete(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      String path,
      IFilesystem.DeleteOptions options,
      IFilesystem.MutationContext mutation) {
    return filesystem
        .delete(context, path, options, mutation)
        .thenApply(IFilesystem.Mutation::path);
  }

  static CompletionStage<Object> copy(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      String source,
      String target,
      IFilesystem.CopyOptions options,
      IFilesystem.MutationContext mutation) {
    return filesystem
        .copy(context, source, target, options, mutation)
        .thenApply(IFilesystem.Mutation::path);
  }

  static CompletionStage<Object> move(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      String source,
      String target,
      IFilesystem.MoveOptions options,
      IFilesystem.MutationContext mutation) {
    return filesystem
        .move(context, source, target, options, mutation)
        .thenApply(IFilesystem.Mutation::path);
  }

  private static CompletionStage<List<IFilesystem.Entry>> collectEntries(
      IFilesystem filesystem, IFilesystem.CallContext context, String path) {
    return collectPage(filesystem, context, path, null, new ArrayList<>());
  }

  private static CompletionStage<List<IFilesystem.Entry>> collectPage(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      String path,
      String token,
      ArrayList<IFilesystem.Entry> output) {
    return filesystem
        .entriesPage(
            context,
            path,
            new IFilesystem.PageRequest(token, IFilesystem.PageRequest.DEFAULT_LIMIT))
        .thenCompose(
            page -> {
              output.addAll(page.entries());
              if (page.nextToken() == null) {
                output.sort(Comparator.comparing(IFilesystem.Entry::path));
                return CompletableFuture.completedFuture(List.copyOf(output));
              }
              return collectPage(filesystem, context, path, page.nextToken(), output);
            });
  }

  private static CompletionStage<Object> walkDirectories(
      IFilesystem filesystem,
      IFilesystem.CallContext context,
      Deque<String> directories,
      ArrayList<String> output) {
    String directory = directories.pollFirst();
    if (directory == null) {
      output.sort(String::compareTo);
      return CompletableFuture.completedFuture(HaraPersistentValues.normalize(output));
    }
    return collectEntries(filesystem, context, directory)
        .thenCompose(
            entries -> {
              for (IFilesystem.Entry entry : entries) {
                if (entry.type() == IFilesystem.EntryType.DIRECTORY) {
                  directories.addLast(entry.path());
                } else {
                  output.add(entry.path());
                }
              }
              return walkDirectories(filesystem, context, directories, output);
            });
  }

  private static Object entryValue(IFilesystem.Entry entry) {
    LinkedHashMap<Object, Object> extensions = new LinkedHashMap<>();
    for (Map.Entry<String, Object> extension : entry.extensions().entrySet()) {
      extensions.put(Keyword.create(extension.getKey()), extension.getValue());
    }
    if (entry.id() != null) extensions.put(Keyword.create("file/id"), entry.id());
    if (entry.revision() != null) {
      extensions.put(Keyword.create("file/revision"), entry.revision());
    }
    if (entry.capabilities() != null) {
      Set<Keyword> capabilities =
          entry.capabilities().values().stream()
              .map(capability -> Keyword.create(capability.keyword()))
              .collect(java.util.stream.Collectors.toUnmodifiableSet());
      extensions.put(Keyword.create("provider/capabilities"), capabilities);
    }
    LinkedHashMap<Object, Object> value = new LinkedHashMap<>();
    value.put(Keyword.create("path"), entry.path());
    value.put(Keyword.create("name"), entry.name());
    value.put(Keyword.create("type"), Keyword.create(entry.type().keyword()));
    value.put(Keyword.create("size"), entry.size());
    value.put(Keyword.create("modified-at"), entry.modifiedAt());
    value.put(Keyword.create("extensions"), extensions);
    return HaraPersistentValues.normalize(value);
  }

  private static Throwable unwrap(Throwable error) {
    Throwable current = error;
    while ((current instanceof java.util.concurrent.CompletionException
            || current instanceof java.util.concurrent.ExecutionException)
        && current.getCause() != null) {
      current = current.getCause();
    }
    return current;
  }
}
