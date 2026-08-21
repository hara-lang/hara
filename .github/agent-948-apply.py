from pathlib import Path
import re

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

cell_import = "use std::cell::RefCell;\n"
if text.count(cell_import) < 1:
    raise SystemExit("unable to locate the raw Session RefCell import")
text = text.replace(cell_import, "use std::cell::{Cell, RefCell};\n", 1)

field_anchor = '''    next_call: u64,
    events: Rc<RefCell<VecDeque<Vec<u8>>>>,
'''
field_replacement = '''    next_call: Rc<Cell<u64>>,
    pending_calls:
        Rc<RefCell<Vec<(u64, u64, Promise, String, String, Vec<Value>)>>>,
    events: Rc<RefCell<VecDeque<Vec<u8>>>>,
'''
if text.count(field_anchor) != 1:
    raise SystemExit(f"expected one Session host-call field anchor, found {text.count(field_anchor)}")
text = text.replace(field_anchor, field_replacement, 1)

init_anchor = '''            next_call: 1,
            events,
'''
init_replacement = '''            next_call: Rc::new(Cell::new(1)),
            pending_calls: Rc::new(RefCell::new(Vec::new())),
            events,
'''
if text.count(init_anchor) != 1:
    raise SystemExit(f"expected one Session host-call initializer, found {text.count(init_anchor)}")
text = text.replace(init_anchor, init_replacement, 1)

host_start = text.find("    fn host_handler(")
host_end = text.find("    fn start_fiber(", host_start)
if host_start < 0 or host_end < 0:
    raise SystemExit("unable to locate the raw host handler and collector block")
host_block = '''    fn host_handler(
        &self,
        task: u64,
    ) -> Rc<dyn Fn(String, String, Vec<Value>) -> Result<Value, String>> {
        let queue = self.pending_calls.clone();
        let ids = self.next_call.clone();
        Rc::new(move |service: String, method: String, args: Vec<Value>| {
            let call = ids.get();
            ids.set(call.saturating_add(1));
            let promise = Promise::new();
            queue.borrow_mut().push((
                task,
                call,
                promise.clone(),
                service,
                method,
                args,
            ));
            Ok(Value::Promise(promise))
        })
    }

    fn collect_calls(&mut self) {
        let pending = self
            .pending_calls
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();
        for (task, call, promise, service, method, args) in pending {
            let value = Value::Vector(
                vec![
                    Value::Number(2),
                    Value::Number(call as i64),
                    Value::Number(task as i64),
                    Value::String(self.name.clone()),
                    self.mount_id
                        .map(|mount| Value::Number(mount as i64))
                        .unwrap_or(Value::Nil),
                    Value::String(service),
                    Value::String(method),
                    Value::Vector(args.into()),
                ]
                .into(),
            );
            match hta::encode(&value) {
                Ok(bytes) => {
                    self.calls.insert(call, (task, promise));
                    self.events.borrow_mut().push_back(bytes);
                }
                Err(error) => {
                    promise.reject(format!("hta/value-unsupported: {error}"));
                }
            }
        }
    }
'''
text = text[:host_start] + host_block + text[host_end:]

text, handler_sites = re.subn(
    r"let \(handler, pending, next\) = self\.host_handler\(task\);",
    "let handler = self.host_handler(task);",
    text,
)
text, collector_sites = re.subn(
    r"self\.collect_calls\(task, pending, next\);",
    "self.collect_calls();",
    text,
)
if handler_sites < 3 or collector_sites != handler_sites:
    raise SystemExit(
        f"unexpected raw host handler sites: handlers={handler_sites}, collectors={collector_sites}"
    )

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
        // Host services may be invoked by the poll above, after the evaluator's
        // original synchronous collection window. Publish those deferred calls
        // through the same canonical HTA event and Session-owned call table.
        self.collect_calls();
        loop {
'''
if text.count(old_drain) != 1:
    raise SystemExit(f"expected one Session drain loop, found {text.count(old_drain)}")
text = text.replace(old_drain, new_drain, 1)

if "HostFileProvider" in text:
    raise SystemExit("legacy HostFileProvider remains after patch")
path.write_text(text)
print(
    "Applied raw-Wasm direct IFilesystem host dispatch patch "
    f"across {handler_sites} evaluator boundaries"
)
