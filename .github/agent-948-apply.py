from pathlib import Path

path = Path("core/rust/raw/src/lib.rs")
text = path.read_text()

module_anchor = '''#[path = "../../src/file.rs"]
pub mod file;
'''
module_replacement = '''#[path = "../../src/file.rs"]
pub mod file;
#[path = "../../src/file/interface.rs"]
pub mod filesystem;
#[path = "../../src/runtime/filesystem_bridge.rs"]
mod filesystem_bridge;
#[path = "../../src/runtime/filesystem_adapter.rs"]
mod filesystem_runtime;
mod host_filesystem;
'''
if text.count(module_anchor) != 1:
    raise SystemExit(f"expected one raw module anchor, found {text.count(module_anchor)}")
text = text.replace(module_anchor, module_replacement, 1)

old_provider = '''        let file_provider = self.mount_id.map(|_| {
            Rc::new(HostFileProvider {
                handler: handler.clone(),
            }) as Rc<dyn core::FileProvider>
        });
'''
new_provider = '''        let file_provider = self
            .mount_id
            .map(|_| host_filesystem::provider(handler.clone()));
'''
if text.count(old_provider) != 3:
    raise SystemExit(f"expected three HostFileProvider construction sites, found {text.count(old_provider)}")
text = text.replace(old_provider, new_provider)

start = text.find("struct HostFileProvider {")
end = text.find("struct FilesystemMount {", start)
if start < 0 or end < 0:
    raise SystemExit("unable to locate the legacy HostFileProvider block")
text = text[:start] + text[end:]

old_drain = '''    fn drain_ready(&mut self) {
        loop {
'''
new_drain = '''    fn drain_ready(&mut self) {
        // Provider-neutral filesystem futures wrap host Promises. Poll each
        // suspended outer Promise before consuming the ready queue so a host
        // settlement can drive the shared FilesystemRuntimeAdapter exactly
        // once without retaining a private FileProvider settlement path.
        for fiber in self.fibers.values() {
            if let Some(promise) = fiber.pending() {
                promise.state();
            }
        }
        for promise in self.tasks.values() {
            promise.state();
        }
        loop {
'''
if text.count(old_drain) != 1:
    raise SystemExit(f"expected one Session drain loop, found {text.count(old_drain)}")
text = text.replace(old_drain, new_drain, 1)

if "HostFileProvider" in text:
    raise SystemExit("legacy HostFileProvider remains after patch")
path.write_text(text)
print("Applied raw-Wasm direct IFilesystem host dispatch patch")
