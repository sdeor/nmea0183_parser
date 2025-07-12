#[cfg(feature = "nmea-v2-3")]
use nom::combinator::opt;
use nom::{Parser, character::complete::char, number::complete::float};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v2-3")]
use crate::nmea_content::FaaMode;
use crate::{
    IResult,
    nmea_content::{Parsable, parse::opt_with_unit},
};

/// VTG - Track made good and Ground speed
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_vtg_track_made_good_and_ground_speed>
///
/// ```text
///          1  2  3  4  5  6  7  8
///          |  |  |  |  |  |  |  |
///  $--VTG,x.x,T,x.x,M,x.x,N,x.x,K*hh<CR><LF>
/// ```
///
/// NMEA 2.3:
///
/// ```text
///          1  2  3  4  5  6  7  8 9
///          |  |  |  |  |  |  |  | |
///  $--VTG,x.x,T,x.x,M,x.x,N,x.x,K,m*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct VTG {
    /// Course over ground in degrees true
    pub course_over_ground_true: Option<f32>,
    /// Course over ground in degrees magnetic
    pub course_over_ground_magnetic: Option<f32>,
    /// Speed over ground in knots
    pub speed_over_ground: Option<f32>,
    #[cfg(feature = "nmea-v2-3")]
    /// FAA Mode Indicator
    pub faa_mode: Option<FaaMode>,
}

impl<'a> Parsable<'a> for VTG {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, course_over_ground_true) = opt_with_unit(float, 'T').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, course_over_ground_magnetic) = opt_with_unit(float, 'M').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, speed_over_ground_knots) = opt_with_unit(float, 'N').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, speed_over_ground_kph) = opt_with_unit(float, 'K').parse(i)?;

        #[cfg(feature = "nmea-v2-3")]
        let (i, _) = char(',').parse(i)?;
        #[cfg(feature = "nmea-v2-3")]
        let (i, faa_mode) = opt(FaaMode::parser).parse(i)?;

        let speed_over_ground =
            speed_over_ground_knots.or(speed_over_ground_kph.map(|kph| kph / 1.852));

        Ok((
            i,
            Self {
                course_over_ground_true,
                course_over_ground_magnetic,
                speed_over_ground,
                #[cfg(feature = "nmea-v2-3")]
                faa_mode,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtg_parsing() {
        let cases = [
            ",T,,M,,N,,K,N",
            "360.0,T,348.7,M,000.0,N,000.0,K,N",
            "360.0,T,348.7,M,000.0,N,,,N",
            "360.0,T,348.7,M,,,000.0,K,N",
            "360.0,T,348.7,M,,,,,N",
        ];

        for &input in &cases {
            let result = VTG::parser(input);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
