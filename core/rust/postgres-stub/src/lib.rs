use hara_abi::{Error, NativeIdentity, NativeModule, TaskEvent, TaskId, Value};
use std::sync::Arc;

struct UnavailablePostgres {
    identity: NativeIdentity,
}

impl NativeModule for UnavailablePostgres {
    fn identity(&self) -> &NativeIdentity {
        &self.identity
    }

    fn operations(&self) -> &[&str] {
        &[]
    }

    fn capabilities(&self) -> &[&str] {
        &[]
    }

    fn start(&self, _operation: &str, _arguments: Vec<Value>) -> Result<TaskId, Error> {
        Err(Error::new(
            "postgres/ci-stub",
            "PostgreSQL is disabled in the production HBX validation probe",
        ))
    }

    fn poll(&self, _task: TaskId) -> Result<TaskEvent, Error> {
        Err(Error::new(
            "postgres/ci-stub",
            "PostgreSQL is disabled in the production HBX validation probe",
        ))
    }

    fn cancel(&self, _task: TaskId) -> Result<(), Error> {
        Ok(())
    }

    fn drop_task(&self, _task: TaskId) {}

    fn shutdown(&self) {}
}

pub fn module() -> Arc<dyn NativeModule> {
    Arc::new(UnavailablePostgres {
        identity: NativeIdentity::new(
            "gh:hara-lang:std-db-postgres-ci-stub",
            "std.db.postgres",
            "hara-db-postgres",
            "hara.db-provider/1",
        )
        .expect("valid CI-only PostgreSQL module identity"),
    })
}
