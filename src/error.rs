//! # Error Types
//!
//! This module defines the error types used throughout the NMEA parsing library.

use nom::error::{ErrorKind, FromExternalError, ParseError};
use std::fmt::Debug;

/// Holds the result of parsing functions.
///
/// It depends on the input type `I`, the output type `O`, and the error type `E`
/// (by default `nom::error::Error<I>`).
///
/// The `Ok` side is a pair containing the remainder of the input (the part of the data that
/// was not parsed) and the produced value. The `Err` side contains an instance of `nom::Err`.
///
/// Outside of the parsing code, you can use the [nom::Finish::finish] method to convert
/// it to a more common result type.
pub type IResult<I, O, E = nom::error::Error<I>> = nom::IResult<I, O, Error<I, E>>;

/// Represents all possible errors that can occur during NMEA message parsing.
///
/// This enum covers various failure modes including input validation,
/// checksum verification, and parsing errors.
#[derive(Debug, PartialEq)]
pub enum Error<I, E> {
    /// The provided input contains non-ASCII characters.
    ///
    /// NMEA messages must be ASCII-only for proper parsing and checksum calculation.
    NonAscii,

    /// The checksum of the sentence was corrupt or incorrect.
    ///
    /// Contains both the expected checksum (calculated from the message content)
    /// and the actual checksum found in the message.
    ChecksumMismatch {
        /// The checksum calculated from the message content
        expected: u8,
        /// The checksum found in the message
        found: u8,
    },

    /// The sentence could not be parsed because its format was invalid.
    ///
    /// This wraps nom's standard parsing errors and provides context about
    /// what went wrong during parsing.
    ParsingError(E),

    /// The message type is not recognized by the parser.
    ///
    /// This variant is used when a valid NMEA sentence is encountered, but the
    /// parser does not implement handling for this specific message type.
    /// The message type that caused the error is provided for reference.
    UnrecognizedMessage(I),

    /// A field in the NMEA sentence was invalid.
    ///
    /// This error occurs when a specific field in the NMEA sentence does not
    /// conform to the expected format, type, or value range.
    ///
    /// Contains the input that caused the error.
    InvalidField(I),

    /// An unknown error occurred.
    ///
    /// This is a catch-all for unexpected error conditions.
    Unknown,
}

impl<I, E> ParseError<I> for Error<I, E>
where
    E: ParseError<I>,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Error::ParsingError(E::from_error_kind(input, kind))
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E, EX> FromExternalError<I, EX> for Error<I, E>
where
    E: FromExternalError<I, EX>,
{
    fn from_external_error(input: I, kind: ErrorKind, e: EX) -> Self {
        Error::ParsingError(E::from_external_error(input, kind, e))
    }
}
