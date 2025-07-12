use std::time::Duration;

use nom::{
    Parser,
    character::complete::{char, u8, u16},
    combinator::opt,
    number::complete::float,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    IResult,
    nmea_content::{
        Parsable, Quality,
        parse::{latlon, opt_with_unit, time},
    },
};

/// GGA - Global Positioning System Fix Data
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gga_global_positioning_system_fix_data>
///
/// ```text
///                                                      11
///         1         2       3 4        5 6 7  8   9  10 |  12 13  14
///         |         |       | |        | | |  |   |   | |   | |   |
///  $--GGA,hhmmss.ss,ddmm.mm,a,dddmm.mm,a,x,xx,x.x,x.x,M,x.x,M,x.x,xxxx*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct GGA {
    /// Fix time in UTC
    pub fix_time: Option<time::Time>,
    /// Latitude in degrees
    pub latitude: Option<f64>,
    /// Longitude in degrees
    pub longitude: Option<f64>,
    /// GPS Quality Indicator
    pub fix_quality: Quality,
    /// Number of satellites in use
    pub satellite_count: Option<u8>,
    /// Horizontal Dilution of Precision
    pub hdop: Option<f32>,
    /// Altitude above/below mean sea level (geoid) in meters
    pub altitude: Option<f32>,
    /// Geoidal separation in meters, the difference between the WGS-84 earth ellipsoid and mean sea level (geoid),
    /// negative values indicate that the geoid is below the ellipsoid
    pub geoidal_separation: Option<f32>,
    /// Age of Differential GPS data in seconds, time since last SC104 type 1 or 9 update, null field when DGPS is not used
    pub age_of_dgps: Option<Duration>,
    /// Differential reference station ID
    pub ref_station_id: Option<u16>,
}

impl<'a> Parsable<'a> for GGA {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, time) = opt(time).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, (latitude, longitude)) = latlon.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, fix_quality) = Quality::parser(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, satellite_count) = opt(u8).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, hdop) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, altitude) = opt_with_unit(float, 'M').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, geoidal_separation) = opt_with_unit(float, 'M').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, age_of_dgps) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, ref_station_id) = opt(u16).parse(i)?;

        Ok((
            i,
            Self {
                fix_time: time,
                latitude,
                longitude,
                fix_quality,
                satellite_count,
                hdop,
                altitude,
                geoidal_separation,
                age_of_dgps: age_of_dgps.map(|sec| Duration::from_millis((sec * 1000.0) as u64)),
                ref_station_id,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gga_parsing() {
        let cases = [",,", ",42.0,", ",,69", ",42.0,69"];

        for &input in &cases {
            let i = format!(
                "001043.00,4404.14036,N,12118.85961,W,1,12,0.98,1113.0,M,-21.3,M{}",
                input
            );

            let result = GGA::parser(&i);
            assert!(result.is_ok(), "Failed: {input:?}\n\t{result:?}");
        }

        let cases = ["", ",", ",42.0", "42.0"];

        for &input in &cases {
            let i = format!(
                "001043.00,4404.14036,N,12118.85961,W,1,12,0.98,1113.0,M,-21.3,M{}",
                input
            );

            let result = GGA::parser(&i);
            assert!(result.is_err(), "Failed: {input:?}\n\t{result:?}");
        }
    }
}
