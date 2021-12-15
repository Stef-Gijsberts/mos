use std::fmt;

use crate::ast::{Literal, Program, Statement, Value};

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for statement in &self.statements {
            writeln!(f, "{}", statement)?;
        }

        Ok(())
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Bind(name, value) => write!(f, "{} = {};", name, value),
            Statement::Print(value) => write!(f, "print {};", value),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::BuiltinLambda(_builtin) => write!(f, "<BUILTIN LAMBDA>"),
            Value::Apply(l, r) => write!(f, "({} {})", l, r),
            Value::Lambda(name, val) => write!(f, "\\{} -> {}", name, val),
            Value::Literal(lit) => write!(f, "{}", lit),
            Value::Reference(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Bytes(bytes) => write!(f, "\"{}\"", String::from_utf8(bytes.clone()).unwrap()),
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::Boolean(true) => write!(f, "True"),
            Literal::Boolean(false) => write!(f, "False"),
        }
    }
}
