use nom::{Parser, character::complete::char, combinator::opt};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    IResult,
    nmea_content::{
        Parsable,
        parse::{date_full_year, time, utc_offset},
    },
};

/// ZDA - Time & Date - UTC, day, month, year and local time zone
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_zda_time_date_utc_day_month_year_and_local_time_zone>
///
/// ```text
///         1         2  3  4    5  6  
///         |         |  |  |    |  |  
///  $--ZDA,hhmmss.ss,xx,xx,xxxx,xx,xx*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct ZDA {
    /// Fix time in UTC
    pub time: Option<time::Time>,
    /// Fix date in UTC
    pub date: Option<time::Date>,
    /// Local zone description, offset from UTC
    pub utc_offset: Option<time::UtcOffset>,
}

impl<'a> Parsable<'a> for ZDA {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, time) = opt(time).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, date) = date_full_year.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, utc_offset) = utc_offset.parse(i)?;

        Ok((
            i,
            ZDA {
                time,
                date,
                utc_offset,
            },
        ))
    }
}

impl From<time::OffsetDateTime> for ZDA {
    fn from(value: time::OffsetDateTime) -> Self {
        ZDA {
            time: Some(value.time()),
            date: Some(value.date()),
            utc_offset: Some(value.offset()),
        }
    }
}

impl From<ZDA> for Option<time::OffsetDateTime> {
    fn from(value: ZDA) -> Self {
        if let (Some(time), Some(date), Some(utc_offset)) =
            (value.time, value.date, value.utc_offset)
        {
            Some(time::OffsetDateTime::new_in_offset(date, time, utc_offset))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zda_parsing() {
        let cases = [
            "123456.78,01,01,2023,,",
            "132502.00,11,07,2025,+03,00",
            ",,,,,",
            "132502.00,11,07,2025,,",
            "132502.00,,,,,",
            "132502.00,,,,-03,30",
            "120000.00,29,02,2024,01,00",
            "101112.13,12,11,2025,+14,00",
        ];

        for &input in &cases {
            let result = ZDA::parser(input);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }

        let cases = [
            "132502.00,11,,,,",
            "132502.00,,07,2025,,",
            "123456.78,01,,2023,,",
            "132502.00,00,07,2025,,",
            "132502.00,11,07,,+03,",
        ];

        for &input in &cases {
            let result = ZDA::parser(input);
            assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
