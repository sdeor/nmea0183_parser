use nom::{
    AsChar, IResult, Input, Parser,
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{char, i8, one_of, u8, u16},
    combinator::{map_res, opt, value},
    error::{FromExternalError, ParseError},
    number::complete::{double, float},
    sequence::separated_pair,
};

pub fn opt_with_unit<I, O, E, F>(
    parser: F,
    unit: char,
) -> impl Parser<I, Output = Option<O>, Error = E>
where
    I: Input,
    <I as Input>::Item: AsChar,
    F: Parser<I, Output = O, Error = E>,
    E: ParseError<I>,
{
    separated_pair(opt(parser), char(','), opt(char(unit))).map(|(value, unit)| unit.and(value))
}

pub fn time<'a, E>(i: &'a str) -> IResult<&'a str, time::Time, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, &'static str>,
{
    map_res(
        (take(2u8).and_then(u8), take(2u8).and_then(u8), float),
        |(hour, minute, second)| {
            if second.is_sign_negative() {
                return Err("Invalid time: second is negative");
            }

            let milliseconds = second.fract() * 1000.0;
            let second = second.trunc();

            time::Time::from_hms_milli(hour, minute, second as u8, milliseconds as u16)
                .or(Err("Invalid time: hour, minute, or second out of range"))
        },
    )
    .parse(i)
}

pub fn date<'a, E>(i: &'a str) -> IResult<&'a str, time::Date, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, &'static str>,
{
    map_res(
        (
            take(2u8).and_then(u8),
            take(2u8).and_then(u8),
            take(2u8).and_then(u8),
        ),
        |(day, month, year)| {
            let month = month
                .try_into()
                .or(Err("Invalid date: month out of range"))?;

            let year = match year {
                83..=99 => year as i32 + 1900,
                _ => year as i32 + 2000,
            };

            time::Date::from_calendar_date(year, month, day)
                .or(Err("Invalid date: year, month, or day out of range"))
        },
    )
    .parse(i)
}

pub fn date_full_year<'a, E>(i: &'a str) -> IResult<&'a str, Option<time::Date>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, &'static str>,
{
    alt((
        tag(",,").map(|_| None),
        map_res(
            (u8, char(','), u8, char(','), u16),
            |(day, _, month, _, year)| {
                let month = month
                    .try_into()
                    .or(Err("Invalid date: month out of range"))?;

                time::Date::from_calendar_date(year as i32, month, day)
                    .or(Err("Invalid date: year, month, or day out of range"))
            },
        )
        .map(Some),
    ))
    .parse(i)
}

pub fn utc_offset<'a, E>(i: &'a str) -> IResult<&'a str, Option<time::UtcOffset>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, &'static str>,
{
    alt((
        value(None, char(',')),
        map_res(
            separated_pair((opt(one_of("+-")), i8), char(','), i8),
            |((sign, hours), minutes)| {
                let (hours, minutes) = match sign {
                    Some('-') => (-hours, -minutes),
                    _ => (hours, minutes),
                };

                time::UtcOffset::from_hms(hours, minutes, 0)
                    .or(Err("Invalid UTC offset: hours or minutes out of range"))
            },
        )
        .map(Some),
    ))
    .parse(i)
}

pub fn latlon<'a, E>(i: &'a str) -> IResult<&'a str, (Option<f64>, Option<f64>), E>
where
    E: ParseError<&'a str>,
{
    alt((
        value(None, tag(",,,")),
        separated_pair(
            (take(2u8).and_then(u8), double, char(','), one_of("NS")).map(|(deg, min, _, dir)| {
                let mut lat = deg as f64 + (min / 60.0);
                if dir == 'S' {
                    lat = -lat;
                }
                lat
            }),
            char(','),
            (take(3u8).and_then(u8), double, char(','), one_of("EW")).map(|(deg, min, _, dir)| {
                let mut lon = deg as f64 + (min / 60.0);
                if dir == 'W' {
                    lon = -lon;
                }
                lon
            }),
        )
        .map(Some),
    ))
    .map(Option::unzip)
    .parse(i)
}

pub fn magnetic_variation<'a, E>(i: &'a str) -> IResult<&'a str, Option<f32>, E>
where
    E: ParseError<&'a str>,
{
    alt((
        value(None, char(',')),
        separated_pair(float, char(','), one_of("EW")).map(|(value, dir)| {
            if dir == 'W' {
                Some(-value)
            } else {
                Some(value)
            }
        }),
    ))
    .parse(i)
}
