#[allow(warnings)]
mod bindings;

use bindings::exports::hara::values_echo::value_echo::{Guest, Value};

struct ValuesEcho;

impl Guest for ValuesEcho {
    fn echo(value: Value) -> Value {
        value
    }
}

bindings::export!(ValuesEcho with_types_in bindings);
