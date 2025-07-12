//! # A Flexible NMEA 0183 Parser for Rust
//!
//! **A zero-allocation NMEA 0183 parser that separates message framing from content parsing,
//! giving you full control over how the sentence content is parsed and interpreted.**
//!
//! This crate provides a generic and configurable parser for NMEA 0183-style messages with the format:
//! `$HHH,D1,D2,...,Dn*CC\r\n`
//!
//! ## Key Design Philosophy
//!
//! Unlike traditional NMEA parsers that tightly couple format and content parsing,
//! this crate provides tools to parse and validate NMEA 0183-style sentences by separating concerns:
//!
//! 1. **[`nmea0183`] framing parser** - A generic factory function that handles the outer structure:
//!    - ASCII-only validation
//!    - Start delimiter (`$`)
//!    - Optional checksum validation (`*CC`)
//!    - Optional CRLF endings (`\r\n`)
//!
//! 2. **Content parser** - Either user-defined or built-in parsers that handle the inner data
//!
//! This separation allows you to build modular and reusable parsers tailored to your needs.
//! The framing and content parsers are fully decoupled and composable.
//!
//! - ✅ Choose your compliance level (strict, lenient, or intermediate)
//! - ✅ Plug in custom parsers for any protocol that resembles NMEA
//! - ✅ Support both `&str` and `&[u8]` inputs
//! - ✅ Parse without allocations using [`nom`] combinators
//!
//! ## Quick Start
//!
//! ### Custom Content Parsing
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
//! use nom::Parser;
//!
//! // Your custom parsing logic for the sentence content
//! fn parse_fields(input: &str) -> IResult<&str, Vec<&str>> {
//!     Ok(("", input.split(',').collect()))
//! }
//!
//! // Create a strict parser (checksum + CRLF required)
//! let mut parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(parse_fields);
//!
//! let result = parser.parse("$GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,*73\r\n");
//! match result {
//!     Ok((_remaining, fields)) => {
//!         println!("Parsed {} fields", fields.len());
//!     }
//!     Err(e) => println!("Parse error: {:?}", e),
//! }
//! ```
//!
//! ### Built-in NMEA Content Parser
//!
//! For common NMEA 0183 sentences, use the built-in content parser with the `nmea-content` feature:
//!
//! ```rust
//! #[cfg(feature = "nmea-content")]
//! {
//! use nmea0183_parser::{
//!     ChecksumMode, LineEndingMode, nmea0183,
//!     nmea_content::{NmeaSentence, Parsable}
//! };
//! use nom::Parser;
//!
//! let mut parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(NmeaSentence::parser);
//!
//! let result = parser.parse("$GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,*73\r\n");
//!
//! if let Ok((_remaining, sentence)) = result {
//!     match sentence {
//!         NmeaSentence::GGA(gga) => {
//!             println!("GPS Fix at: {:?}, {:?}", gga.latitude, gga.longitude);
//!         }
//!         _ => println!("Other NMEA sentence parsed"),
//!     }
//! }
//! }
//! ```
//!
//! ## Configuration Options
//!
//! The [`nmea0183`] function accepts [`ChecksumMode`] and [`LineEndingMode`] parameters to control validation:
//!
//! ```rust
//! use nmea0183_parser::{ChecksumMode, LineEndingMode, nmea0183, IResult};
//! use nom::Parser;
//!
//! fn simple_parser(input: &str) -> IResult<&str, &str> {
//!     Ok(("", input))
//! }
//!
//! // Strict: checksum and CRLF both required
//! let mut strict_parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(simple_parser);
//!
//! // Forbidden CRLF: checksum required, no CRLF allowed
//! let mut no_crlf_parser = nmea0183(ChecksumMode::Required, LineEndingMode::Forbidden)(simple_parser);
//!
//! // Optional checksum: checksum optional, CRLF required
//! let mut optional_checksum_parser = nmea0183(ChecksumMode::Optional, LineEndingMode::Required)(simple_parser);
//!
//! // Lenient: checksum optional, no CRLF allowed
//! let mut lenient_parser = nmea0183(ChecksumMode::Optional, LineEndingMode::Forbidden)(simple_parser);
//! ```
//!
//! ## Error Handling
//!
//! All parsers return [`IResult`] values using [`Error`] for rich, diagnostic-friendly error reporting.
//!
//! ## Feature Flags
//!
//! Enable specific functionality via Cargo feature flags:
//!
//! - **`nmea-content`**: Enables built-in content parser for common NMEA 0183 sentences (pre-2.3)
//! - **`nmea-v2-3`**: Support for NMEA 0183 version 2.3 (older GPS/marine equipment)
//! - **`nmea-v3-0`**: Support for NMEA 0183 version 3.0 (includes v2.3 features)
//! - **`nmea-v4-11`**: Support for NMEA 0183 version 4.11 (includes v3.0 features, modern equipment)
//! - **`serde`**: Enables serialization/deserialization support for NMEA sentences
//!
//! Note: Higher version features automatically include support for lower versions.
//!
//! [`nom`]: https://docs.rs/nom/latest/nom

mod error;
mod nmea0183;
#[cfg(feature = "nmea-content")]
pub mod nmea_content;

pub use error::{Error, IResult};
pub use nmea0183::*;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct README;

#[cfg(test)]
mod tests {
    mod cc_crlf00;
    mod cc_crlf01;
    mod cc_crlf10;
    mod cc_crlf11;
    mod crlf;
}
