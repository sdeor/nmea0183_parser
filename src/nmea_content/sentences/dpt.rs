use nom::{Parser, character::complete::char, combinator::opt, number::complete::float};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{IResult, nmea_content::Parsable};

/// DPT - Depth of Water
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_dpt_depth_of_water>
///
/// ```text
///         1   2
///         |   |
///  $--DPT,x.x,x.x*hh<CR><LF>
/// ```
///
/// NMEA 3.0:
/// ```text
///        1   2   3
///        |   |   |
/// $--DPT,x.x,x.x,x.x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct DPT {
    /// Water depth relative to transducer in meters
    pub water_depth: Option<f32>,
    /// Offset from transducer in meters,
    /// positive means distance from transducer to water line,
    /// negative means distance from transducer to keel
    pub offset_from_transducer: Option<f32>,
    #[cfg(feature = "nmea-v3-0")]
    /// Maximum range scale in used for the measurement in meters
    pub max_range_scale: Option<f32>,
}

impl<'a> Parsable<'a> for DPT {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, water_depth) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, offset_from_transducer) = opt(float).parse(i)?;

        #[cfg(feature = "nmea-v3-0")]
        let (i, _) = char(',').parse(i)?;
        #[cfg(feature = "nmea-v3-0")]
        let (i, max_range_scale) = opt(float).parse(i)?;

        Ok((
            i,
            DPT {
                water_depth,
                offset_from_transducer,
                #[cfg(feature = "nmea-v3-0")]
                max_range_scale,
            },
        ))
    }
}
