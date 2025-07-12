mod dbt;
mod dpt;
mod gga;
mod gll;
mod gsa;
mod gsv;
mod rmc;
mod vtg;
mod zda;

pub use dbt::DBT;
pub use dpt::DPT;
pub use gga::GGA;
pub use gll::GLL;
pub use gsa::GSA;
pub use gsv::GSV;
pub use rmc::RMC;
pub use vtg::VTG;
pub use zda::ZDA;

use nom::{
    Parser,
    bytes::complete::take,
    character::complete::{char, u8, u16},
    combinator::opt,
    error::ErrorKind,
};

use crate::{Error, IResult, consumed};

/// A trait for types that can be parsed from a string input.
///
/// This trait defines a single method `parser` that takes a string slice
/// and returns an `IResult` containing the remaining input and the parsed value.
///
/// This trait is implemented by all strongly-typed NMEA sentence structs
/// and the `NmeaSentence` enum, allowing them to be parsed using the
/// `nmea0183` framing parser.
pub trait Parsable<'a>: Sized {
    /// Parses the input and returns a result.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to parse into `Self`.
    ///
    /// # Returns
    ///
    /// Returns an [`IResult`] containing:
    /// - On success: A tuple of `(remaining_input, parsed_value)`, where `remaining_input`
    ///   is the unparsed portion of the input and `parsed_value` is the successfully parsed
    ///   instance of `Self`.
    /// - On failure: An [`Error`] indicating the parsing error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nmea0183_parser::nmea_content::{NmeaSentence, Parsable};
    ///
    /// // Parse complete sentence content (including talker ID and sentence type)
    /// let content = "GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,";
    /// let result = NmeaSentence::parser(content);
    /// assert!(result.is_ok());
    /// ```
    fn parser(input: &'a str) -> IResult<&'a str, Self>;
}

/// A unified enum representing all supported NMEA 0183 sentence types.
///
/// This enum acts as a comprehensive abstraction over all built-in NMEA sentence
/// types supported by this parser. Each variant wraps the corresponding strongly-typed
/// struct, providing type-safe access to parsed sentence data.
///
/// ## Design Philosophy
///
/// `NmeaSentence` serves as the built-in content parser that works seamlessly with
/// the [`nmea0183`](crate::nmea0183) framing parser. While the framing parser handles
/// the outer NMEA structure (`$`, checksum, CRLF validation), [`NmeaSentence::parser`] focuses
/// on parsing and validating the inner sentence content.
///
/// This design allows you to:
/// - Easily parse any supported NMEA sentence type using a single parser
/// - Access strongly-typed data for each sentence variant
/// - Extend with custom parsers for additional sentence types if needed
///
/// The parser performs several validations:
/// - Checks the sentence type and content format.
/// - Validates each individual field to ensure all required fields are present and correctly formatted.
/// - Returns an error if any field is missing or malformed, indicating the specific issue.
///   If a field is optional and not present, it will not trigger an error.
/// - Ensures the sentence is fully consumed, with no remaining unparsed content after the last field.
///   If there is unexpected trailing data, an error is returned.
///
/// ## Example Usage
///
/// ```rust
/// use nmea0183_parser::nmea_content::{NmeaSentence, Parsable};
///
/// let result = NmeaSentence::parser("GPZDA,123456.78,29,02,2024,03,00");
/// assert!(result.is_ok());
///
/// let sentence = result.unwrap().1;
/// match sentence {
///     NmeaSentence::ZDA(zda) => {
///         assert!(zda.time.is_some());
///         assert!(zda.date.is_some());
///         assert!(zda.utc_offset.is_some());
///     }
///     _ => println!("Other NMEA sentence parsed"),
/// }
/// ```
///
/// ## Usage with Framing Parser
///
/// ```rust
/// use nmea0183_parser::{
///     ChecksumMode, LineEndingMode, nmea0183,
///     nmea_content::{NmeaSentence, Parsable}
/// };
/// use nom::Parser;
///
/// // Create a complete NMEA parser
/// let mut parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(NmeaSentence::parser);
///
/// // Parse a complete NMEA sentence
/// let input = "$GPGSV,3,2,12,01,40,083,45*44\r\n";
/// let result = parser.parse(input);
///
/// match result {
///     Ok((remaining, sentence)) => {
///         match sentence {
///             NmeaSentence::GGA(gga) => {
///                 println!("GPS position: {:?}, {:?}", gga.latitude, gga.longitude);
///                 println!("Fix quality: {:?}", gga.fix_quality);
///                 println!("Satellites: {:?}", gga.satellite_count);
///             }
///             NmeaSentence::RMC(rmc) => {
///                 println!("Speed: {:?} knots", rmc.speed_over_ground);
///                 println!("Course: {:?}°", rmc.course_over_ground);
///             }
///             NmeaSentence::GSV(gsv) => {
///                 println!("Satellites in view: {:?}", gsv.satellites);
///             }
///             _ => println!("Other sentence type parsed"),
///         }
///     }
///     Err(e) => println!("Parse error: {:?}", e),
/// }
/// ```
///
/// ## Supported Sentence Types
///
/// | Variant      | Sentence Type                                           | Description                      |
/// |--------------|---------------------------------------------------------|----------------------------------|
/// | DBT([`DBT`]) | Depth Below Transducer                                  | Water depth measurements         |
/// | DPT([`DPT`]) | Depth of Water                                          | Water depth with offset          |
/// | GGA([`GGA`]) | Global Positioning System Fix Data                      | GPS position and fix quality     |
/// | GLL([`GLL`]) | Geographic Position - Latitude/Longitude                | Latitude/longitude with time     |
/// | GSA([`GSA`]) | GPS DOP and active satellites                           | Satellite constellation info     |
/// | GSV([`GSV`]) | Satellites in View                                      | Individual satellite details     |
/// | RMC([`RMC`]) | Recommended Minimum Navigation Information              | Essential navigation data        |
/// | VTG([`VTG`]) | Track made good and Ground speed                        | Velocity information             |
/// | ZDA([`ZDA`]) | Time & Date - UTC, day, month, year and local time zone | UTC time and date with time zone |
///
/// ## NMEA Version Support
///
/// Different NMEA versions may include additional fields. Enable appropriate feature flags:
/// - `nmea-content`: Basic NMEA parsing (pre-2.3)
/// - `nmea-v2-3`: NMEA 2.3 support  
/// - `nmea-v3-0`: NMEA 3.0 support (includes v2.3)
/// - `nmea-v4-11`: NMEA 4.11 support (includes v3.0)
///
/// ## Error Handling
///
/// The parser will return an error for:
/// - Unrecognized sentence types (not in the supported list above)
/// - Malformed sentence content that doesn't match the expected format
/// - Invalid field values (non-numeric where numbers expected, etc.)
///
/// ```rust
/// use nmea0183_parser::nmea_content::{NmeaSentence, Parsable};
///
/// // This will fail - unrecognized sentence type
/// let result = NmeaSentence::parser("GPUNK,some,data,here");
/// assert!(result.is_err());
///
/// // This will fail - malformed GGA sentence
/// let result = NmeaSentence::parser("GPGGA,invalid,data");
/// assert!(result.is_err());
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub enum NmeaSentence {
    /// Depth Below Transducer
    DBT(DBT),
    /// Depth of Water
    DPT(DPT),
    /// Global Positioning System Fix Data
    GGA(GGA),
    /// Geographic Position - Latitude/Longitude
    GLL(GLL),
    /// GPS DOP and active satellites
    GSA(GSA),
    /// Satellites in View
    GSV(GSV),
    /// Recommended Minimum Navigation Information
    RMC(RMC),
    /// Track made good and Ground speed
    VTG(VTG),
    /// Time & Date - UTC, day, month, year and local time zone
    ZDA(ZDA),
}

impl<'a> Parsable<'a> for NmeaSentence {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let msg = i;

        // TODO: Handle talker ID and sentence type parsing
        let (i, _talker_id) = take(2u8).parse(i)?;
        let (i, sentence_type) = take(3u8).parse(i)?;
        let (i, _) = char(',').parse(i)?;

        let (i, sentence) = match sentence_type {
            "DBT" => DBT::parser.map(Self::DBT).parse(i)?,
            "DPT" => DPT::parser.map(Self::DPT).parse(i)?,
            "GGA" => GGA::parser.map(Self::GGA).parse(i)?,
            "GLL" => GLL::parser.map(Self::GLL).parse(i)?,
            "GSA" => GSA::parser.map(Self::GSA).parse(i)?,
            "GSV" => GSV::parser.map(Self::GSV).parse(i)?,
            "RMC" => RMC::parser.map(Self::RMC).parse(i)?,
            "VTG" => VTG::parser.map(Self::VTG).parse(i)?,
            "ZDA" => ZDA::parser.map(Self::ZDA).parse(i)?,
            _ => return Err(nom::Err::Error(Error::UnrecognizedMessage(msg))),
        };

        let (i, _) = consumed(take(0u8), ErrorKind::Eof).parse(i)?;
        Ok((i, sentence))
    }
}

macro_rules! parsable_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $char:literal => $variant:ident
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, PartialEq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl<'a> Parsable<'a> for $name {
            fn parser(i: &'a str) -> IResult<&'a str, Self> {
                nom::branch::alt(($(
                    #[allow(unused_doc_comments)]
                    $(#[$variant_meta])*
                    nom::character::complete::char($char).map(|_| Self::$variant),
                )*)).parse(i)
            }
        }
    };
}

parsable_enum! {
    /// Status Mode Indicator
    pub enum Status {
        /// A - Valid
        'A' => Valid,
        /// V - Invalid
        'V' => Invalid,
    }
}

#[cfg(feature = "nmea-v2-3")]
parsable_enum! {
    /// FAA Mode Indicator
    ///
    /// <https://gpsd.gitlab.io/gpsd/NMEA.html#_sentence_mixes_and_nmea_variations>
    pub enum FaaMode {
        /// A - Autonomous mode
        'A' => Autonomous,
        /// C - Quectel Querk, "Caution"
        'C' => Caution,
        /// D - Differential Mode
        'D' => Differential,
        /// E - Estimated (dead-reckoning) mode
        'E' => Estimated,
        /// F - RTK Float mode
        'F' => FloatRtk,
        /// M - Manual Input Mode
        'M' => Manual,
        /// N - Data Not Valid
        'N' => DataNotValid,
        #[cfg(feature = "nmea-v4-11")]
        /// P - Precise
        'P' => Precise,
        /// R - RTK Integer mode
        'R' => FixedRtk,
        /// S - Simulated Mode
        'S' => Simulator,
        /// U - Quectel Querk, "Unsafe"
        'U' => Unsafe,
    }
}

#[cfg(feature = "nmea-v4-11")]
parsable_enum! {
    /// Navigation Status
    pub enum NavStatus {
        /// A - Autonomous mode
        'A' => Autonomous,
        /// D - Differential Mode
        'D' => Differential,
        /// E - Estimated (dead-reckoning) mode
        'E' => Estimated,
        /// M - Manual Input Mode
        'M' => Manual,
        /// N - Not Valid
        'N' => NotValid,
        /// S - Simulated Mode
        'S' => Simulator,
        /// V - Valid
        'V' => Valid,
    }
}

parsable_enum! {
    /// Quality of the GPS fix
    pub enum Quality {
        /// 0 - Fix not available
        '0' => NoFix,
        /// 1 - GPS fix
        '1' => GPSFix,
        /// 2 - Differential GPS fix
        '2' => DGPSFix,
        #[cfg(feature = "nmea-v2-3")]
        /// 3 - PPS fix
        '3' => PPSFix,
        #[cfg(feature = "nmea-v2-3")]
        /// 4 - Real Time Kinematic
        '4' => RTK,
        #[cfg(feature = "nmea-v2-3")]
        /// 5 - Float RTK
        '5' => FloatRTK,
        #[cfg(feature = "nmea-v2-3")]
        /// 6 - estimated (dead reckoning)
        '6' => Estimated,
        #[cfg(feature = "nmea-v2-3")]
        /// 7 - Manual input mode
        '7' => Manual,
        #[cfg(feature = "nmea-v2-3")]
        /// 8 - Simulation mode
        '8' => Simulation,
    }
}

parsable_enum! {
    /// Selection Mode
    pub enum SelectionMode {
        /// A - Automatic, 2D/3D
        'A' => Automatic,
        /// M - Manual, forced to operate in 2D or 3D
        'M' => Manual,
    }
}

parsable_enum! {
    /// Fix Mode
    pub enum FixMode {
        /// 1 - No fix
        '1' => NoFix,
        /// 2 - 2D Fix
        '2' => Fix2D,
        /// 3 - 3D Fix
        '3' => Fix3D,
    }
}

#[cfg(feature = "nmea-v4-11")]
parsable_enum! {
    /// NMEA 4.11 System ID
    ///
    /// <https://gpsd.gitlab.io/gpsd/NMEA.html#_nmea_4_11_system_id_and_signal_id>
    pub enum SystemId {
        /// 1 - GPS (GP)
        '1' => Gps,
        /// 2 - GLONASS (GL)
        '2' => Glonass,
        /// 3 - Galileo (GA)
        '3' => Galileo,
        /// 4 - BeiDou (GB/BD)
        '4' => Beidou,
        /// 5 - QZSS (GQ)
        '5' => Qzss,
        /// 6 - NavIC (GI)
        '6' => Navic,
    }
}

/// NMEA 4.11 Signal ID
///
/// <https://gpsd.gitlab.io/gpsd/NMEA.html#_nmea_4_11_system_id_and_signal_id>
#[cfg(feature = "nmea-v4-11")]
pub type SignalId = u8;
/*
 * // TODO:
 * pub enum SignalId {
 *     Gps(GpsSignalId),
 *     Glonass(GlonassSignalId),
 *     Galileo(GalileoSignalId),
 *     Beidou(BeidouSignalId),
 *     Qzss(QzssSignalId),
 *     Navic(NavicSignalId),
 *     Unknown(u8),
 * }
 */

/// Satellite information used in [`GSV`] sentences
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq)]
pub struct Satellite {
    pub prn: u8,
    pub elevation: Option<u8>,
    pub azimuth: Option<u16>,
    pub snr: Option<u8>,
}

impl<'a> Parsable<'a> for Satellite {
    fn parser(i: &'a str) -> IResult<&'a str, Self> {
        let (i, prn) = u8.parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, elevation) = opt(u8).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, azimuth) = opt(u16).parse(i)?;
        let (i, _) = char(',').parse(i)?;
        let (i, snr) = opt(u8).parse(i)?;

        Ok((
            i,
            Self {
                prn,
                elevation,
                azimuth,
                snr,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_status() {
        assert_eq!(
            (Status::parser("A") as IResult<_, _>).unwrap(),
            ("", Status::Valid)
        );
        assert_eq!(
            (Status::parser("V") as IResult<_, _>).unwrap(),
            ("", Status::Invalid)
        );
        assert!((Status::parser("K") as IResult<_, _>).is_err());
    }

    #[test]
    fn test_faa_mode() {
        #[cfg(feature = "nmea-v2-3")]
        {
            assert_eq!(
                (FaaMode::parser("A") as IResult<_, _>).unwrap(),
                ("", FaaMode::Autonomous)
            );
            assert_eq!(
                (FaaMode::parser("C") as IResult<_, _>).unwrap(),
                ("", FaaMode::Caution)
            );
            assert_eq!(
                (FaaMode::parser("D") as IResult<_, _>).unwrap(),
                ("", FaaMode::Differential)
            );
            assert_eq!(
                (FaaMode::parser("E") as IResult<_, _>).unwrap(),
                ("", FaaMode::Estimated)
            );
            assert_eq!(
                (FaaMode::parser("F") as IResult<_, _>).unwrap(),
                ("", FaaMode::FloatRtk)
            );
            assert_eq!(
                (FaaMode::parser("M") as IResult<_, _>).unwrap(),
                ("", FaaMode::Manual)
            );
            assert_eq!(
                (FaaMode::parser("N") as IResult<_, _>).unwrap(),
                ("", FaaMode::DataNotValid)
            );
            #[cfg(feature = "nmea-v4-11")]
            {
                assert_eq!(
                    (FaaMode::parser("P") as IResult<_, _>).unwrap(),
                    ("", FaaMode::Precise)
                );
            }
            #[cfg(not(feature = "nmea-v4-11"))]
            {
                assert!((FaaMode::parser("P") as IResult<_, _>).is_err());
            }
            assert_eq!(
                (FaaMode::parser("R") as IResult<_, _>).unwrap(),
                ("", FaaMode::FixedRtk)
            );
            assert_eq!(
                (FaaMode::parser("S") as IResult<_, _>).unwrap(),
                ("", FaaMode::Simulator)
            );
            assert_eq!(
                (FaaMode::parser("U") as IResult<_, _>).unwrap(),
                ("", FaaMode::Unsafe)
            );
            assert!((FaaMode::parser("X") as IResult<_, _>).is_err());
        }
    }

    #[cfg(feature = "nmea-v4-11")]
    #[test]
    fn test_gsa_quality() {
        assert_eq!(
            (Quality::parser("0") as IResult<_, _>).unwrap(),
            ("", Quality::NoFix)
        );
        assert_eq!(
            (Quality::parser("1") as IResult<_, _>).unwrap(),
            ("", Quality::GPSFix)
        );
        assert_eq!(
            (Quality::parser("2") as IResult<_, _>).unwrap(),
            ("", Quality::DGPSFix)
        );
        assert_eq!(
            (Quality::parser("3") as IResult<_, _>).unwrap(),
            ("", Quality::PPSFix)
        );
        assert_eq!(
            (Quality::parser("4") as IResult<_, _>).unwrap(),
            ("", Quality::RTK)
        );
        assert_eq!(
            (Quality::parser("5") as IResult<_, _>).unwrap(),
            ("", Quality::FloatRTK)
        );
        assert_eq!(
            (Quality::parser("6") as IResult<_, _>).unwrap(),
            ("", Quality::Estimated)
        );
        assert_eq!(
            (Quality::parser("7") as IResult<_, _>).unwrap(),
            ("", Quality::Manual)
        );
        assert_eq!(
            (Quality::parser("8") as IResult<_, _>).unwrap(),
            ("", Quality::Simulation)
        );
        assert!((Quality::parser("9") as IResult<_, _>).is_err());
    }

    #[test]
    fn test_ssa_mode1() {
        assert_eq!(
            (SelectionMode::parser("A") as IResult<_, _>).unwrap(),
            ("", SelectionMode::Automatic)
        );
        assert_eq!(
            (SelectionMode::parser("M") as IResult<_, _>).unwrap(),
            ("", SelectionMode::Manual)
        );
        assert!((SelectionMode::parser("X") as IResult<_, _>).is_err());
    }

    #[test]
    fn test_ssa_mode2() {
        assert_eq!(
            (FixMode::parser("1") as IResult<_, _>).unwrap(),
            ("", FixMode::NoFix)
        );
        assert_eq!(
            (FixMode::parser("2") as IResult<_, _>).unwrap(),
            ("", FixMode::Fix2D)
        );
        assert_eq!(
            (FixMode::parser("3") as IResult<_, _>).unwrap(),
            ("", FixMode::Fix3D)
        );
        assert!((FixMode::parser("4") as IResult<_, _>).is_err());
    }

    #[cfg(feature = "nmea-v4-11")]
    #[test]
    fn test_system_id() {
        assert_eq!(
            (SystemId::parser("1") as IResult<_, _>).unwrap(),
            ("", SystemId::Gps)
        );
        assert_eq!(
            (SystemId::parser("2") as IResult<_, _>).unwrap(),
            ("", SystemId::Glonass)
        );
        assert_eq!(
            (SystemId::parser("3") as IResult<_, _>).unwrap(),
            ("", SystemId::Galileo)
        );
        assert_eq!(
            (SystemId::parser("4") as IResult<_, _>).unwrap(),
            ("", SystemId::Beidou)
        );
        assert_eq!(
            (SystemId::parser("5") as IResult<_, _>).unwrap(),
            ("", SystemId::Qzss)
        );
        assert_eq!(
            (SystemId::parser("6") as IResult<_, _>).unwrap(),
            ("", SystemId::Navic)
        );
        assert!((SystemId::parser("7") as IResult<_, _>).is_err());
    }

    #[cfg(feature = "nmea-v2-3")]
    #[cfg(not(feature = "nmea-v3-0"))]
    #[test]
    fn test_nmea_parser() {
        let valid = [
            "GPDBT,12.34,f,3.76,M,2.05,F",
            "GPDBT,0.00,f,0.00,M,0.00,F",
            "GPDBT,50.00,f,15.24,M,8.20,F",
            "GPDBT,1.50,f,0.46,M,0.25,F",
            "GPDBT,100.00,f,30.48,M,16.40,F",
            "GPDPT,10.5,0.2",
            "GPDPT,0.0,",
            "GPDPT,50.0,1.0",
            "GPDPT,1.2,",
            "GPDPT,100.0,0.5",
            "GPGGA,092725.00,4717.113,N,00833.915,E,1,08,1.0,499.7,M,48.0,M,,",
            "GPGGA,235959,0000.000,N,00000.000,W,1,00,99.9,0.0,M,0.0,M,,",
            "GPGGA,000000,9000.000,S,18000.000,W,1,12,0.5,100.0,M,10.0,M,,",
            "GPGGA,010203,1234.567,N,01234.567,E,2,05,2.0,20.0,M,5.0,M,,",
            "GPGLL,4916.45,N,12311.12,W,225444,A,A",
            "GPGLL,0000.00,N,00000.00,E,000000,V,N",
            "GPGLL,9000.00,S,18000.00,W,235959,A,D",
            "GPGLL,3456.78,N,07890.12,E,123456,A,A",
            "GPGLL,1234.56,S,01234.56,W,010203,V,N",
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0,2.0",
            "GPGSA,M,1,,,,,,,,,,,,,99.9,99.9,99.9",
            "GPGSA,A,2,10,20,30,,,,,,,,,,2.0,1.5,2.5",
            "GPGSA,A,3,01,03,05,07,09,11,13,15,17,19,21,23,0.5,0.3,0.7",
            "GPGSA,M,2,02,04,06,,,,,,,,,,3.0,2.5,3.5",
            "GPGSV,3,1,11,01,65,123,45,02,40,210,30,03,70,300,35,04,20,090,20",
            "GPGSV,3,2,11,05,50,045,25,06,30,180,15,07,80,270,40,08,10,315,10",
            "GPGSV,3,3,11,09,40,060,22,10,60,150,33,11,75,240,38",
            "GPGSV,1,1,01,01,90,100,50",
            "GPGSV,2,1,04,01,45,120,25,02,30,200,18,03,60,090,30,04,70,310,35",
            "GPGSV,2,2,04,05,20,150,10,06,50,070,28,07,85,240,42",
            "GPRMC,123519,A,4807.038,N,01131.000,E,0.20,0.83,230394,004.2,W,A",
            "GPRMC,092725.00,A,4717.113,N,00833.915,E,0.0,0.0,010190,,,A",
            "GPRMC,235959,V,0000.000,N,00000.000,W,10.5,180.0,311299,,,N",
            "GPRMC,000000,A,9000.000,S,18000.000,W,100.0,0.0,010100,,,A",
            "GPRMC,010203,A,1234.567,N,01234.567,E,5.0,270.0,050607,,,A",
            "GPVTG,054.7,T,034.4,M,005.5,N,010.2,K,A",
            "GPVTG,000.0,T,000.0,M,000.0,N,000.0,K,N",
            "GPVTG,359.9,T,330.0,M,010.0,N,018.5,K,A",
            "GPVTG,090.0,T,060.0,M,001.0,N,001.8,K,A",
            "GPVTG,180.0,T,150.0,M,020.0,N,037.0,K,A",
            "GPZDA,123519,04,07,2025,,",
            "GPZDA,092725.00,01,01,1990,,",
            "GPZDA,235959,31,12,1999,,",
            "GPZDA,000000,01,01,2000,,",
            "GPZDA,010203,05,06,2007,,",
            "GPZDA,100000,15,03,2024,+01,30",
            "GPZDA,153045,20,11,2023,-08,00",
            "GPZDA,204510,02,09,2022,+03,00",
            "GPZDA,051520,10,04,2021,+07,00",
            "GPZDA,220000,25,12,2020,-11,00",
        ];

        for sentence in valid {
            let result = NmeaSentence::parser(sentence);
            assert!(
                result.is_ok(),
                "Failed to parse valid sentence: {}, error: {:?}",
                sentence,
                result.unwrap_err()
            );
        }

        let invalid = [
            "GPDBT,12.34,x,3.76,M,2.05,F",   // Invalid unit 'x'
            "GPDBT,1.0,f,a,M,2.0,F",         // Non-numeric depth
            "GPDBT,10.0,f,5.0,M",            // Missing last field
            "GPDBT,TooDeep,f,1.0,M,2.0,F",   // Non-numeric depth
            "GPDBT,1.0,f,2.0,M,3.0,F,extra", // Extra field
            "GPDPT,10.5,0.2,x",              // Invalid character
            "GPDPT,10.5,0.2,1,2",            // Too many fields
            "GPDPT,abc,,",                   // Non-numeric depth
            "GPDPT,,0.5,",                   // Missing depth
            "GPDPT,10.0",                    // Too few fields
            "GPGGA,123519,4807.038,N,01131.000,X,1,08,0.9,545.4,M,46.9,M,,", // Invalid East/West indicator
            "GPGGA,123519,4807.038,N,01131.000,E,9,08,0.9,545.4,M,46.9,M,,", // Invalid Fix Quality
            "GPGGA,123519,4807.038,N,01131.000,E,1,A8,0.9,545.4,M,46.9,M,,", // Invalid satellites (non-numeric)
            "GPGLL,4916.45,N,12311.12,W,225444,A,X", // Invalid mode indicator
            "GPGLL,4916.45,N,12311.12,W,225444,A",   // Missing mode indicator
            "GPGLL,abc,N,12311.12,W,225444,A,A",     // Non-numeric latitude
            "GPGLL,4916.45,N,def,W,225444,A,A",      // Non-numeric longitude
            "GPGLL,4916.45,N,12311.12,W,25444,A,A",  // Invalid time format (too short)
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,A,1.0,2.0", // Non-numeric PDOP
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,B,2.0", // Non-numeric HDOP
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0,C", // Non-numeric VDOP
            "GPGSA,A,4,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0,2.0", // Invalid fix mode (4 is not 1, 2, or 3)
            "GPGSA,A,3,01,02,03,04,05,06,07,08,09,10,11,12,1.5,1.0",     // Missing VDOP
            "GPGSV,3,1,11,01,65,123,45,02,40,210,30,03,70,300,35,04,20,090,XX", // Non-numeric SNR
            "GPGSV,3,1,11,01,65,123,45,02,40,210,30,03,70,300,35,04,20,090", // Missing SNR
            "GPRMC,123519,A,4807.038,N,01131.000,E,0.20,0.83,230394,004.2,W,X", // Invalid mode (X not one of ACDEFMNRSU)
            "GPRMC,123519,A,4807.038,N,01131.000,E,0.20,0.83,230394,004.2,W",   // Missing mode
            "GPRMC,123519,A,4807.038,N,01131.000,E,abc,0.83,230394,004.2,W,A",  // Non-numeric speed
            "GPVTG,054.7,T,034.4,M,005.5,N,010.2,K,X", // Invalid mode indicator
            "GPVTG,054.7,T,034.4,M,005.5,N,010.2,K",   // Missing mode indicator
            "GPVTG,abc,T,034.4,M,005.5,N,010.2,K,A",   // Non-numeric true track
            "GPVTG,054.7,T,def,M,005.5,N,010.2,K,A",   // Non-numeric magnetic track
            "GPVTG,054.7,T,034.4,M,ghi,N,010.2,K,A",   // Non-numeric speed over ground (knots)
            "GPZDA,123519,04,07,2025,XX,",             // Non-numeric local time zone hours
            "GPZDA,123519,04,07,2025,,XX",             // Non-numeric local time zone minutes
            "GPZDA,123519,32,07,2025,,",               // Invalid day (32)
            "GPZDA,123519,04,13,2025,,",               // Invalid month (13)
            "GPZDA,123519,04,07,2025",                 // Missing local time zone fields
            "GPZDA,abc,04,07,2025,,",                  // Non-numeric time
            "GPZDA,123519,0,07,2025,,",                // Day 0
            "GPZDA,123519,04,0,2025,,",                // Month 0
            "GPZDA,123519,04,07,2025,01,ab",           // Non-numeric local time zone minutes
            "GPZDA,123519,04,07,2025,ab,00",           // Non-numeric local time zone hours
        ];

        for sentence in invalid {
            let result = NmeaSentence::parser(sentence);
            assert!(
                result.is_err(),
                "Parsed invalid sentence as valid: {}, sentence: {:?}",
                sentence,
                result.unwrap(),
            );
        }
    }
}
