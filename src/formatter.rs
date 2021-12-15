use std::fmt;

use crate::ast::{BuiltinLambda, Literal, Program, Statement, Value};

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

fn application_list(val: &Value) -> Vec<&Value> {
    match val {
        Value::Apply(l, r) => {
            let mut list = application_list(l);
            list.push(r);
            list
        }
        other => vec![other],
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::BuiltinLambda(builtin) => write!(f, "{}", builtin),
            Value::Apply(l, r) => write!(
                f,
                "({})",
                application_list(self)
                    .into_iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Value::Lambda(name, val) => write!(f, "\\{} -> {}", name, val),
            Value::Literal(lit) => write!(f, "{}", lit),
            Value::Reference(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for BuiltinLambda {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let arglist = self
            .args
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(" ");

        if arglist.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "({} {})", self.name, arglist)
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
