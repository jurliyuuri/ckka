#[warn(clippy::pedantic)]
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Header {
    pub info: Vec<HeaderElem>,
    pub players: Option<(PlayerAndPoint, PlayerAndPoint)>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PlayerAndPoint {
    pub player_name: String,
    pub point: i64,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum HeaderElem {
    Value(String),
    KeyedValue(String, String),
}

use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::character::complete::*;
use nom::error::*;
use nom::multi::many0;
use nom::multi::many_m_n;
use nom::Err;
use nom::IResult;

pub fn parse_pekzep_numeral(s: &str) -> IResult<&str, i64> {
    let (no_used, vec) =
        many_m_n(1, 1000, one_of("無下一二三四五六七八九十百万億"))(s)?;
    match super::pekzep_numeral::analyze(&vec) {
        Some(n) => Ok((no_used, n)),
        None => Err(Err::Error(Error::new(no_used, ErrorKind::Verify))), /* unparsable pekzep numeral */
    }
}

pub fn parse_braced_string(s: &str, open: char, close: char) -> IResult<&str, &str> {
    let (no_used, vec) = many0(char('#'))(s)?;
    let (no_used, _) = char(open)(no_used)?;

    // `}####` if vec.len() == 4
    let end_pattern = format!(
        "{}{}",
        close,
        (0..vec.len()).map(|_| "#").collect::<String>()
    );
    let (no_used, in_string) = take_until(&*end_pattern)(no_used)?;
    let (no_used, _) = tag(&*end_pattern)(no_used)?;

    let (no_used, _) = skip_spaces_and_newlines(no_used)?;

    if in_string.contains('\n') || in_string.contains('\r') {
        return Err(Err::Error(Error::new(no_used, ErrorKind::Verify)));
        /* neither key nor value in the header can contain a newline */
    }

    Ok((no_used, in_string))
}

pub fn header_elem_parser(s: &str) -> IResult<&str, HeaderElem> {
    let (no_used, in_string) = parse_braced_string(s, '{', '}')?;
    Ok((no_used, {
        let mut splitter = in_string.splitn(2, ':');
        let first = splitter.next().unwrap();
        let second = splitter.next();
        match (first, second) {
            ("", Some(val)) => HeaderElem::Value(val.to_owned()),
            (key, Some(value)) => HeaderElem::KeyedValue(key.to_owned(), value.to_owned()),
            (val, None) => HeaderElem::Value(val.to_owned()),
        }
    }))
}

fn skip_spaces_and_newlines(s: &str) -> IResult<&str, ()> {
    let (no_used, _) = many0(one_of("\t\r\n \u{00a0}\u{3000}"))(s)?;
    Ok((no_used, ()))
}

pub fn player_and_point_parser(s: &str) -> IResult<&str, (String, Option<i64>)> {
    // TODO implement parsing point
    let (no_used, player_name) = parse_braced_string(s, '[', ']')?;
    let (no_used, _) = skip_spaces_and_newlines(no_used)?;
    let (no_used, v) = many_m_n(0, 1, parse_pekzep_numeral)(no_used)?;
    let (no_used, _) = skip_spaces_and_newlines(no_used)?;
    Ok((
        no_used,
        (
            player_name.to_owned(),
            match v.as_slice() {
                [] => None,
                [num] => Some(*num),
                _ => unreachable!(),
            },
        ),
    ))
}

pub fn header_parser(input: &str) -> IResult<&str, Header> {
    let (no_used, _) = skip_spaces_and_newlines(input)?;
    let (no_used, info) = many0(header_elem_parser)(no_used)?;
    let (no_used, vec2) = many_m_n(0, 2, player_and_point_parser)(no_used)?;
    let players = match vec2.as_slice() {
        [] => None,
        [q, r] => {
            let (n1, p1) = q.clone();
            let (n2, p2) = r.clone();
            let (p1, p2) = match (p1, p2) {
                (Some(b), Some(d)) => (b, d),
                (Some(b), None) => (b, 40 - b),
                (None, Some(d)) => (40 - d, d),
                (None, None) => (20, 20),
            };

            Some((
                PlayerAndPoint {
                    player_name: n1,
                    point: p1,
                },
                PlayerAndPoint {
                    player_name: n2,
                    point: p2,
                },
            ))
        }
        _ => return Err(Err::Error(Error::new(no_used, ErrorKind::Verify))), /* only one player found */
    };

    Ok((no_used, Header { info, players }))
}