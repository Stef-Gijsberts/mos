use std::collections::HashMap;
use std::fmt;

pub type Name = String;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Bind(Name, Value),
    Print(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Literal(Literal),
    Reference(Name),
    Lambda(Name, Box<Value>),
    BuiltinLambda(BuiltinLambda),
    Apply(Box<Value>, Box<Value>),
}

#[derive(Clone)]
pub struct BuiltinLambda {
    pub num_params: usize,
    pub args: Vec<Value>,
    pub apply: fn(args: Vec<Value>, context: &HashMap<Name, Value>) -> Value,
}

impl PartialEq for BuiltinLambda {
    fn eq(&self, other: &BuiltinLambda) -> bool {
        // maybe solve this differently than with pointer comparison?
        if self.apply as usize != other.apply as usize {
            return false;
        }

        if self.args != other.args {
            return false;
        }

        if self.num_params != other.num_params {
            return false;
        }

        true
    }
}

impl fmt::Debug for BuiltinLambda {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<BUILTIN LAMBDA>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Bytes(Vec<u8>),
    Integer(i128),
    Boolean(bool),
}
