use std::collections::HashMap;
use crate::ast::Stmt;

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Text(String),
    Bool(bool),
    Array(Vec<Value>),
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Null,
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "رقم",
            Value::Text(_) => "نص",
            Value::Bool(_) => "منطقي",
            Value::Array(_) => "مصفوفة",
            Value::Function { .. } => "دالة",
            Value::Null => "عدم",
        }
    }

    pub fn as_number(&self) -> Result<i64, String> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(format!("متوقع رقم، وُجد {}", self.type_name())),
        }
    }

    pub fn as_array(&self) -> Result<Vec<Value>, String> {
        match self {
            Value::Array(a) => Ok(a.clone()),
            _ => Err(format!("متوقع مصفوفة، وُجد {}", self.type_name())),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0,
            Value::Null => false,
            Value::Array(a) => !a.is_empty(),
            _ => true,
        }
    }
}

pub type Env = HashMap<String, Value>;
