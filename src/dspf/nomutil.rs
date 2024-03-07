use color_eyre::{eyre::Ok, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alphanumeric1, line_ending, not_line_ending, space0},
    combinator::{not, recognize},
    error::ParseError,
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};

pub fn ws_no_cont<'a, F, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(space0, inner, space0)
}

pub fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    let space_or_cont = many0(alt((tag(" "), tag("\t"), tag("\n+"), tag("\n*+"))));

    // TODO: do we need the continuation before AND after?
    delimited(space0, inner, space_or_cont)
}

#[test]
fn test_ws() {
    let input = "  123  notseparatedabc  def ";
    let res = tuple((
        ws(tag::<&str, &str, ()>("123")),
        tag("not"),
        tag("separated"),
        ws(tag("abc")),
        ws(tag("def")),
    ))
    .parse(input);
    assert_eq!(
        res.unwrap(),
        ("", ("123", "not", "separated", "abc", "def"))
    );
}

pub fn qstring(input: &str) -> IResult<&str, &str> {
    ws(delimited(tag("\""), is_not("\"\n"), tag("\""))).parse(input)
}

pub fn optionally_quoted_string(input: &str) -> IResult<&str, &str> {
    ws(alt((
        delimited(tag("\""), is_not("\"\n"), tag("\"")),
        is_not("\"\n"),
    )))
    .parse(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alphanumeric1,
        many0_count(alt((alphanumeric1, tag("_"), tag("<"), tag(">")))),
    ))
    .parse(input)
}

pub fn comment_line(input: &str) -> IResult<&str, &str> {
    preceded(tag("*").and(not(tag("|"))), not_line_ending).parse(input)
}

pub fn comment_lines(input: &str) -> IResult<&str, Vec<&str>> {
    many0(delimited(
        tag("*").and(not(tag("|"))).and(not(tag("LAYER_MAP"))),
        not_line_ending,
        line_ending,
    ))
    .parse(input)
}

pub fn empty_or_comment(input: &str) -> IResult<&str, Vec<&str>> {
    many0(alt((
        terminated(space0, line_ending),
        delimited(
            tag("*").and(not(tag("|"))).and(not(tag("LAYER_MAP"))),
            not_line_ending,
            line_ending,
        ),
    )))
    .parse(input)
}
