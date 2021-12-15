use std::fmt;

use crate::ast::{Literal, Name, Program, Statement, Value};

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::combinator::{eof, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::Parser;

use nom_supreme::error::ErrorTree;
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

use nom_locate::LocatedSpan;

type Span<'a> = LocatedSpan<&'a str>;
type PError<'a> = ErrorTree<Span<'a>>;
type PResult<'a, O> = nom::IResult<Span<'a>, O, PError<'a>>;

#[derive(Debug)]
pub struct Error<'a>(nom::Err<PError<'a>>);

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            nom::Err::Error(e) => write!(f, "{}", e),
            nom::Err::Failure(e) => write!(f, "{}", e),
            nom::Err::Incomplete(_) => write!(f, "{:?}", self.0)
        }
    }
}


pub fn parse(s: &str) -> Result<Program, Error> {
    let span = Span::from(s);
    program(span).map(|(_rem, res)| res).map_err(Error)
}

pub fn parse_statement(s: &str) -> Result<Statement, Error> {
    let span = Span::from(s);
    terminated(statement, eof)(span).map(|(_rem, res)| res).map_err(Error)
}

fn program(s: Span) -> PResult<Program> {
    terminated(many0(ws(statement)), eof)
        .map(|statements| Program { statements })
        .context("Program")
        .parse(s)
}

fn statement(s: Span) -> PResult<Statement> {
    alt((bind, print)).context("Statement").parse(s)
}

fn print(s: Span) -> PResult<Statement> {
    ws(lambda)
        .map(Statement::Print)
        .context("Print-statement")
        .parse(s)
}

fn bind(s: Span) -> PResult<Statement> {
    let assign = ws(tag("="));

    separated_pair(identifier, assign, ws(lambda))
        .map(|(iden, val)| Statement::Bind(iden, val))
        .context("Bind-statement")
        .parse(s)
}

fn lambda(s: Span) -> PResult<Value> {
    alt((
        separated_pair(preceded(char('\\'), name), ws(tag("->")), lambda)
            .map(|(name, sub)| Value::Lambda(name, Box::new(sub))),
        application,
    ))
    .parse(s)
}

fn name(s: Span) -> PResult<Name> {
    identifier(s)
}

fn make_application(mut values: Vec<Value>) -> Value {
    if values.len() == 1 {
        values.pop().unwrap()
    } else if values.len() == 2 {
        let r = values.pop().unwrap();
        let l = values.pop().unwrap();
        Value::Apply(Box::new(l), Box::new(r))
    } else {
        let r = values.pop().unwrap();
        Value::Apply(Box::new(make_application(values)), Box::new(r))
    }
}

fn application(s: Span) -> PResult<Value> {
    fn item(s: Span) -> PResult<Value> {
        alt((application, reference)).parse(s)
    }

    let list = delimited(char('('), many1(ws(item)), char(')')).map(make_application);

    alt((list, reference)).parse(s)
}

fn reference(s: Span) -> PResult<Value> {
    alt((
        literal.map(Value::Literal),
        identifier.map(Value::Reference),
    ))
    .parse(s)
}

fn literal(s: Span) -> PResult<Literal> {
    let boolean = alt((
        tag("True").map(|_| Literal::Boolean(true)),
        tag("False").map(|_| Literal::Boolean(false)),
    ));
    let bytes = delimited(tag("\""), is_not("\""), tag("\""))
        .map(|span: Span| *span.fragment())
        .map(str::as_bytes)
        .map(Vec::from)
        .map(Literal::Bytes);
    let integer = nom::character::complete::i128.map(Literal::Integer);

    alt((boolean, bytes, integer))(s)
}

fn identifier(s: Span) -> PResult<String> {
    recognize(tuple((
        alpha1,
        many0(alt((alphanumeric1, tag("_")))),
        opt(char('\'')),
    )))
    .context("Identifier")
    .map(|span: Span| *span.fragment())
    .map(String::from)
    .parse(s)
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes both leading and trailing whitespace, returning the output of
/// `inner`.
fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(Span<'a>) -> PResult<O>
where
    F: Fn(Span<'a>) -> PResult<O>,
{
    delimited(multispace0, inner, multispace0)
}
