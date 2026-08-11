package hara.truffle;

import hara.truffle.bytecode.HbcBundleCodec;
import hara.truffle.bytecode.HbcCodec;
import hara.truffle.bytecode.HbcFormatException;
import hara.truffle.bytecode.HbcProgram;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** Immutable namespace index over the shared Rust-produced HBB2 standard-library bundle. */
final class HbcBundleLibrary {
  static final String RESOURCE = "std.foundation.hbb";

  record Module(String namespace, HbcBundleCodec.Module descriptor, HbcProgram program) {}

  private final Map<String, Module> modules;

  HbcBundleLibrary(ClassLoader loader) {
    this.modules = load(loader);
  }

  boolean available() {
    return !modules.isEmpty();
  }

  boolean provides(String namespace) {
    return modules.containsKey(namespace);
  }

  Module module(String namespace) {
    return modules.get(namespace);
  }

  List<Module> eagerModules() {
    ArrayList<Module> eager = new ArrayList<>();
    for (Module module : modules.values()) {
      if (module.descriptor().eager()) eager.add(module);
    }
    return List.copyOf(eager);
  }

  private static Map<String, Module> load(ClassLoader loader) {
    try (InputStream input = loader.getResourceAsStream(RESOURCE)) {
      if (input == null) return Map.of();
      LinkedHashMap<String, Module> indexed = new LinkedHashMap<>();
      for (HbcBundleCodec.Module descriptor : HbcBundleCodec.decode(input.readAllBytes())) {
        HbcProgram program = HbcCodec.decode(descriptor.artifact());
        String namespace =
            program.namespace() == null ? descriptor.resource() : program.namespace();
        if (namespace == null || namespace.isBlank()) {
          throw new HbcFormatException("HBB2 module has no namespace: " + descriptor.resource());
        }
        if (indexed.put(namespace, new Module(namespace, descriptor, program)) != null) {
          throw new HbcFormatException("HBB2 contains duplicate namespace: " + namespace);
        }
      }
      return Map.copyOf(indexed);
    } catch (IOException error) {
      throw new HaraException("Unable to read " + RESOURCE + ": " + error.getMessage());
    }
  }
}
