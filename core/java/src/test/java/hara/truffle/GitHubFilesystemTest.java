package hara.truffle;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import org.junit.Test;

public class GitHubFilesystemTest {
  @Test
  public void immutableCommitMountReadsAndEnumeratesWithoutFollowingLinks() {
    Fixture fixture = new Fixture();
    try {
      IFilesystem filesystem = fixture.open("read-only", fixture.client.initialCommit());
      IFilesystem.Descriptor descriptor = filesystem.descriptor();
      assertEquals("github", descriptor.kind());
      assertTrue(descriptor.readOnly());
      assertTrue(descriptor.capabilities().contains(IFilesystem.Capability.READ));
      assertFalse(descriptor.capabilities().contains(IFilesystem.Capability.WRITE));
      assertFalse(descriptor.display().contains("credential"));

      IFilesystem.Entry readme =
          join(filesystem.stat(context(), "/README.md"));
      assertEquals(IFilesystem.EntryType.FILE, readme.type());
      assertNull(readme.modifiedAt());
      assertEquals(fixture.client.readmeBlob(), readme.id());
      assertArrayEquals(
          "hello".getBytes(java.nio.charset.StandardCharsets.UTF_8),
          join(filesystem.read(context(), "/README.md")));

      IFilesystem.Entry link = join(filesystem.stat(context(), "/link"));
      assertEquals(IFilesystem.EntryType.SYMLINK, link.type());
      assertFailure("unsupported", () -> join(filesystem.read(context(), "/link")));
      assertEquals(
          IFilesystem.EntryType.OTHER,
          join(filesystem.stat(context(), "/vendor")).type());

      IFilesystem.EntryPage first =
          join(filesystem.entriesPage(context(), "/", new IFilesystem.PageRequest(null, 2)));
      assertEquals(List.of("/README.md", "/link"), paths(first.entries()));
      assertTrue(first.nextToken() != null && !first.nextToken().isBlank());
      IFilesystem.EntryPage second =
          join(
              filesystem.entriesPage(
                  context(), "/", new IFilesystem.PageRequest(first.nextToken(), 2)));
      assertEquals(List.of("/src", "/vendor"), paths(second.entries()));
      assertNull(second.nextToken());

      assertFailure(
          "permission-denied",
          () ->
              join(
                  filesystem.write(
                      context(),
                      "/new.bin",
                      new byte[] {1},
                      new IFilesystem.WriteOptions(IFilesystem.WriteMode.CREATE, false),
                      IFilesystem.MutationContext.none())));
    } finally {
      fixture.close();
    }
  }

  @Test
  public void writableBranchCommitsExactBytesAndRejectsStaleEntryRevisions() {
    Fixture fixture = new Fixture();
    try {
      IFilesystem first = fixture.open("commit", "heads/main");
      IFilesystem second = fixture.open("commit", "heads/main");
      String initialHead = fixture.client.head();
      IFilesystem.Entry stale = join(first.stat(context(), "/README.md"));

      IFilesystem.Mutation created =
          join(
              first.write(
                  context(),
                  "/data/new.bin",
                  new byte[] {0, 1, 0, (byte) 255},
                  new IFilesystem.WriteOptions(IFilesystem.WriteMode.CREATE, true),
                  IFilesystem.MutationContext.none()));
      assertNotEquals(initialHead, created.mountRevision());
      assertEquals(fixture.client.head(), created.mountRevision());
      assertArrayEquals(
          new byte[] {0, 1, 0, (byte) 255},
          join(first.read(context(), "/data/new.bin")));
      assertEquals(1, fixture.client.commitMessages().size());

      IFilesystem.Mutation replaced =
          join(
              first.write(
                  context(),
                  "/README.md",
                  "changed".getBytes(java.nio.charset.StandardCharsets.UTF_8),
                  new IFilesystem.WriteOptions(IFilesystem.WriteMode.REPLACE, false),
                  new IFilesystem.MutationContext(stale.revision(), null)));
      assertEquals(fixture.client.head(), replaced.mountRevision());
      assertEquals(2, fixture.client.commitMessages().size());

      assertFailure(
          "conflict",
          () ->
              join(
                  second.write(
                      context(),
                      "/README.md",
                      "stale".getBytes(java.nio.charset.StandardCharsets.UTF_8),
                      new IFilesystem.WriteOptions(IFilesystem.WriteMode.REPLACE, false),
                      new IFilesystem.MutationContext(stale.revision(), null))));
      assertEquals(2, fixture.client.commitMessages().size());
    } finally {
      fixture.close();
    }
  }

  @Test
  public void nonForcedRefMovementRejectsWithoutOverwritingTheNewHead() {
    Fixture fixture = new Fixture();
    try {
      IFilesystem filesystem = fixture.open("commit", "heads/main");
      fixture.client.moveBeforeNextUpdate();
      String competingHead = fixture.client.competingHead();
      assertFailure(
          "conflict",
          () ->
              join(
                  filesystem.write(
                      context(),
                      "/raced.bin",
                      new byte[] {7},
                      new IFilesystem.WriteOptions(IFilesystem.WriteMode.CREATE, false),
                      IFilesystem.MutationContext.none())));
      assertEquals(competingHead, fixture.client.head());
      assertFailure("not-found", () -> join(filesystem.stat(context(), "/raced.bin")));
    } finally {
      fixture.close();
    }
  }

  @Test
  public void copyMoveDeleteAndDirectoryPoliciesRemainExplicit() {
    Fixture fixture = new Fixture();
    try {
      IFilesystem filesystem = fixture.open("commit", "heads/main");
      join(
          filesystem.copy(
              context(),
              "/README.md",
              "/copy.md",
              new IFilesystem.CopyOptions(false, false, false),
              IFilesystem.MutationContext.none()));
      assertArrayEquals(
          "hello".getBytes(java.nio.charset.StandardCharsets.UTF_8),
          join(filesystem.read(context(), "/copy.md")));

      join(
          filesystem.move(
              context(),
              "/copy.md",
              "/moved.md",
              new IFilesystem.MoveOptions(false, false, false),
              IFilesystem.MutationContext.none()));
      assertFailure("not-found", () -> join(filesystem.stat(context(), "/copy.md")));
      join(
          filesystem.delete(
              context(),
              "/moved.md",
              new IFilesystem.DeleteOptions(false),
              IFilesystem.MutationContext.none()));
      assertFailure("not-found", () -> join(filesystem.stat(context(), "/moved.md")));

      join(
          filesystem.copy(
              context(),
              "/src",
              "/source-copy",
              new IFilesystem.CopyOptions(false, false, false),
              IFilesystem.MutationContext.none()));
      assertArrayEquals(
          "(+ 1 2)".getBytes(java.nio.charset.StandardCharsets.UTF_8),
          join(filesystem.read(context(), "/source-copy/main.hal")));
      join(
          filesystem.move(
              context(),
              "/source-copy",
              "/source-moved",
              new IFilesystem.MoveOptions(false, false, false),
              IFilesystem.MutationContext.none()));
      assertArrayEquals(
          "(+ 1 2)".getBytes(java.nio.charset.StandardCharsets.UTF_8),
          join(filesystem.read(context(), "/source-moved/main.hal")));
      assertFailure(
          "directory-not-empty",
          () ->
              join(
                  filesystem.delete(
                      context(),
                      "/source-moved",
                      new IFilesystem.DeleteOptions(false),
                      IFilesystem.MutationContext.none())));

      assertFailure(
          "unsupported",
          () ->
              join(
                  filesystem.mkdir(
                      context(),
                      "/empty",
                      new IFilesystem.MkdirOptions(false, false),
                      IFilesystem.MutationContext.none())));
      assertFailure(
          "unsupported",
          () ->
              join(
                  filesystem.copy(
                      context(),
                      "/README.md",
                      "/preserved.md",
                      new IFilesystem.CopyOptions(false, false, true),
                      IFilesystem.MutationContext.none())));
      assertFailure(
          "unsupported",
          () ->
              join(
                  filesystem.move(
                      context(),
                      "/README.md",
                      "/atomic.md",
                      new IFilesystem.MoveOptions(false, false, true),
                      IFilesystem.MutationContext.none())));
    } finally {
      fixture.close();
    }
  }

  @Test
  public void cancellationAndCloseSettleCallsOnceAndRejectReuse() {
    Fixture fixture = new Fixture();
    try {
      IFilesystem filesystem = fixture.open("commit", "heads/main");
      IFilesystem.CallContext cancelled = context();
      assertTrue(cancelled.cancel());
      assertFailure("cancelled", () -> join(filesystem.stat(cancelled, "/")));
      join(filesystem.close(context()));
      join(filesystem.close(context()));
      assertFailure("provider-closed", () -> join(filesystem.stat(context(), "/")));
    } finally {
      fixture.close();
    }
  }

  private static final class Fixture implements AutoCloseable {
    final FakeGitHubClient client = new FakeGitHubClient();
    final ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();

    IFilesystem open(String mode, String reference) {
      GitHubFilesystem.Factory factory = new GitHubFilesystem.Factory();
      IFilesystemFactory.OpenContext context =
          new IFilesystemFactory.OpenContext(
              Runnable::run,
              scheduler,
              credential -> {
                assertEquals("github:test", credential);
                return client;
              });
      return join(
          factory.open(
              context,
              Map.of(
                  "credential-ref", "github:test",
                  "repository", "hara-lang/hara",
                  "ref", reference,
                  "root", "/",
                  "mode", mode,
                  "display", "hara-lang/hara@test")));
    }

    @Override
    public void close() {
      scheduler.shutdownNow();
    }
  }

  private static final class FakeGitHubClient implements GitHubObjectClient {
    private final Map<String, byte[]> blobs = new HashMap<>();
    private final Map<String, TreeSnapshot> trees = new HashMap<>();
    private final Map<String, Revision> commits = new HashMap<>();
    private final Map<String, String> references = new HashMap<>();
    private final List<String> commitMessages = new ArrayList<>();
    private long sequence = 1;
    private final String readmeBlob;
    private final String initialCommit;
    private boolean moveBeforeNextUpdate;
    private String competingHead;

    FakeGitHubClient() {
      readmeBlob = blob("hello".getBytes(java.nio.charset.StandardCharsets.UTF_8));
      String sourceBlob = blob("(+ 1 2)".getBytes(java.nio.charset.StandardCharsets.UTF_8));
      String linkBlob = blob("README.md".getBytes(java.nio.charset.StandardCharsets.UTF_8));
      String sourceTree = sha();
      trees.put(
          sourceTree,
          new TreeSnapshot(
              sourceTree,
              List.of(
                  new TreeEntry(
                      "main.hal", "100644", "blob", sourceBlob, 7L)),
              false));
      String rootTree = sha();
      trees.put(
          rootTree,
          new TreeSnapshot(
              rootTree,
              List.of(
                  new TreeEntry("README.md", "100644", "blob", readmeBlob, 5L),
                  new TreeEntry("link", "120000", "blob", linkBlob, 9L),
                  new TreeEntry("src", "040000", "tree", sourceTree, null),
                  new TreeEntry("src/main.hal", "100644", "blob", sourceBlob, 7L),
                  new TreeEntry("vendor", "160000", "commit", sha(), null)),
              false));
      initialCommit = sha();
      commits.put(initialCommit, new Revision(initialCommit, rootTree));
      references.put("heads/main", initialCommit);
    }

    String initialCommit() {
      return initialCommit;
    }

    String readmeBlob() {
      return readmeBlob;
    }

    String head() {
      return references.get("heads/main");
    }

    List<String> commitMessages() {
      return List.copyOf(commitMessages);
    }

    void moveBeforeNextUpdate() {
      moveBeforeNextUpdate = true;
      Revision current = commits.get(head());
      competingHead = sha();
      commits.put(competingHead, new Revision(competingHead, current.treeSha()));
    }

    String competingHead() {
      return competingHead;
    }

    @Override
    public CompletionStage<Revision> resolveRevision(String repository, String reference) {
      String commit = commits.containsKey(reference) ? reference : references.get(reference);
      if (commit == null) return missing("ref-not-found");
      return CompletableFuture.completedFuture(commits.get(commit));
    }

    @Override
    public CompletionStage<TreeSnapshot> readTree(String repository, String treeSha) {
      TreeSnapshot snapshot = trees.get(treeSha);
      return snapshot == null
          ? missing("tree-not-found")
          : CompletableFuture.completedFuture(snapshot);
    }

    @Override
    public CompletionStage<byte[]> readBlob(String repository, String blobSha) {
      byte[] bytes = blobs.get(blobSha);
      return bytes == null
          ? missing("blob-not-found")
          : CompletableFuture.completedFuture(bytes.clone());
    }

    @Override
    public CompletionStage<String> createBlob(String repository, byte[] bytes) {
      return CompletableFuture.completedFuture(blob(bytes));
    }

    @Override
    public CompletionStage<String> createTree(
        String repository, String baseTreeSha, List<TreeChange> changes) {
      TreeSnapshot base = trees.get(baseTreeSha);
      if (base == null) return missing("base-tree-not-found");
      LinkedHashMap<String, TreeEntry> flat = new LinkedHashMap<>();
      for (TreeEntry entry : base.entries()) {
        if (!"tree".equals(entry.type())) flat.put(entry.path(), entry);
      }
      for (TreeChange change : changes) {
        if (change.sha() == null) {
          flat.keySet().removeIf(
              path -> path.equals(change.path()) || path.startsWith(change.path() + "/"));
          continue;
        }
        flat.keySet().removeIf(
            path -> path.equals(change.path()) || path.startsWith(change.path() + "/"));
        if ("tree".equals(change.type())) {
          TreeSnapshot subtree = trees.get(change.sha());
          if (subtree == null) return missing("subtree-not-found");
          for (TreeEntry child : subtree.entries()) {
            if ("tree".equals(child.type())) continue;
            flat.put(
                change.path() + "/" + child.path(),
                new TreeEntry(
                    change.path() + "/" + child.path(),
                    child.mode(),
                    child.type(),
                    child.sha(),
                    child.size()));
          }
        } else {
          Long size = "blob".equals(change.type()) ? (long) blobs.get(change.sha()).length : null;
          flat.put(
              change.path(),
              new TreeEntry(
                  change.path(), change.mode(), change.type(), change.sha(), size));
        }
      }
      String root = rebuildTrees(flat);
      return CompletableFuture.completedFuture(root);
    }

    @Override
    public CompletionStage<String> createCommit(
        String repository, String message, String treeSha, String parentCommitSha) {
      if (!trees.containsKey(treeSha) || !commits.containsKey(parentCommitSha)) {
        return missing("commit-input-not-found");
      }
      String commit = sha();
      commits.put(commit, new Revision(commit, treeSha));
      commitMessages.add(message);
      return CompletableFuture.completedFuture(commit);
    }

    @Override
    public CompletionStage<Void> updateReference(
        String repository,
        String reference,
        String expectedCommitSha,
        String newCommitSha) {
      if (moveBeforeNextUpdate) {
        moveBeforeNextUpdate = false;
        references.put(reference, competingHead);
      }
      if (!expectedCommitSha.equals(references.get(reference))) {
        return CompletableFuture.failedFuture(
            new Failure(
                FailureKind.CONFLICT,
                "reference moved",
                "reference-update-conflict",
                true));
      }
      references.put(reference, newCommitSha);
      return CompletableFuture.completedFuture(null);
    }

    private String rebuildTrees(Map<String, TreeEntry> files) {
      Set<String> directories = new java.util.TreeSet<>();
      for (String path : files.keySet()) {
        int separator = path.lastIndexOf('/');
        while (separator > 0) {
          directories.add(path.substring(0, separator));
          separator = path.lastIndexOf('/', separator - 1);
        }
      }
      Map<String, String> directoryShas = new HashMap<>();
      ArrayList<String> deepest = new ArrayList<>(directories);
      deepest.sort(
          Comparator.comparingInt((String value) -> value.split("/").length).reversed());
      for (String directory : deepest) {
        String treeSha = sha();
        directoryShas.put(directory, treeSha);
        ArrayList<TreeEntry> subtree = new ArrayList<>();
        String prefix = directory + "/";
        for (TreeEntry entry : files.values()) {
          if (!entry.path().startsWith(prefix)) continue;
          String relative = entry.path().substring(prefix.length());
          subtree.add(
              new TreeEntry(relative, entry.mode(), entry.type(), entry.sha(), entry.size()));
        }
        trees.put(treeSha, new TreeSnapshot(treeSha, subtree, false));
      }
      ArrayList<TreeEntry> rootEntries = new ArrayList<>(files.values());
      for (Map.Entry<String, String> directory : directoryShas.entrySet()) {
        rootEntries.add(
            new TreeEntry(
                directory.getKey(), "040000", "tree", directory.getValue(), null));
      }
      rootEntries.sort(Comparator.comparing(TreeEntry::path));
      String rootSha = sha();
      trees.put(rootSha, new TreeSnapshot(rootSha, rootEntries, false));
      return rootSha;
    }

    private String blob(byte[] bytes) {
      String sha = sha();
      blobs.put(sha, bytes.clone());
      return sha;
    }

    private String sha() {
      return String.format("%040x", sequence++);
    }

    private static <T> CompletionStage<T> missing(String providerCode) {
      return CompletableFuture.failedFuture(
          new Failure(FailureKind.NOT_FOUND, "GitHub object not found", providerCode, false));
    }
  }

  private static IFilesystem.CallContext context() {
    return IFilesystem.CallContext.create();
  }

  private static List<String> paths(List<IFilesystem.Entry> entries) {
    return entries.stream().map(IFilesystem.Entry::path).toList();
  }

  private static void assertFailure(String code, Runnable operation) {
    FilesystemException error = assertThrows(FilesystemException.class, operation::run);
    assertEquals(code, error.code());
    assertEquals("github", error.provider());
  }

  private static <T> T join(CompletionStage<T> stage) {
    return stage.toCompletableFuture().join();
  }
}
