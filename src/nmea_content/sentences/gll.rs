use nom::{Parser, character::complete::char, combinator::opt};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v2-3")]
use crate::nmea_content::FaaMode;
use crate::{
    IResult,
    nmea_content::{
        Parsable, Status,
        parse::{latlon, time},
    },
};

/// GLL - Geographic Position - Latitude/Longitude
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gll_geographic_position_latitudelongitude>
///
/// ```text
///         1       2 3        4 5         6
///         |       | |        | |         |
///  $--GLL,ddmm.mm,a,dddmm.mm,a,hhmmss.ss,a*hh<CR><LF>
/// ```
///
/// NMEA 2.3:
/// ```text
///         1       2 3        4 5         6 7
///         |       | |        | |         | |
///  $--GLL,ddmm.mm,a,dddmm.mm,a,hhmmss.ss,a,m*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct GLL {
    /// Latitude in degrees
    pub latitude: Option<f64>,
    /// Longitude in degrees
    pub longitude: Option<f64>,
    /// Fix time in UTC
    pub fix_time: Option<time::Time>,
    /// Status Mode Indicator
    pub status: Status,
    #[cfg(feature = "nmea-v2-3")]
    /// FAA Mode Indicator
    pub faa_mode: Option<FaaMode>,
}

impl<'a> Parsable<'a> for GLL {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, (latitude, longitude)) = latlon.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, fix_time) = opt(time).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, status) = Status::parser(i)?;

        #[cfg(feature = "nmea-v2-3")]
        let (i, _) = char(',').parse(i)?;
        #[cfg(feature = "nmea-v2-3")]
        let (i, faa_mode) = opt(FaaMode::parser).parse(i)?;

        Ok((
            i,
            Self {
                latitude,
                longitude,
                fix_time,
                status,
                #[cfg(feature = "nmea-v2-3")]
                faa_mode,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gll_parsing() {
        let cases = [",", ",A", ",Z"];

        for &input in &cases {
            let i = format!("4404.14012,N,12118.85993,W,001037.00,A{}", input);

            let result = GLL::parser(&i);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }

        #[cfg(feature = "nmea-v2-3")]
        {
            let cases = ["", "Z"];

            for &input in &cases {
                let i = format!("4404.14012,N,12118.85993,W,001037.00,A{}", input);

                let result = GLL::parser(&i);
                assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
            }
        }
    }
}
