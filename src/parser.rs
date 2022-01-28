use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::multispace1,
    combinator::map,
    multi::many0,
    sequence::{preceded, terminated},
    IResult,
};

#[inline]
fn bakka_comment(input: &[u8]) -> IResult<&[u8], ()> {
    map(terminated(tag("# bakka:"), line_rest), |_| ())(input)
}

#[inline]
fn line_rest(input: &[u8]) -> IResult<&[u8], ()> {
    map(take_until("\n"), |_| ())(input)
}

#[inline]
fn whitespace(input: &[u8]) -> IResult<&[u8], ()> {
    alt((map(multispace1, |_| ()), bakka_comment))(input)
}

#[inline]
fn hr(input: &[u8]) -> IResult<&[u8], ()> {
    map(many0(whitespace), |_| ())(input)
}

#[inline]
fn single_line(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("\n")(input)
}

#[inline]
fn parse_autobuild_file(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    many0(preceded(hr, single_line))(input)
}

pub fn handle_autobuild_file(input: &[u8]) -> Result<Vec<u8>> {
    let (_, output) = parse_autobuild_file(input)
        .map_err(|e| anyhow!("Cannout handle autobuild file! why: {}", e))?;
    let mut buf = Vec::new();
    for i in output.into_iter() {
        buf.extend(i);
        buf.push(b'\n');
    }

    Ok(buf)
}

#[test]
fn test_parser() {
    let input = b"aaa\n# bakka: bbb\nccc\n";
    assert_eq!(
        std::str::from_utf8(&handle_autobuild_file(input).unwrap()).unwrap(),
        "aaa\nccc\n"
    );
}
