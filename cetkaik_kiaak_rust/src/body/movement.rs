use cetkaik_core::absolute;
use nom::branch::alt;
use nom::character::complete::{char, one_of};
use nom::combinator::map;
use nom::combinator::opt;
use nom::error::{Error, ErrorKind};
use nom::multi::many_m_n;
use nom::Err;
use nom::IResult;

type PossiblyUnknown<T> = Option<T>;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Move {
    NoStepAndNoStick {
        src: absolute::Coord,
        prof: PossiblyUnknown<cetkaik_core::Profession>,
        dest: absolute::Coord,
    },

    NoStepAndWaterStick {
        src: absolute::Coord,
        prof: PossiblyUnknown<cetkaik_core::Profession>,
        dest: absolute::Coord,
        water_stick_size: PossiblyUnknown<i32>,
        water_stick_successful: bool,
    },

    StepAndNoStick {
        src: absolute::Coord,
        prof: PossiblyUnknown<cetkaik_core::Profession>,
        step: absolute::Coord,
        dest: absolute::Coord,
    },

    StepAndWaterStick {
        src: absolute::Coord,
        prof: PossiblyUnknown<cetkaik_core::Profession>,
        step: absolute::Coord,
        dest: absolute::Coord,
        water_stick_size: PossiblyUnknown<i32>,
        water_stick_successful: bool,
    },

    StepAndBridgeStick {
        src: absolute::Coord,
        prof: PossiblyUnknown<cetkaik_core::Profession>,
        step: absolute::Coord,
        dest: absolute::Coord,
        bridge_stick_size: PossiblyUnknown<i32>,
        bridge_stick_successful: bool,
    },

    StepAndBridgeStickAndWaterStick {
        src: absolute::Coord,
        prof: PossiblyUnknown<cetkaik_core::Profession>,
        step: absolute::Coord,
        dest: absolute::Coord,
        bridge_stick_size: PossiblyUnknown<i32>,
        /* The fact that water_stick_size exists assert that bridge_stick was successful */
        water_stick_size: PossiblyUnknown<i32>,
        water_stick_successful: bool,
    },

    TamNoStep {
        src: absolute::Coord,
        first_dest: PossiblyUnknown<absolute::Coord>,
        second_dest: absolute::Coord,
    },

    TamStepUnspecified {
        src: absolute::Coord,
        step: absolute::Coord,
        second_dest: absolute::Coord,
    },

    TamStepDuringFormer {
        src: absolute::Coord,
        step: absolute::Coord,
        first_dest: PossiblyUnknown<absolute::Coord>,
        second_dest: absolute::Coord,
    },

    TamStepDuringLatter {
        src: absolute::Coord,
        first_dest: PossiblyUnknown<absolute::Coord>,
        step: absolute::Coord,
        second_dest: absolute::Coord,
    },

    Parachute {
        color: cetkaik_core::Color,
        prof: cetkaik_core::Profession,
        dest: absolute::Coord,
    },
}

/// Examples:
/// ```
/// use cetkaik_kiaak::body::movement::{parse, Move};
/// use cetkaik_core::Profession;
/// use cetkaik_core::absolute::*;
/// assert_eq!(
///     parse("XU兵XY無撃裁"),
///     Ok((
///         "",
///         Move::NoStepAndNoStick {
///             src: Coord(Row::U, Column::X),
///             prof: Some(Profession::Kauk2),
///             dest: Coord(Row::Y, Column::X),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("LY弓ZY水或此無"),
///     Ok((
///         "",
///         Move::NoStepAndWaterStick {
///             src: Coord(Row::Y, Column::L),
///             prof: Some(Profession::Gua2),
///             dest: Coord(Row::Y, Column::Z),
///             water_stick_size: None,
///             water_stick_successful: false,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("LY弓ZY水一此無"),
///     Ok((
///         "",
///         Move::NoStepAndWaterStick {
///             src: Coord(Row::Y, Column::L),
///             prof: Some(Profession::Gua2),
///             dest: Coord(Row::Y, Column::Z),
///             water_stick_size: Some(1),
///             water_stick_successful: false,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("LY弓ZY水五"),
///     Ok((
///         "",
///         Move::NoStepAndWaterStick {
///             src: Coord(Row::Y, Column::L),
///             prof: Some(Profession::Gua2),
///             dest: Coord(Row::Y, Column::Z),
///             water_stick_size: Some(5),
///             water_stick_successful: true,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("LY弓ZY水或"),
///     Ok((
///         "",
///         Move::NoStepAndWaterStick {
///             src: Coord(Row::Y, Column::L),
///             prof: Some(Profession::Gua2),
///             dest: Coord(Row::Y, Column::Z),
///             water_stick_size: None,
///             water_stick_successful: true,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("XU兵XYXAU無撃裁"),
///     Ok((
///         "",
///         Move::StepAndNoStick {
///             src: Coord(Row::U, Column::X),
///             prof: Some(Profession::Kauk2),
///             step: Coord(Row::Y, Column::X),
///             dest: Coord(Row::AU, Column::X)
///         }
///     ))
/// );
/// assert_eq!(
///     parse("NY巫CYCO水五"),
///     Ok((
///         "",
///         Move::StepAndWaterStick {
///             src: Coord(Row::Y, Column::N),
///             prof: Some(Profession::Tuk2),
///             step: Coord(Row::Y, Column::C),
///             dest: Coord(Row::O, Column::C),
///             water_stick_size: Some(5),
///             water_stick_successful: true,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("ME弓MIMU橋四"),
///     Ok((
///         "",
///         Move::StepAndBridgeStick {
///             src: Coord(Row::E, Column::M),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::I, Column::M),
///             dest: Coord(Row::U, Column::M),
///             bridge_stick_size: Some(4),
///             bridge_stick_successful: true,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("ME弓MIMY橋或"),
///     Ok((
///         "",
///         Move::StepAndBridgeStick {
///             src: Coord(Row::E, Column::M),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::I, Column::M),
///             dest: Coord(Row::Y, Column::M),
///             bridge_stick_size: None,
///             bridge_stick_successful: true,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("ME弓MIMY橋或此無"),
///     Ok((
///         "",
///         Move::StepAndBridgeStick {
///             src: Coord(Row::E, Column::M),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::I, Column::M),
///             dest: Coord(Row::Y, Column::M),
///             bridge_stick_size: None,
///             bridge_stick_successful: false,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("ME弓MIMY橋一此無"),
///     Ok((
///         "",
///         Move::StepAndBridgeStick {
///             src: Coord(Row::E, Column::M),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::I, Column::M),
///             dest: Coord(Row::Y, Column::M),
///             bridge_stick_size: Some(1),
///             bridge_stick_successful: false,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("ME弓MIMY橋無此無"),
///     Ok((
///         "",
///         Move::StepAndBridgeStick {
///             src: Coord(Row::E, Column::M),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::I, Column::M),
///             dest: Coord(Row::Y, Column::M),
///             bridge_stick_size: Some(0),
///             bridge_stick_successful: false,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("LO弓NOCO橋四水五"),
///     Ok((
///         "",
///         Move::StepAndBridgeStickAndWaterStick {
///             src: Coord(Row::O, Column::L),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::O, Column::N),
///             dest: Coord(Row::O, Column::C),
///             bridge_stick_size: Some(4),
///             water_stick_size: Some(5),
///             water_stick_successful: true,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("LO弓NOCO橋四水一此無"),
///     Ok((
///         "",
///         Move::StepAndBridgeStickAndWaterStick {
///             src: Coord(Row::O, Column::L),
///             prof: Some(Profession::Gua2),
///             step: Coord(Row::O, Column::N),
///             dest: Coord(Row::O, Column::C),
///             bridge_stick_size: Some(4),
///             water_stick_size: Some(1),
///             water_stick_successful: false,
///         }
///     ))
/// );
/// assert_eq!(
///     parse("黒弓MY"),
///     Ok((
///         "",
///         Move::Parachute {
///             color: cetkaik_core::Color::Huok2,
///             prof: Profession::Gua2,
///             dest: Coord(Row::Y, Column::M),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("赤車CI"),
///     Ok((
///         "",
///         Move::Parachute {
///             color: cetkaik_core::Color::Kok1,
///             prof: Profession::Kaun1,
///             dest: Coord(Row::I, Column::C),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("KE皇KI"),
///     Ok((
///         "",
///         Move::TamNoStep {
///             src: Coord(Row::E, Column::K),
///             first_dest: None,
///             second_dest: Coord(Row::I, Column::K),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("KE皇[或]KI"),
///     Ok((
///         "",
///         Move::TamNoStep {
///             src: Coord(Row::E, Column::K),
///             first_dest: None,
///             second_dest: Coord(Row::I, Column::K),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("KE皇[LE]KI"),
///     Ok((
///         "",
///         Move::TamNoStep {
///             src: Coord(Row::E, Column::K),
///             first_dest: Some(Coord(Row::E, Column::L)),
///             second_dest: Coord(Row::I, Column::K),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("PAU皇CAIMAU"),
///     Ok((
///         "",
///         Move::TamStepUnspecified {
///             src: Coord(Row::AU, Column::P),
///             step: Coord(Row::AI, Column::C),
///             second_dest: Coord(Row::AU, Column::M),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("PAU皇[MAU]CAIMAU"),
///     Ok((
///         "",
///         Move::TamStepDuringLatter {
///             src: Coord(Row::AU, Column::P),
///             first_dest: Some(Coord(Row::AU, Column::M)),
///             step: Coord(Row::AI, Column::C),
///             second_dest: Coord(Row::AU, Column::M),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("PAU皇[或]CAIMAU"),
///     Ok((
///         "",
///         Move::TamStepDuringLatter {
///             src: Coord(Row::AU, Column::P),
///             first_dest: None,
///             step: Coord(Row::AI, Column::C),
///             second_dest: Coord(Row::AU, Column::M),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("KE皇LI[KE]KA"),
///     Ok((
///         "",
///         Move::TamStepDuringFormer {
///             src: Coord(Row::E, Column::K),
///             step: Coord(Row::I, Column::L),
///             first_dest: Some(Coord(Row::E, Column::K)),
///             second_dest: Coord(Row::A, Column::K),
///         }
///     ))
/// );
/// assert_eq!(
///     parse("KE皇LI[或]KA"),
///     Ok((
///         "",
///         Move::TamStepDuringFormer {
///             src: Coord(Row::E, Column::K),
///             step: Coord(Row::I, Column::L),
///             first_dest: None,
///             second_dest: Coord(Row::A, Column::K),
///         }
///     ))
/// );
/// ```
///
pub fn parse(s: &str) -> IResult<&str, Move> {
    let (rem, movement) = alt((
        parse_parachute,
        parse_tam_step_during_former,
        parse_tam_step_during_latter,
        parse_tam_step_unspecified,
        parse_tam_no_step,
        parse_step_and_bridge_stick_and_water_stick,
        parse_step_and_bridge_stick,
        parse_step_and_water_stick,
        parse_step_and_no_stick,
        parse_no_step_and_water_stick,
        parse_no_step_and_no_stick,
    ))(s)?;

    Ok((rem, movement))
}

use nom::bytes::complete::tag;

fn parse_no_step_and_no_stick(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, prof) = parse_profession_or_wildcard(rem)?;
    let (rem, dest) = parse_square(rem)?;
    let (rem, _) = tag("無撃裁")(rem)?;

    Ok((rem, Move::NoStepAndNoStick { src, prof, dest }))
}

fn parse_no_step_and_water_stick(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, prof) = parse_profession_or_wildcard(rem)?;
    let (rem, dest) = parse_square(rem)?;
    let (rem, (water_stick_size, water_stick_successful)) = parse_water_stick(rem)?;

    Ok((
        rem,
        Move::NoStepAndWaterStick {
            src,
            prof,
            dest,
            water_stick_size,
            water_stick_successful,
        },
    ))
}

fn parse_step_and_no_stick(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, prof) = parse_profession_or_wildcard(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, dest) = parse_square(rem)?;
    let (rem, _) = tag("無撃裁")(rem)?;

    Ok((
        rem,
        Move::StepAndNoStick {
            src,
            prof,
            step,
            dest,
        },
    ))
}

fn parse_step_and_water_stick(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, prof) = parse_profession_or_wildcard(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, dest) = parse_square(rem)?;
    let (rem, (water_stick_size, water_stick_successful)) = parse_water_stick(rem)?;

    Ok((
        rem,
        Move::StepAndWaterStick {
            src,
            prof,
            step,
            dest,
            water_stick_size,
            water_stick_successful,
        },
    ))
}

fn parse_step_and_bridge_stick(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, prof) = parse_profession_or_wildcard(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, dest) = parse_square(rem)?;
    let (rem, bridge_stick_size) = parse_bridge_stick_size(rem)?;
    let (rem, fail) = opt(tag("此無"))(rem)?;

    Ok((
        rem,
        Move::StepAndBridgeStick {
            src,
            prof,
            step,
            dest,
            bridge_stick_size,
            bridge_stick_successful: fail.is_none(),
        },
    ))
}

fn parse_step_and_bridge_stick_and_water_stick(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, prof) = parse_profession_or_wildcard(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, dest) = parse_square(rem)?;
    let (rem, bridge_stick_size) = parse_bridge_stick_size(rem)?;
    let (rem, (water_stick_size, water_stick_successful)) = parse_water_stick(rem)?;

    Ok((
        rem,
        Move::StepAndBridgeStickAndWaterStick {
            src,
            prof,
            step,
            dest,
            bridge_stick_size,
            water_stick_size,
            water_stick_successful,
        },
    ))
}

fn parse_tam_no_step(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, _) = char('皇')(rem)?;
    let (rem, tamsq) = opt(parse_tam_sqbracket)(rem)?;
    let first_dest: Option<absolute::Coord> = match tamsq {
        None | Some(None) => None,
        Some(Some(a)) => Some(a),
    };
    let (rem, second_dest) = parse_square(rem)?;

    Ok((
        rem,
        Move::TamNoStep {
            src,
            first_dest,
            second_dest,
        },
    ))
}

fn parse_tam_step_unspecified(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, _) = char('皇')(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, second_dest) = parse_square(rem)?;
    Ok((
        rem,
        Move::TamStepUnspecified {
            src,
            step,
            second_dest,
        },
    ))
}

fn parse_tam_step_during_former(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, _) = char('皇')(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, first_dest) = parse_tam_sqbracket(rem)?;
    let (rem, second_dest) = parse_square(rem)?;
    Ok((
        rem,
        Move::TamStepDuringFormer {
            src,
            step,
            first_dest,
            second_dest,
        },
    ))
}

fn parse_tam_step_during_latter(s: &str) -> IResult<&str, Move> {
    let (rem, src) = parse_square(s)?;
    let (rem, _) = char('皇')(rem)?;
    let (rem, first_dest) = parse_tam_sqbracket(rem)?;
    let (rem, step) = parse_square(rem)?;
    let (rem, second_dest) = parse_square(rem)?;
    Ok((
        rem,
        Move::TamStepDuringLatter {
            src,
            step,
            first_dest,
            second_dest,
        },
    ))
}

fn parse_parachute(s: &str) -> IResult<&str, Move> {
    let (rem, color) = one_of("黒赤")(s)?;
    let color = match color {
        '黒' => cetkaik_core::Color::Huok2,
        '赤' => cetkaik_core::Color::Kok1,
        _ => unreachable!(),
    };
    let (rem, prof) = parse_profession(rem)?;
    let (rem, dest) = parse_square(rem)?;
    Ok((rem, Move::Parachute { color, prof, dest }))
}

/// Examples:
/// ```
/// use cetkaik_kiaak::body::movement::parse_tam_sqbracket;
/// use cetkaik_core::absolute;
/// assert_eq!(parse_tam_sqbracket("[TY]"), Ok(("", Some(absolute::Coord(absolute::Row::Y, absolute::Column::T)))));
/// assert_eq!(parse_tam_sqbracket("[或]"), Ok(("", None)))
/// ```
///
pub fn parse_tam_sqbracket(s: &str) -> IResult<&str, PossiblyUnknown<absolute::Coord>> {
    let (rem, _) = char('[')(s)?;
    let (rem, opt_coord) = alt((map(parse_square, Some), map(char('或'), |_| None)))(rem)?;
    let (rem, _) = char(']')(rem)?;

    Ok((rem, opt_coord))
}

use std::collections::HashMap;
fn one_of_and_map<'a, U, Error: nom::error::ParseError<&'a str>>(
    list: HashMap<char, U>,
) -> impl Fn(&'a str) -> IResult<&str, U, Error>
where
    U: Copy,
{
    use nom::AsChar;
    use nom::InputIter;
    use nom::Slice;
    move |i: &str| match (i).iter_elements().next().map(|c| (c, list.get(&c))) {
        Some((c, Some(u))) => Ok((i.slice(c.len()..), *u)),
        _ => Err(Err::Error(Error::from_error_kind(i, ErrorKind::OneOf))),
    }
}

use maplit::hashmap;

/// Examples:
/// ```
/// use cetkaik_kiaak::body::movement::parse_profession;
/// use cetkaik_core::Profession;
/// assert_eq!(parse_profession("船"), Ok(("", Profession::Nuak1)));
/// assert_eq!(parse_profession("巫"), Ok(("", Profession::Tuk2)))
/// ```
///
pub fn parse_profession(s: &str) -> IResult<&str, cetkaik_core::Profession> {
    use cetkaik_core::Profession;
    one_of_and_map(hashmap![
        '船' => Profession::Nuak1,
        '兵' => Profession::Kauk2,
        '弓' => Profession::Gua2,
        '車' => Profession::Kaun1,
        '虎' => Profession::Dau2,
        '馬' => Profession::Maun1,
        '筆' => Profession::Kua2,
        '巫' => Profession::Tuk2,
        '将' => Profession::Uai1,
        '王' => Profession::Io,
    ])(s)
}

/// Examples:
/// ```
/// use cetkaik_kiaak::body::movement::parse_profession_or_wildcard;
/// use cetkaik_core::Profession;
/// assert_eq!(parse_profession_or_wildcard("船"), Ok(("", Some(Profession::Nuak1))));
/// assert_eq!(parse_profession_or_wildcard("巫"), Ok(("", Some(Profession::Tuk2))));
/// assert_eq!(parse_profession_or_wildcard("片"), Ok(("", None)))
/// ```
///
pub fn parse_profession_or_wildcard(
    s: &str,
) -> IResult<&str, PossiblyUnknown<cetkaik_core::Profession>> {
    use cetkaik_core::Profession;
    one_of_and_map(hashmap! {
        '船' => Some(Profession::Nuak1),
        '兵' => Some(Profession::Kauk2),
        '弓' => Some(Profession::Gua2),
        '車' => Some(Profession::Kaun1),
        '虎' => Some(Profession::Dau2),
        '馬' => Some(Profession::Maun1),
        '筆' => Some(Profession::Kua2),
        '巫' => Some(Profession::Tuk2),
        '将' => Some(Profession::Uai1),
        '王' => Some(Profession::Io),
        '片' => None
    })(s)
}

pub fn parse_bridge_stick_size(s: &str) -> IResult<&str, PossiblyUnknown<i32>> {
    let (rem, _) = char('橋')(s)?;
    one_of_and_map(hashmap! {
        '或' => None,
        '無' => Some(0),
        '一' => Some(1),
        '二' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
    })(rem)
}

pub fn parse_water_stick(s: &str) -> IResult<&str, (PossiblyUnknown<i32>, bool)> {
    let (rem, _) = char('水')(s)?;
    let (rem, vec) = many_m_n(1, 3, one_of("或無一二三四五此"))(rem)?;

    let result = match vec.as_slice() {
        ['無', '此', '無'] => (Some(0), false),
        ['一', '此', '無'] => (Some(1), false),
        ['二', '此', '無'] => (Some(2), false),
        ['三'] => (Some(3), true),
        ['四'] => (Some(4), true),
        ['五'] => (Some(5), true),
        ['或'] => (None, true), /* unspecified but successful */
        ['或', '此', '無'] => (None, false), /* unspecified but not successful */
        _ => return Err(Err::Error(Error::new(rem, ErrorKind::Verify))),
        /*(
            "Unparsable fragment {:?} while parsing water stick",
            vec.into_iter().collect::<String>()
        ),*/
    };

    Ok((rem, result))
}

pub fn parse_square(s: &str) -> IResult<&str, absolute::Coord> {
    let (rem, column) = one_of("KLNTZXCMP")(s)?;
    let (rem, row) = many_m_n(1, 2, one_of("AEIOUY"))(rem)?;

    let coord = absolute::parse_coord(&format!(
        "{}{}",
        column,
        row.into_iter().collect::<String>()
    ))
    .ok_or_else(|| Err::Error(Error::new(rem, ErrorKind::Verify)))?;
    Ok((rem, coord))
}
