package hara.truffle;

import java.io.IOException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.atomic.AtomicBoolean;

/** Host-owned loader for one resolver-selected, prebuilt JVM package artifact. */
final class JvmPackageLoader {
  record Selection(
      String identity,
      Path artifact,
      String sha256,
      String abi,
      String entryPoint,
      Set<String> allowedCapabilities) {
    Selection {
      identity = requireText(identity, "package identity");
      artifact = Objects.requireNonNull(artifact, "package artifact").toAbsolutePath().normalize();
      sha256 = requireDigest(sha256);
      abi = requireText(abi, "package ABI");
      entryPoint = requireText(entryPoint, "package entry point");
      allowedCapabilities = Set.copyOf(Objects.requireNonNull(allowedCapabilities, "capabilities"));
    }
  }

  static final class LoadedProvider implements AutoCloseable {
    private final JvmPackageProvider provider;
    private final URLClassLoader classLoader;
    private final FilesystemProviderRegistry filesystems;
    private final List<IFilesystemFactory> factories;
    private final AtomicBoolean closed = new AtomicBoolean();

    LoadedProvider(
        JvmPackageProvider provider,
        URLClassLoader classLoader,
        FilesystemProviderRegistry filesystems,
        List<IFilesystemFactory> factories) {
      this.provider = provider;
      this.classLoader = classLoader;
      this.filesystems = filesystems;
      this.factories = List.copyOf(factories);
    }

    String identity() {
      return provider.identity();
    }

    @Override
    public void close() throws Exception {
      if (!closed.compareAndSet(false, true)) return;
      Exception failure = null;
      for (int index = factories.size() - 1; index >= 0; index--) {
        try {
          filesystems.unregister(factories.get(index));
        } catch (RuntimeException exception) {
          if (failure == null) failure = exception;
          else failure.addSuppressed(exception);
        }
      }
      try {
        provider.close();
      } catch (Exception exception) {
        failure = exception;
      }
      try {
        classLoader.close();
      } catch (IOException exception) {
        if (failure == null) failure = exception;
        else failure.addSuppressed(exception);
      }
      if (failure != null) throw failure;
    }
  }

  private JvmPackageLoader() {}

  static LoadedProvider load(Selection selection, FilesystemProviderRegistry filesystems) {
    Objects.requireNonNull(selection, "package selection");
    Objects.requireNonNull(filesystems, "filesystem registry");
    verifyArtifact(selection);
    URLClassLoader loader = null;
    JvmPackageProvider provider = null;
    List<IFilesystemFactory> factories = new ArrayList<>();
    try {
      loader =
          new URLClassLoader(
              new URL[] {selection.artifact().toUri().toURL()}, JvmPackageProvider.class.getClassLoader());
      Class<?> entry = Class.forName(selection.entryPoint(), true, loader);
      if (!JvmPackageProvider.class.isAssignableFrom(entry)) {
        throw failure("PACKAGE_JVM_ENTRYPOINT_INVALID", selection.entryPoint());
      }
      provider = (JvmPackageProvider) entry.getDeclaredConstructor().newInstance();
      if (!selection.identity().equals(provider.identity())) {
        throw failure("PACKAGE_JVM_IDENTITY_MISMATCH", provider.identity());
      }
      if (!selection.abi().equals(provider.abi()) || !JvmPackageProvider.ABI.equals(provider.abi())) {
        throw failure("PACKAGE_JVM_ABI_MISMATCH", provider.abi());
      }
      TreeSet<String> requested = new TreeSet<>(provider.capabilities());
      if (!selection.allowedCapabilities().containsAll(requested)) {
        requested.removeAll(selection.allowedCapabilities());
        throw failure("PACKAGE_JVM_CAPABILITY_DENIED", String.join(",", requested));
      }
      JvmPackageProvider finalProvider = provider;
      provider.register(
          factory -> {
            filesystems.register(factory);
            factories.add(factory);
          });
      return new LoadedProvider(finalProvider, loader, filesystems, factories);
    } catch (Throwable error) {
      unregisterAll(filesystems, factories, error);
      closeFailed(provider, loader, error);
      if (error instanceof IllegalArgumentException exception) throw exception;
      throw failure("PACKAGE_JVM_INITIALIZATION_FAILED", message(error), error);
    }
  }

  private static void verifyArtifact(Selection selection) {
    if (!Files.isRegularFile(selection.artifact())) {
      throw failure("PACKAGE_JVM_ARTIFACT_MISSING", selection.artifact().toString());
    }
    try {
      String actual = "sha256:" + HexFormat.of().formatHex(digest().digest(Files.readAllBytes(selection.artifact())));
      if (!selection.sha256().equals(actual)) {
        throw failure("PACKAGE_JVM_DIGEST_MISMATCH", actual);
      }
    } catch (IOException exception) {
      throw failure("PACKAGE_JVM_ARTIFACT_UNREADABLE", selection.artifact().toString(), exception);
    }
  }

  private static MessageDigest digest() {
    try {
      return MessageDigest.getInstance("SHA-256");
    } catch (NoSuchAlgorithmException impossible) {
      throw new IllegalStateException("SHA-256 unavailable", impossible);
    }
  }

  private static void closeFailed(
      JvmPackageProvider provider, URLClassLoader loader, Throwable original) {
    if (provider != null) {
      try {
        provider.close();
      } catch (Throwable close) {
        original.addSuppressed(close);
      }
    }
    if (loader != null) {
      try {
        loader.close();
      } catch (Throwable close) {
        original.addSuppressed(close);
      }
    }
  }

  private static void unregisterAll(
      FilesystemProviderRegistry filesystems,
      List<IFilesystemFactory> factories,
      Throwable original) {
    for (int index = factories.size() - 1; index >= 0; index--) {
      try {
        filesystems.unregister(factories.get(index));
      } catch (Throwable rollback) {
        original.addSuppressed(rollback);
      }
    }
  }

  private static String requireText(String value, String label) {
    if (value == null || value.isBlank()) throw failure("PACKAGE_JVM_SELECTION_INVALID", label);
    return value;
  }

  private static String requireDigest(String value) {
    if (value == null || !value.matches("sha256:[0-9a-f]{64}")) {
      throw failure("PACKAGE_JVM_SELECTION_INVALID", "artifact digest");
    }
    return value;
  }

  private static String message(Throwable error) {
    return error.getMessage() == null ? error.getClass().getName() : error.getMessage();
  }

  private static IllegalArgumentException failure(String code, String detail) {
    return new IllegalArgumentException(code + " " + detail);
  }

  private static IllegalArgumentException failure(String code, String detail, Throwable cause) {
    return new IllegalArgumentException(code + " " + detail, cause);
  }
}
