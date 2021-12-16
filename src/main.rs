use std::collections::HashMap;
use std::env;
use std::fs;

use colored::Colorize;

mod ast;
mod formatter;
mod parser;

use ast::{BuiltinLambda, Literal, Name, Program, Statement, Value};

fn substitute_fully(mut value: Value, name: Name, referent: Value) -> Value {
    fn sub_mut(value: &mut Value, name: &Name, referent: &Value) {
        match value {
            Value::Reference(hay) => {
                if hay == name {
                    *value = referent.clone()
                }
            }
            Value::Apply(l, r) => {
                sub_mut(l.as_mut(), name, referent);
                sub_mut(r.as_mut(), name, referent);
            }
            Value::Lambda(par_name, subval) => {
                if name != par_name {
                    sub_mut(subval, name, referent);
                }
            }
            Value::Literal(_) => {}
            Value::BuiltinLambda(_) => {}
        }
    }

    sub_mut(&mut value, &name, &referent);
    value
}

fn evaluate(value: Value, context: &HashMap<Name, Value>) -> Value {
    match value {
        Value::Reference(ref name) => match context.get(name) {
            Some(referent) => evaluate(referent.clone(), context),
            // None => value,
            None => panic!("Value with name '{}' does not exist in the context", name)
        },
        Value::Apply(f, arg) => {
            let ev_f = evaluate(*f, context);

            match ev_f {
                Value::BuiltinLambda(BuiltinLambda {
                    num_params,
                    mut args,
                    apply,
                    name,
                }) => {
                    args.push(*arg);

                    if args.len() == num_params {
                        apply(args, context)
                    } else {
                        Value::BuiltinLambda(BuiltinLambda {
                            num_params,
                            args,
                            apply,
                            name,
                        })
                    }
                }
                Value::Lambda(parname, subvalue) => {
                    evaluate(substitute_fully(*subvalue, parname, *arg), context)
                }
                _ => Value::Apply(Box::new(ev_f), arg),
            }
        }
        x => x,
    }
}

fn context_with_builtins() -> HashMap<Name, Value> {
    fn builtin_add(mut args: Vec<Value>, context: &HashMap<Name, Value>) -> Value {
        let rval = args.pop().unwrap();
        let lval = args.pop().unwrap();

        let l = evaluate(lval, context);
        let r = evaluate(rval, context);

        if let (Value::Literal(Literal::Integer(l_int)), Value::Literal(Literal::Integer(r_int))) =
            (l, r)
        {
            Value::Literal(Literal::Integer(l_int + r_int))
        } else {
            panic!("Add called with non-integer")
        }
    }

    fn builtin_mul(mut args: Vec<Value>, context: &HashMap<Name, Value>) -> Value {
        let rval = args.pop().unwrap();
        let lval = args.pop().unwrap();

        let l = evaluate(lval, context);
        let r = evaluate(rval, context);

        if let (Value::Literal(Literal::Integer(l_int)), Value::Literal(Literal::Integer(r_int))) =
            (l, r)
        {
            Value::Literal(Literal::Integer(l_int * r_int))
        } else {
            panic!("Mul called with non-integer")
        }
    }

    fn builtin_div(mut args: Vec<Value>, context: &HashMap<Name, Value>) -> Value {
        let rval = args.pop().unwrap();
        let lval = args.pop().unwrap();

        let l = evaluate(lval, context);
        let r = evaluate(rval, context);

        if let (Value::Literal(Literal::Integer(l_int)), Value::Literal(Literal::Integer(r_int))) =
            (l, r)
        {
            Value::Literal(Literal::Integer(l_int / r_int))
        } else {
            panic!("Div called with non-integer")
        }
    }

    fn builtin_rem(mut args: Vec<Value>, context: &HashMap<Name, Value>) -> Value {
        let rval = args.pop().unwrap();
        let lval = args.pop().unwrap();

        let l = evaluate(lval, context);
        let r = evaluate(rval, context);

        if let (Value::Literal(Literal::Integer(l_int)), Value::Literal(Literal::Integer(r_int))) =
            (l, r)
        {
            Value::Literal(Literal::Integer(l_int % r_int))
        } else {
            panic!("Rem called with non-integer")
        }
    }

    fn builtin_if(mut args: Vec<Value>, context: &HashMap<Name, Value>) -> Value {
        let if_false = args.pop().unwrap();
        let if_true = args.pop().unwrap();
        let condition = args.pop().unwrap();

        if let Value::Literal(Literal::Boolean(b)) = evaluate(condition, context) {
            if b {
                evaluate(if_true, context)
            } else {
                evaluate(if_false, context)
            }
        } else {
            panic!("If called with non-boolean")
        }
    }

    fn builtin_eq(mut args: Vec<Value>, context: &HashMap<Name, Value>) -> Value {
        let rval = args.pop().unwrap();
        let lval = args.pop().unwrap();

        let l = evaluate(lval, context);
        let r = evaluate(rval, context);

        Value::Literal(Literal::Boolean(l == r))
    }

    let mut context = HashMap::new();

    context.insert(
        "if".to_owned(),
        Value::BuiltinLambda(BuiltinLambda {
            name: "if".to_owned(),
            apply: builtin_if,
            args: vec![],
            num_params: 3,
        }),
    );

    context.insert(
        "eq".to_owned(),
        Value::BuiltinLambda(BuiltinLambda {
            name: "eq".to_owned(),
            apply: builtin_eq,
            args: vec![],
            num_params: 2,
        }),
    );

    context.insert(
        "add".to_owned(),
        Value::BuiltinLambda(BuiltinLambda {
            name: "add".to_owned(),
            apply: builtin_add,
            args: vec![],
            num_params: 2,
        }),
    );

    context.insert(
        "mul".to_owned(),
        Value::BuiltinLambda(BuiltinLambda {
            name: "mul".to_owned(),
            apply: builtin_mul,
            args: vec![],
            num_params: 2,
        }),
    );

    context.insert(
        "div".to_owned(),
        Value::BuiltinLambda(BuiltinLambda {
            name: "div".to_owned(),
            apply: builtin_div,
            args: vec![],
            num_params: 2,
        }),
    );

    context.insert(
        "rem".to_owned(),
        Value::BuiltinLambda(BuiltinLambda {
            name: "rem".to_owned(),
            apply: builtin_rem,
            args: vec![],
            num_params: 2,
        }),
    );

    context
}

fn execute_statement(statement: Statement, context: &mut HashMap<Name, Value>) -> Option<String> {
    match statement {
        Statement::Bind(name, value) => {
            if context.contains_key(&name) {
                panic!("cannot redefine {}", name);
            }

            context.insert(name, value);
            None
        }

        Statement::Print(value) => Some(format!("{}", evaluate(value, &context))),
    }
}

fn run(program: Program) {
    let mut context = context_with_builtins();

    for statement in program.statements {
        if let Some(to_be_printed) = execute_statement(statement, &mut context) {
            eprintln!("{}", to_be_printed);
        }
    }
}

fn repl() {
    let history_path = dirs::home_dir().unwrap().join(".moshistory");

    let mut context = context_with_builtins();
    let mut rl = rustyline::Editor::<()>::new();

    rl.load_history(&history_path);

    while let Ok(line) = rl.readline("") {
        if line == "" {
            continue;
        }

        match parser::parse_statement(&line) {
            Ok(statement) => {
                rl.add_history_entry(line.as_str());
                if let Some(to_be_printed) = execute_statement(statement, &mut context) {
                    eprintln!("{}", to_be_printed.green());
                }
            }
            Err(e) => eprintln!("{}", format!("{}", e).red()),
        }
    }


    rl.save_history(&history_path).unwrap();
}

pub fn main() {
    if let Some(source_path) = env::args().nth(1) {
        let source = fs::read_to_string(source_path).unwrap();

        match parser::parse(&source) {
            Err(e) => eprintln!("{}", format!("{}", e).red()),
            Ok(ast) => run(ast),
        }
    } else {
        repl();
    }
}
