#[cfg(feature = "nmea-v4-11")]
use nom::{
    Input,
    combinator::{cond, opt},
    number::complete::hex_u32,
};
use nom::{
    Parser,
    character::complete::{char, u8},
    multi::many0,
    sequence::preceded,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v4-11")]
use crate::nmea_content::SignalId;
use crate::{
    IResult,
    nmea_content::{Parsable, Satellite},
};

/// GSV - Satellites in View
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gsv_satellites_in_view>
///
/// ```text
///         1 2 3 4 5 6 7     n
///         | | | | | | |     |
///  $--GSV,x,x,x,x,x,x,x,...,x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct GSV {
    /// Total number of GSV sentences to be transmitted in this group
    pub total_messages: u8,
    /// Sentence number of this GSV message within current group
    pub message_number: u8,
    /// Total number of satellites in view
    pub satellites_in_view: u8,
    /// Satellite information
    pub satellites: heapless::Vec<Satellite, 4>,
    #[cfg(feature = "nmea-v4-11")]
    /// Signal ID of the GNSS system used for the fix
    pub signal_id: Option<SignalId>,
}

impl<'a> Parsable<'a> for GSV {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, total_messages) = u8.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, message_number) = u8.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, satellites_in_view) = u8.parse(i)?;
        let (i, satellites) = many0(preceded(char(','), Satellite::parser)).parse(i)?;

        #[cfg(feature = "nmea-v4-11")]
        let (i, signal_id) = cond(
            !satellites.is_empty() || i.input_len() > 0,
            preceded(char(','), opt(hex_u32.map(|id| id as u8))),
        )
        .map(Option::flatten)
        .parse(i)?;

        Ok((
            i,
            Self {
                total_messages,
                message_number,
                satellites_in_view,
                satellites: satellites.into_iter().collect::<heapless::Vec<_, 4>>(),
                #[cfg(feature = "nmea-v4-11")]
                signal_id,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gsv_parsing() {
        let cases = [
            "1,1,00",
            "1,1,00,",
            "1,1,00,F",
            "1,1,01,05,45,120,38,",
            "1,1,04,01,60,150,45,02,30,090,30,03,70,270,50,04,10,010,20,",
            "1,1,01,05,45,120,,",
            "1,1,01,06,30,,40,",
            "1,1,01,07,,070,35,",
            "1,1,01,08,,,30,",
            "1,1,01,09,,180,,",
            "1,1,01,10,50,,,",
            "1,1,01,11,,,,",
            "1,1,03,01,60,150,45,02,30,,30,03,,270,,",
            "1,1,01,12,,,,",
            "1,1,01,05,45,,,",
        ];

        for &input in &cases {
            let result = GSV::parser(input);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }

        #[cfg(feature = "nmea-v4-11")]
        {
            let cases = [
                "1,1,01,05,45,120,38",
                "1,1,04,01,60,150,45,02,30,090,30,03,70,270,50,04,10,010,20",
                "1,1,01,05,45,120,",
                "1,1,01,06,30,,40",
                "1,1,01,07,,070,35",
                "1,1,01,08,,,30",
                "1,1,01,09,,180,",
                "1,1,01,10,50,,",
                "1,1,01,11,,,",
                "1,1,03,01,60,150,45,02,30,,30,03,,270,",
                "1,1,01,12,,,",
                "1,1,01,05,45,,",
            ];

            for &input in &cases {
                let result = GSV::parser(input);
                assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
            }
        }
    }
}
