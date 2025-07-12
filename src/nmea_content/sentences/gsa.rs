use nom::{
    Parser,
    character::complete::{char, u8},
    combinator::opt,
    multi::fill,
    number::complete::float,
    sequence::preceded,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nmea-v4-11")]
use crate::nmea_content::SystemId;
use crate::{
    IResult,
    nmea_content::{FixMode, Parsable, SelectionMode},
};

/// GSA - GPS DOP and active satellites
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_gsa_gps_dop_and_active_satellites>
///
/// ```text
///         1 2 3                      15 16  17
///         | | |                       | |   |
///  $--GSA,a,a,x,x,x,x,x,x,x,x,x,x,x,x,x,x.x,x.x,*hh<CR><LF>
/// ```
///
/// NMEA 4.11:
/// ```text
///         1 2 3                      15 16  17  18
///         | | |                       | |   |   |
///  $--GSA,a,a,x,x,x,x,x,x,x,x,x,x,x,x,x,x.x,x.x,x.x*hh<CR><LF>
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug)]
pub struct GSA {
    /// Selection mode
    pub selection_mode: SelectionMode,
    /// Fix mode
    pub fix_mode: FixMode,
    /// PRN numbers of the satellites used in the fix, up to 12
    pub fix_sats_prn: heapless::Vec<u8, 12>,
    /// Position Dilution of Precision
    pub pdop: Option<f32>,
    /// Horizontal Dilution of Precision
    pub hdop: Option<f32>,
    /// Vertical Dilution of Precision
    pub vdop: Option<f32>,
    #[cfg(feature = "nmea-v4-11")]
    /// System ID of the GNSS system used for the fix
    pub system_id: Option<SystemId>,
}

impl<'a> Parsable<'a> for GSA {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, selection_mode) = SelectionMode::parser(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, fix_mode) = FixMode::parser(i)?;

        let mut fix_sats_prn = [None; 12];
        let (i, _) = fill(preceded(char(','), opt(u8)), &mut fix_sats_prn).parse(i)?;

        let (i, _) = char(',').parse(i)?;
        let (i, pdop) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, hdop) = opt(float).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, vdop) = opt(float).parse(i)?;

        #[cfg(feature = "nmea-v4-11")]
        let (i, _) = char(',').parse(i)?;
        #[cfg(feature = "nmea-v4-11")]
        let (i, system_id) = opt(SystemId::parser).parse(i)?;

        Ok((
            i,
            Self {
                selection_mode,
                fix_mode,
                fix_sats_prn: fix_sats_prn
                    .into_iter()
                    .flatten()
                    .collect::<heapless::Vec<_, 12>>(),
                pdop,
                hdop,
                vdop,
                #[cfg(feature = "nmea-v4-11")]
                system_id,
            },
        ))
    }
}
