use std::collections::HashMap;
use std::sync::Arc;

/// Context for the interpreter.
///
/// Contains values for variables and modules
pub struct Context {
    modules: HashMap<String, Module>,
    variables: HashMap<String, Value>,
    parent: Option<Arc<Context>>,
}

pub enum Value {
    Bool(bool),
    Number(f64),
    Text(String),
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Bool(b) => b,
            Value::Number(x) => x.is_normal(),
            Value::Text(ref txt) => !txt.is_empty(),
        }
    }
}

pub struct Module {}
