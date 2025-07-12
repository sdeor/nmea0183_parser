use nom::{Parser, character::complete::char, number::complete::float};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    IResult,
    nmea_content::{Parsable, parse::opt_with_unit},
};

/// DBT - Depth Below Transducer
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dbt_depth_below_transducer>
///
/// ```text
///         1   2 3   4 5   6
///         |   | |   | |   |
///  $--DBT,x.x,f,x.x,M,x.x,F*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct DBT {
    /// Water depth in meters
    pub water_depth: Option<f32>,
}

impl<'a> Parsable<'a> for DBT {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, water_depth_feet) = opt_with_unit(float, 'f').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, water_depth_meters) = opt_with_unit(float, 'M').parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, water_depth_fathoms) = opt_with_unit(float, 'F').parse(i)?;

        let water_depth = water_depth_meters
            .or(water_depth_feet.map(|feet| feet * 0.3048))
            .or(water_depth_fathoms.map(|fathoms| fathoms * 1.8288));

        Ok((i, Self { water_depth }))
    }
}
