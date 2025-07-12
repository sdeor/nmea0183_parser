use nom::{Parser, character::complete::char, combinator::opt, number::complete::float};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v2-3")]
use crate::nmea_content::FaaMode;
#[cfg(feature = "nmea-v4-11")]
use crate::nmea_content::NavStatus;
use crate::{
    IResult,
    nmea_content::{
        Parsable, Status,
        parse::{date, latlon, magnetic_variation, time},
    },
};

/// RMC - Recommended Minimum Navigation Information
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_rmc_recommended_minimum_navigation_information>
///
/// ```text
///         1         2 3       4 5        6  7   8   9    10 11
///         |         | |       | |        |  |   |   |    |  |
///  $--RMC,hhmmss.ss,A,ddmm.mm,a,dddmm.mm,a,x.x,x.x,xxxx,x.x,a*hh<CR><LF>
/// ```
///
/// NMEA 2.3:
/// ```text
///         1         2 3       4 5        6  7   8   9    10 1112
///         |         | |       | |        |  |   |   |    |  | |
///  $--RMC,hhmmss.ss,A,ddmm.mm,a,dddmm.mm,a,x.x,x.x,xxxx,x.x,a,m*hh<CR><LF>
/// ```
///
/// NMEA 4.1:
/// ```text
///         1         2 3       4 5        6  7   8   9    10 111213
///         |         | |       | |        |  |   |   |    |  | | |
///  $--RMC,hhmmss.ss,A,ddmm.mm,a,dddmm.mm,a,x.x,x.x,xxxx,x.x,a,m,s*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct RMC {
    /// Fix time in UTC
    pub fix_time: Option<time::Time>,
    /// Fix date in UTC
    pub fix_date: Option<time::Date>,
    /// Status Mode Indicator
    pub status: Status,
    /// Latitude in degrees
    pub latitude: Option<f64>,
    /// Longitude in degrees
    pub longitude: Option<f64>,
    /// Speed over ground in knots
    pub speed_over_ground: Option<f32>,
    /// Course over ground in degrees
    pub course_over_ground: Option<f32>,
    /// Magnetic variation in degrees
    pub magnetic_variation: Option<f32>,
    #[cfg(feature = "nmea-v2-3")]
    /// FAA Mode Indicator
    pub faa_mode: Option<FaaMode>,
    #[cfg(feature = "nmea-v4-11")]
    /// Navigation status
    pub nav_status: Option<NavStatus>,
}

impl<'a> Parsable<'a> for RMC {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, fix_time) = opt(time).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, status) = Status::parser(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, (latitude, longitude)) = latlon.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, speed_over_ground) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, course_over_ground) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, fix_date) = opt(date).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, magnetic_variation) = magnetic_variation.parse(i)?;
        #[cfg(feature = "nmea-v2-3")]
        let (i, _) = char(',').parse(i)?;
        #[cfg(feature = "nmea-v2-3")]
        let (i, faa_mode) = opt(FaaMode::parser).parse(i)?;
        #[cfg(feature = "nmea-v4-11")]
        let (i, _) = char(',').parse(i)?;
        #[cfg(feature = "nmea-v4-11")]
        let (i, nav_status) = opt(NavStatus::parser).parse(i)?;

        Ok((
            i,
            Self {
                fix_time,
                fix_date,
                status,
                latitude,
                longitude,
                speed_over_ground,
                course_over_ground,
                magnetic_variation,
                #[cfg(feature = "nmea-v2-3")]
                faa_mode,
                #[cfg(feature = "nmea-v4-11")]
                nav_status,
            },
        ))
    }
}
