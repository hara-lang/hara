use crate::core::{ExceptionInfo, Promise, PromiseRejection, PromiseState, Value};
use crate::file::{
    CopyOptions, DeleteOptions, FileError, FileProvider, FileType, MkdirOptions, MoveOptions,
    WriteMode, WriteOptions,
};
use crate::filesystem::{
    FilesystemCallContext, FilesystemCapabilities, FilesystemCapability, FilesystemDescriptor,
    FilesystemEntry, FilesystemEntryPage, FilesystemFuture, FilesystemHandle, FilesystemMutation,
    FilesystemMutationContext, FilesystemPageRequest, IFilesystem,
};
use crate::filesystem_runtime::FilesystemRuntimeAdapter;
use std::collections::BTreeMap;
use std::future::{ready, Future};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub(crate) type HostHandler =
    Rc<dyn Fn(String, String, Vec<Value>) -> Result<Value, String>>;

pub(crate) fn provider(handler: HostHandler) -> Rc<dyn FileProvider> {
    let handle = FilesystemHandle::new(HostFilesystem::new(handler));
    Rc::new(FilesystemRuntimeAdapter::new(handle))
}

#[derive(Clone)]
pub(crate) struct HostFilesystem {
    handler: HostHandler,
}

impl HostFilesystem {
    pub(crate) fn new(handler: HostHandler) -> Self {
        Self { handler }
    }

    fn invoke<'a, T: 'a>(
        &'a self,
        context: FilesystemCallContext,
        method: &'static str,
        arguments: Vec<Value>,
        decode: fn(Value) -> Result<T, FileError>,
    ) -> FilesystemFuture<'a, T> {
        if let Err(error) = context.check() {
            return Box::pin(ready(Err(error)));
        }
        let result = (self.handler)("file".into(), method.into(), arguments);
        let promise = match result {
            Ok(Value::Promise(promise)) => promise,
            Ok(value) => {
                return Box::pin(ready(Err(FileError::Io(format!(
                    "file/{method} host call returned {} rather than a promise",
                    value.display()
                )))))
            }
            Err(error) => return Box::pin(ready(Err(FileError::Io(error)))),
        };
        Box::pin(HostPromiseFuture::new(promise, context, decode))
    }
}

impl IFilesystem for HostFilesystem {
    fn descriptor(&self) -> FilesystemDescriptor {
        FilesystemDescriptor::new(
            "host",
            "Mounted host filesystem",
            false,
            FilesystemCapabilities::new([
                FilesystemCapability::Read,
                FilesystemCapability::Write,
                FilesystemCapability::Entries,
                FilesystemCapability::Mkdir,
                FilesystemCapability::Delete,
                FilesystemCapability::Copy,
                FilesystemCapability::Move,
                FilesystemCapability::Append,
            ]),
        )
    }

    fn stat<'a>(
        &'a self,
        context: FilesystemCallContext,
        path: String,
    ) -> FilesystemFuture<'a, FilesystemEntry> {
        self.invoke(context, "stat", vec![Value::String(path)], decode_entry)
    }

    fn read<'a>(
        &'a self,
        context: FilesystemCallContext,
        path: String,
    ) -> FilesystemFuture<'a, Vec<u8>> {
        self.invoke(context, "read", vec![Value::String(path)], decode_bytes)
    }

    fn write<'a>(
        &'a self,
        context: FilesystemCallContext,
        path: String,
        bytes: Vec<u8>,
        options: WriteOptions,
        mutation: FilesystemMutationContext,
    ) -> FilesystemFuture<'a, FilesystemMutation> {
        let values = mutation_options(
            [
                (
                    "mode",
                    Value::Keyword(
                        match options.mode {
                            WriteMode::Create => "create",
                            WriteMode::Replace => "replace",
                            WriteMode::Append => "append",
                        }
                        .into(),
                    ),
                ),
                ("parents?", Value::Bool(options.parents)),
            ],
            mutation,
        );
        self.invoke(
            context,
            "write",
            vec![Value::String(path), Value::Bytes(bytes), option_map(values)],
            decode_mutation,
        )
    }

    fn entries_page<'a>(
        &'a self,
        context: FilesystemCallContext,
        path: String,
        request: FilesystemPageRequest,
    ) -> FilesystemFuture<'a, FilesystemEntryPage> {
        if request.token.is_some() {
            return Box::pin(ready(Ok(FilesystemEntryPage {
                entries: Vec::new(),
                next_token: None,
            })));
        }
        self.invoke(
            context,
            "entries",
            vec![Value::String(path)],
            decode_entry_page,
        )
    }

    fn mkdir<'a>(
        &'a self,
        context: FilesystemCallContext,
        path: String,
        options: MkdirOptions,
        mutation: FilesystemMutationContext,
    ) -> FilesystemFuture<'a, FilesystemMutation> {
        let values = mutation_options(
            [
                ("parents?", Value::Bool(options.parents)),
                ("exists-ok?", Value::Bool(options.exists_ok)),
            ],
            mutation,
        );
        self.invoke(
            context,
            "mkdir",
            vec![Value::String(path), option_map(values)],
            decode_mutation,
        )
    }

    fn delete<'a>(
        &'a self,
        context: FilesystemCallContext,
        path: String,
        options: DeleteOptions,
        mutation: FilesystemMutationContext,
    ) -> FilesystemFuture<'a, FilesystemMutation> {
        let values = mutation_options(
            [("missing-ok?", Value::Bool(options.missing_ok))],
            mutation,
        );
        self.invoke(
            context,
            "delete",
            vec![Value::String(path), option_map(values)],
            decode_mutation,
        )
    }

    fn copy<'a>(
        &'a self,
        context: FilesystemCallContext,
        source: String,
        target: String,
        options: CopyOptions,
        mutation: FilesystemMutationContext,
    ) -> FilesystemFuture<'a, FilesystemMutation> {
        let values = mutation_options(
            [
                ("replace?", Value::Bool(options.replace)),
                ("parents?", Value::Bool(options.parents)),
                (
                    "preserve-modified?",
                    Value::Bool(options.preserve_modified),
                ),
            ],
            mutation,
        );
        self.invoke(
            context,
            "copy",
            vec![
                Value::String(source),
                Value::String(target),
                option_map(values),
            ],
            decode_mutation,
        )
    }

    fn move_entry<'a>(
        &'a self,
        context: FilesystemCallContext,
        source: String,
        target: String,
        options: MoveOptions,
        mutation: FilesystemMutationContext,
    ) -> FilesystemFuture<'a, FilesystemMutation> {
        let values = mutation_options(
            [
                ("replace?", Value::Bool(options.replace)),
                ("parents?", Value::Bool(options.parents)),
                ("atomic?", Value::Bool(options.atomic)),
            ],
            mutation,
        );
        self.invoke(
            context,
            "move",
            vec![
                Value::String(source),
                Value::String(target),
                option_map(values),
            ],
            decode_mutation,
        )
    }

    fn close<'a>(
        &'a self,
        context: FilesystemCallContext,
    ) -> FilesystemFuture<'a, ()> {
        Box::pin(ready(context.check().map(|()| ())))
    }
}

struct HostPromiseFuture<T> {
    promise: Option<Promise>,
    context: FilesystemCallContext,
    decode: fn(Value) -> Result<T, FileError>,
}

impl<T> HostPromiseFuture<T> {
    fn new(
        promise: Promise,
        context: FilesystemCallContext,
        decode: fn(Value) -> Result<T, FileError>,
    ) -> Self {
        Self {
            promise: Some(promise),
            context,
            decode,
        }
    }
}

impl<T> Future for HostPromiseFuture<T> {
    type Output = Result<T, FileError>;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        if let Err(error) = self.context.check() {
            if let Some(promise) = self.promise.take() {
                promise.cancel();
            }
            return Poll::Ready(Err(error));
        }
        let state = self
            .promise
            .as_ref()
            .map(Promise::state)
            .unwrap_or_else(|| PromiseState::Rejected("host promise was already consumed".into()));
        match state {
            PromiseState::Pending => Poll::Pending,
            PromiseState::Fulfilled(value) => {
                self.promise.take();
                Poll::Ready((self.decode)(value))
            }
            PromiseState::Rejected(rejection) => {
                self.promise.take();
                Poll::Ready(Err(rejection_error(rejection)))
            }
        }
    }
}

impl<T> Drop for HostPromiseFuture<T> {
    fn drop(&mut self) {
        if let Some(promise) = self.promise.take() {
            promise.cancel();
        }
    }
}

fn option_map(values: Vec<(&'static str, Value)>) -> Value {
    Value::Map(
        values
            .into_iter()
            .map(|(key, value)| (Value::Keyword(key.into()), value))
            .collect(),
    )
}

fn mutation_options<const N: usize>(
    values: [(&'static str, Value); N],
    mutation: FilesystemMutationContext,
) -> Vec<(&'static str, Value)> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    if let Some(revision) = mutation.expected_revision {
        values.push(("expected-revision", Value::String(revision)));
    }
    if let Some(revision) = mutation.expected_target_revision {
        values.push(("expected-target-revision", Value::String(revision)));
    }
    values
}

fn decode_bytes(value: Value) -> Result<Vec<u8>, FileError> {
    match value {
        Value::Bytes(bytes) => Ok(bytes),
        Value::ByteBuffer(bytes) => Ok(bytes.borrow().clone()),
        value => Err(FileError::Io(format!(
            "file/read host response must be bytes, got {}",
            value.display()
        ))),
    }
}

fn decode_entry_page(value: Value) -> Result<FilesystemEntryPage, FileError> {
    let Value::Vector(values) = value else {
        return Err(FileError::Io(
            "file/entries host response must be a vector".into(),
        ));
    };
    let entries = values
        .iter()
        .cloned()
        .map(decode_entry)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FilesystemEntryPage {
        entries,
        next_token: None,
    })
}

fn decode_entry(value: Value) -> Result<FilesystemEntry, FileError> {
    let path = required_string(&value, "path", "file entry")?;
    let name = required_string(&value, "name", "file entry")?;
    let kind = match required_keyword(&value, "type", "file entry")?.as_str() {
        "file" => FileType::File,
        "directory" => FileType::Directory,
        "symlink" => FileType::Symlink,
        _ => FileType::Other,
    };
    let size = optional_number(&value, "size")?.and_then(|size| u64::try_from(size).ok());
    let modified_at = optional_number(&value, "modified-at")?;
    let extensions = field(&value, "extensions");
    let id = extensions.and_then(|value| optional_string(value, "file/id").ok().flatten());
    let revision = extensions
        .and_then(|value| optional_string(value, "file/revision").ok().flatten());
    Ok(FilesystemEntry {
        path,
        name,
        kind,
        size,
        modified_at,
        id,
        revision,
        capabilities: None,
        extensions: string_extensions(extensions),
    })
}

fn decode_mutation(value: Value) -> Result<FilesystemMutation, FileError> {
    match value {
        Value::String(path) => Ok(FilesystemMutation::path(path)),
        value @ Value::Map(_) => {
            let path = required_string(&value, "path", "filesystem mutation")?;
            let revision = optional_string(&value, "revision")?;
            let mount_revision = optional_string(&value, "mount-revision")?;
            let extensions = field(&value, "extensions");
            Ok(FilesystemMutation {
                path,
                revision,
                mount_revision,
                extensions: string_extensions(extensions),
            })
        }
        value => Err(FileError::Io(format!(
            "filesystem mutation response must be a path or map, got {}",
            value.display()
        ))),
    }
}

fn field<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let Value::Map(values) = value else {
        return None;
    };
    values.get(&Value::Keyword(key.into()))
}

fn required_string(value: &Value, key: &str, context: &str) -> Result<String, FileError> {
    match field(value, key) {
        Some(Value::String(value)) => Ok(value.clone()),
        _ => Err(FileError::Io(format!(
            "{context} :{key} must be a string"
        ))),
    }
}

fn optional_string(value: &Value, key: &str) -> Result<Option<String>, FileError> {
    match field(value, key) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(FileError::Io(format!(
            "filesystem field :{key} must be a string"
        ))),
    }
}

fn required_keyword(value: &Value, key: &str, context: &str) -> Result<String, FileError> {
    match field(value, key) {
        Some(Value::Keyword(value)) => Ok(value.as_str().into()),
        _ => Err(FileError::Io(format!(
            "{context} :{key} must be a keyword"
        ))),
    }
}

fn optional_number(value: &Value, key: &str) -> Result<Option<i64>, FileError> {
    match field(value, key) {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::Number(value)) => Ok(Some(*value)),
        _ => Err(FileError::Io(format!(
            "filesystem field :{key} must be an integer"
        ))),
    }
}

fn string_extensions(value: Option<&Value>) -> BTreeMap<String, String> {
    let Some(Value::Map(values)) = value else {
        return BTreeMap::new();
    };
    values
        .iter()
        .filter_map(|(key, value)| match (key, value) {
            (Value::Keyword(key), Value::String(value)) => {
                Some((key.as_str().to_owned(), value.clone()))
            }
            _ => None,
        })
        .collect()
}

fn rejection_error(rejection: PromiseRejection) -> FileError {
    let message = rejection.message();
    match rejection.value() {
        Value::ExceptionInfo(exception) => exception_error(exception.as_ref()),
        Value::Map(data) => map_error(&Value::Map(data), &message),
        Value::String(value) => FileError::Io(value),
        _ if rejection.is_cancelled() => FileError::Io("filesystem operation cancelled".into()),
        _ => FileError::Io(message),
    }
}

fn exception_error(exception: &ExceptionInfo) -> FileError {
    map_error(exception.data.as_ref(), &exception.message)
}

fn map_error(value: &Value, message: &str) -> FileError {
    let code = field(value, "ex/code")
        .or_else(|| field(value, "code"))
        .and_then(|value| match value {
            Value::Keyword(value) => Some(value.as_str()),
            Value::String(value) => Some(value.as_str()),
            _ => None,
        })
        .unwrap_or("file/io")
        .trim_start_matches("file/");
    match code {
        "not-found" => FileError::NotFound,
        "already-exists" => FileError::AlreadyExists,
        "invalid-path" => FileError::InvalidPath(message.into()),
        "outside-root" => FileError::OutsideRoot,
        "denied" => FileError::Denied,
        "not-directory" => FileError::NotDirectory,
        "is-directory" => FileError::IsDirectory,
        "directory-not-empty" => FileError::DirectoryNotEmpty,
        "permission-denied" => FileError::PermissionDenied,
        "unsupported" => FileError::Unsupported,
        _ => FileError::Io(message.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    fn poll_ready<T>(mut future: FilesystemFuture<'_, T>) -> Result<T, FileError> {
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("test host future remained pending"),
        }
    }

    fn noop_waker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            raw_waker()
        }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}
        fn raw_waker() -> RawWaker {
            RawWaker::new(
                std::ptr::null(),
                &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
            )
        }
        unsafe { Waker::from_raw(raw_waker()) }
    }

    #[test]
    fn forwards_write_options_and_revision_guards() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let recorded = calls.clone();
        let filesystem = HostFilesystem::new(Rc::new(move |service, method, arguments| {
            recorded.borrow_mut().push((service, method, arguments));
            let promise = Promise::new();
            promise.resolve(Value::String("/out.bin".into()));
            Ok(Value::Promise(promise))
        }));

        let mutation = poll_ready(filesystem.write(
            FilesystemCallContext::default(),
            "/out.bin".into(),
            vec![1, 2, 3],
            WriteOptions {
                mode: WriteMode::Append,
                parents: true,
            },
            FilesystemMutationContext {
                expected_revision: Some("source-r1".into()),
                expected_target_revision: Some("target-r1".into()),
            },
        ))
        .unwrap();
        assert_eq!(mutation.path, "/out.bin");

        let calls = calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "file");
        assert_eq!(calls[0].1, "write");
        let Value::Map(options) = &calls[0].2[2] else {
            panic!("write options were not a map");
        };
        assert_eq!(
            options.get(&Value::Keyword("mode".into())),
            Some(&Value::Keyword("append".into()))
        );
        assert_eq!(
            options.get(&Value::Keyword("parents?".into())),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            options.get(&Value::Keyword("expected-revision".into())),
            Some(&Value::String("source-r1".into()))
        );
        assert_eq!(
            options.get(&Value::Keyword("expected-target-revision".into())),
            Some(&Value::String("target-r1".into()))
        );
    }

    #[test]
    fn forwards_atomic_and_preserve_modified_requests() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let recorded = calls.clone();
        let filesystem = HostFilesystem::new(Rc::new(move |_service, method, arguments| {
            recorded.borrow_mut().push((method, arguments));
            let promise = Promise::new();
            promise.resolve(Value::String("/target".into()));
            Ok(Value::Promise(promise))
        }));

        poll_ready(filesystem.copy(
            FilesystemCallContext::default(),
            "/source".into(),
            "/target".into(),
            CopyOptions {
                replace: true,
                parents: true,
                preserve_modified: true,
            },
            FilesystemMutationContext::default(),
        ))
        .unwrap();
        poll_ready(filesystem.move_entry(
            FilesystemCallContext::default(),
            "/source".into(),
            "/target".into(),
            MoveOptions {
                replace: true,
                parents: true,
                atomic: true,
            },
            FilesystemMutationContext::default(),
        ))
        .unwrap();

        let calls = calls.borrow();
        let Value::Map(copy) = &calls[0].1[2] else {
            panic!("copy options were not a map");
        };
        assert_eq!(
            copy.get(&Value::Keyword("preserve-modified?".into())),
            Some(&Value::Bool(true))
        );
        let Value::Map(moved) = &calls[1].1[2] else {
            panic!("move options were not a map");
        };
        assert_eq!(
            moved.get(&Value::Keyword("atomic?".into())),
            Some(&Value::Bool(true))
        );
    }

    #[test]
    fn maps_structured_host_rejections() {
        let filesystem = HostFilesystem::new(Rc::new(move |_service, _method, _arguments| {
            let promise = Promise::new();
            promise.reject_value(crate::file::file_error_value(
                "file/read",
                "/missing",
                None,
                &FileError::NotFound,
            ));
            Ok(Value::Promise(promise))
        }));

        assert_eq!(
            poll_ready(filesystem.read(
                FilesystemCallContext::default(),
                "/missing".into()
            )),
            Err(FileError::NotFound)
        );
    }

    #[test]
    fn dropping_a_pending_future_cancels_the_host_promise_once() {
        let pending = Rc::new(RefCell::new(None));
        let captured = pending.clone();
        let filesystem = HostFilesystem::new(Rc::new(move |_service, _method, _arguments| {
            let promise = Promise::new();
            *captured.borrow_mut() = Some(promise.clone());
            Ok(Value::Promise(promise))
        }));

        let future = filesystem.read(FilesystemCallContext::default(), "/pending".into());
        drop(future);
        let promise = pending.borrow().clone().expect("captured host promise");
        assert!(matches!(
            promise.state(),
            PromiseState::Rejected(rejection) if rejection.is_cancelled()
        ));
        assert!(!promise.cancel());
    }
}
