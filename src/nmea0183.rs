//! # NMEA 0183 Message Parser
//!
//! This module provides the main parsing functionality for NMEA 0183-style messages.
//! It handles the standard NMEA 0183 format: `$HHH,D1,D2,...,Dn*CC\r\n`
//!
//! The parser is configurable to handle variations in:
//! - Checksum requirements (required or optional)
//! - Line ending requirements (CRLF required or forbidden)

use nom::{
    AsBytes, AsChar, Compare, Err, FindSubstring, Input, Parser,
    branch::alt,
    bytes::complete::{tag, take, take_until},
    character::complete::{char, hex_digit0},
    combinator::{opt, rest, rest_len, verify},
    error::{ErrorKind, ParseError},
    number::complete::hex_u32,
    sequence::terminated,
};

use crate::{Error, IResult};

type NmeaParser<'a, I, O, E> = dyn FnMut(I) -> IResult<I, O, E> + 'a;

/// Defines how the parser should handle NMEA message checksums.
///
/// NMEA 0183 messages can include an optional checksum in the format `*CC` where
/// CC is a two-digit hexadecimal value representing the XOR of all bytes in the
/// message content (excluding the '$' prefix and '*' delimiter).
#[derive(Clone, Copy, PartialEq)]
pub enum ChecksumMode {
    /// Checksum is required and must be present.
    ///
    /// The parser will fail if no `*CC` checksum is found at the end of the message.
    /// If a checksum is present, it will be validated against the calculated checksum.
    ///
    /// Use this mode for strict NMEA 0183 compliance or when data integrity is critical.
    Required,

    /// Checksum is optional but will be validated if present.
    ///
    /// The parser will accept messages both with and without checksums:
    /// - If no checksum is present (`*CC` missing), parsing continues normally
    /// - If a checksum is present, it must be valid or parsing will fail
    ///
    /// Use this mode when working with mixed message sources or legacy equipment
    /// that may not always include checksums.
    Optional,
}

/// Defines how the parser should handle CRLF line endings.
///
/// NMEA 0183 messages typically end with a carriage return and line feed (`\r\n`),
/// but some systems or applications may omit these characters.
#[derive(Clone, Copy, PartialEq)]
pub enum LineEndingMode {
    /// CRLF line ending is required and must be present.
    ///
    /// The parser will fail if the message does not end with `\r\n`.
    /// This is the standard NMEA 0183 format for messages transmitted over
    /// serial connections or stored in files.
    ///
    /// Use this mode when parsing standard NMEA log files or serial port data.
    Required,

    /// CRLF line ending is forbidden and must not be present.
    ///
    /// The parser will fail if the message ends with `\r\n`.
    /// This mode is useful when parsing NMEA messages that have been processed
    /// or transmitted through systems that strip line endings.
    ///
    /// Use this mode when parsing messages from APIs, databases, or other
    /// sources where line endings have been removed.
    Forbidden,
}

/// Creates a configurable NMEA 0183 parser factory.
///
/// This function returns a factory that can be used to create NMEA parsers with
/// different content parsing logic. The factory handles the standard NMEA framing
/// (checksum, line endings) while allowing custom parsing of the message content.
///
/// # Arguments
///
/// * `cc` - Checksum requirement:
///   - [`ChecksumMode::Required`]: Parser will fail if no '*CC' is present
///   - [`ChecksumMode::Optional`]: Parser accepts messages with or without '*CC',
///     but validates checksum if present
/// * `crlf` - CRLF requirement:
///   - [`LineEndingMode::Required`]: Parser will fail if message doesn't end with `\r\n`
///   - [`LineEndingMode::Forbidden`]: Parser will fail if message ends with `\r\n`
///
/// # Returns
///
/// A factory function that takes a content parser and returns a complete NMEA parser.
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
/// use nom::Parser;
///
/// // Create a parser factory with required checksum and CRLF
/// let parser_factory = nmea0183(ChecksumMode::Required, LineEndingMode::Required);
///
/// // Define custom content parser
/// fn parse_gps_data(i: &str) -> IResult<&str, String> {
///     Ok((i, "parsed_data".to_string()))
/// }
///
/// // Create the actual parser
/// let mut parser = parser_factory(parse_gps_data);
///
/// // Use the parser - this will succeed
/// let result = parser.parse("$GPGGA,123456,data*41\r\n");
/// ```
///
/// # Configuration Examples
///
/// ```rust
/// use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
/// use nom::Parser;
///
/// fn content_parser(i: &str) -> IResult<&str, bool> {
///     Ok((i, true))
/// }
///
/// // Strict NMEA compliance - checksum and CRLF both required
/// let mut strict_parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(content_parser);
/// assert!(strict_parser.parse("$GPGGA,data*6A\r\n").is_ok());
/// assert!(strict_parser.parse("$GPGGA,data*6A").is_err());  // (missing CRLF)
/// assert!(strict_parser.parse("$GPGGA,data\r\n").is_err()); // (missing checksum)
///
/// // Checksum required, no CRLF allowed
/// let mut no_crlf_parser = nmea0183(ChecksumMode::Required, LineEndingMode::Forbidden)(content_parser);
/// assert!(no_crlf_parser.parse("$GPGGA,data*6A").is_ok());
/// assert!(no_crlf_parser.parse("$GPGGA,data*6A\r\n").is_err()); // (CRLF present)
/// assert!(no_crlf_parser.parse("$GPGGA,data").is_err());        // (missing checksum)
///
/// // Checksum optional, CRLF required
/// let mut optional_checksum_parser = nmea0183(ChecksumMode::Optional, LineEndingMode::Required)(content_parser);
/// assert!(optional_checksum_parser.parse("$GPGGA,data*6A\r\n").is_ok());  // (with valid checksum)
/// assert!(optional_checksum_parser.parse("$GPGGA,data\r\n").is_ok());     // (without checksum)
/// assert!(optional_checksum_parser.parse("$GPGGA,data*99\r\n").is_err()); // (invalid checksum)
/// assert!(optional_checksum_parser.parse("$GPGGA,data*6A").is_err());     // (missing CRLF)
///
/// // Lenient parsing - checksum optional, no CRLF allowed
/// let mut lenient_parser = nmea0183(ChecksumMode::Optional, LineEndingMode::Forbidden)(content_parser);
/// assert!(lenient_parser.parse("$GPGGA,data*6A").is_ok());   // (with valid checksum)
/// assert!(lenient_parser.parse("$GPGGA,data").is_ok());      // (without checksum)
/// assert!(lenient_parser.parse("$GPGGA,data*99").is_err());  // (invalid checksum)
/// assert!(lenient_parser.parse("$GPGGA,data\r\n").is_err()); // (CRLF present)
/// ```
pub fn nmea0183<'a, I, O, F, E>(
    cc: ChecksumMode,
    crlf: LineEndingMode,
) -> impl Fn(F) -> Box<NmeaParser<'a, I, O, E>>
where
    I: Input + AsBytes + Compare<&'a str> + FindSubstring<&'a str> + 'a,
    <I as Input>::Item: AsChar,
    O: 'a,
    F: Parser<I, Output = O, Error = Error<I, E>> + 'a,
    E: ParseError<I> + 'a,
{
    move |f: F| Box::new(nmea0183_inner(f, checksum_crlf(cc, crlf)))
}

/// Creates a parser for checksum and CRLF based on configuration.
///
/// This function returns a parser that can handle the end portion of NMEA messages,
/// specifically the checksum (if present) and line ending (if present).
///
/// # Arguments
///
/// * `cc` - Checksum requirement:
///   - [`ChecksumMode::Required`]: Parser will fail if no '*CC' is present
///   - [`ChecksumMode::Optional`]: Parser accepts messages with or without '*CC',
///     but validates checksum if present
/// * `crlf` - CRLF requirement:
///   - [`LineEndingMode::Required`]: Parser will fail if message doesn't end with `\r\n`
///   - [`LineEndingMode::Forbidden`]: Parser will fail if message ends with `\r\n`
///
/// # Returns
///
/// A parser that extracts the checksum value ([`None`] if no checksum present).
///
/// # Message Format Expectations
///
/// - cc=[`ChecksumMode::Required`], crlf=[`LineEndingMode::Required`]: Expects `*CC\r\n`
/// - cc=[`ChecksumMode::Required`], crlf=[`LineEndingMode::Forbidden`]: Expects `*CC`
/// - cc=[`ChecksumMode::Optional`], crlf=[`LineEndingMode::Required`]: Expects `\r\n` or `*CC\r\n`
/// - cc=[`ChecksumMode::Optional`], crlf=[`LineEndingMode::Forbidden`]: Expects nothing or `*CC`
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, checksum_crlf};
/// use nom::Parser;
///
/// // Required checksum, required CRLF
/// let mut parser = checksum_crlf(ChecksumMode::Required, LineEndingMode::Required);
/// let result: IResult<_, _> = parser.parse("*51\r\n");
/// assert_eq!(result, Ok(("", Some(0x51))));
///
/// // Optional checksum, forbidden CRLF
/// let mut parser = checksum_crlf(ChecksumMode::Optional, LineEndingMode::Forbidden);
/// let result1: IResult<_, _> = parser.parse("*51");     // With checksum
/// let result2: IResult<_, _> = parser.parse("");        // Without checksum
/// assert!(result1.is_ok());
/// assert!(result2.is_ok());
/// ```
pub fn checksum_crlf<'a, I, E: ParseError<I>>(
    cc: ChecksumMode,
    le: LineEndingMode,
) -> impl FnMut(I) -> nom::IResult<I, Option<u8>, E>
where
    I: Input + AsBytes + Compare<&'a str> + FindSubstring<&'a str>,
    <I as Input>::Item: AsChar,
{
    move |i: I| {
        let (i, _) = crlf(le).parse(i)?;

        let (cc, parse_cc) = match cc {
            ChecksumMode::Required => char('*').map(|_| true).parse(i)?,
            ChecksumMode::Optional => opt(char('*')).map(|parse_cc| parse_cc.is_some()).parse(i)?,
        };

        if parse_cc {
            let (_, cc) = consumed(take(2u8), ErrorKind::Count).parse(cc)?;
            let (_, cc) = consumed(hex_digit0, ErrorKind::IsA).parse(cc)?;

            hex_u32.map(|cc| Some(cc as u8)).parse(cc)
        } else if cc.input_len() != 0 {
            Err(Err::Error(E::from_error_kind(cc, ErrorKind::Count)))
        } else {
            Ok((cc, None))
        }
    }
}

/// Parses CRLF line endings based on configuration.
///
/// This function handles the parsing of carriage return and line feed characters
/// at the end of NMEA messages, with support for both required and forbidden modes.
///
/// # Arguments
///
/// * `crlf` - CRLF requirement:
///   - [`LineEndingMode::Required`]: Parser will fail if message doesn't end with `\r\n`
///   - [`LineEndingMode::Forbidden`]: Parser will fail if message ends with `\r\n`
///
/// # Returns
///
/// A parser function that validates CRLF presence according to the configuration.
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::{IResult, LineEndingMode, crlf};
/// use nom::Parser;
///
/// // CRLF required
/// let mut parser = crlf(LineEndingMode::Required);
/// let result: IResult<_, _> = parser.parse("data\r\n");
/// assert_eq!(result, Ok(("data", ())));
///
/// // CRLF forbidden
/// let mut parser = crlf(LineEndingMode::Forbidden);
/// let result: IResult<_, _> = parser.parse("data");
/// assert_eq!(result, Ok(("data", ())));
/// ```
pub fn crlf<'a, I, E: ParseError<I>>(crlf: LineEndingMode) -> impl Fn(I) -> nom::IResult<I, (), E>
where
    I: Input + Compare<&'a str> + FindSubstring<&'a str>,
{
    move |i: I| {
        let (i, data) = opt(take_until("\r\n")).parse(i)?;

        let data = if crlf == LineEndingMode::Required {
            match data {
                Some(data) => {
                    let (_, _) = consumed(tag("\r\n"), ErrorKind::CrLf).parse(i)?;
                    data
                }
                None => {
                    return Err(Err::Error(E::from_error_kind(i, ErrorKind::CrLf)));
                }
            }
        } else if data.is_some() {
            return Err(Err::Error(E::from_error_kind(i, ErrorKind::CrLf)));
        } else {
            i
        };

        Ok((data, ()))
    }
}

/// Calculates the NMEA 0183 checksum for the given message content.
///
/// The NMEA 0183 checksum is calculated by performing an XOR (exclusive OR) operation
/// on all bytes in the message content. This includes everything between the '$' prefix
/// and the '*' checksum delimiter, but excludes both the '$' and '*' characters themselves.
///
/// # Algorithm
///
/// 1. Initialize checksum to 0
/// 2. For each byte in the message content:
///    - XOR the current checksum with the byte value
/// 3. The final result is an 8-bit value (0-255)
///
/// # Arguments
///
/// * `input` - The message content to calculate checksum for (without '$' prefix or '*' delimiter)
///
/// # Returns
///
/// A tuple of (input, checksum) where:
/// - `input` is returned unchanged (zero-copy)
/// - `checksum` is the calculated XOR value as a u8
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::checksum;
///
/// // Calculate checksum for "GPGGA,123456,data"
/// let (_, cc) = checksum("GPGGA,123456,data");
/// assert_eq!(cc, 0x41);
/// ```
///
/// # NMEA 0183 Standard
///
/// According to the NMEA 0183 standard:
/// - The checksum is represented as a two-digit hexadecimal number
/// - It appears after the '*' character at the end of the sentence
/// - Example: `$GPGGA,123456,data*41` where '41' is the hex representation of the checksum
///
/// # Performance Notes
///
/// This function uses `fold()` with XOR operation, which is:
/// - Efficient for small to medium message sizes (typical NMEA messages are < 100 bytes)
/// - Single-pass algorithm with O(n) time complexity
/// - No memory allocation (zero-copy input handling)
pub fn checksum<I>(input: I) -> (I, u8)
where
    I: Input + AsBytes,
{
    let calculated_checksum = input
        .as_bytes()
        .iter()
        .fold(0u8, |accumulated_xor, &byte| accumulated_xor ^ byte);

    (input, calculated_checksum)
}

/// Formats a checksum value as a two-digit uppercase hexadecimal string.
///
/// # Examples
///
/// ```rust
/// use nmea0183_parser::format_checksum;
///
/// let checksum = 0x41;
/// assert_eq!(format_checksum(checksum), "41");
///
/// let checksum = 0x0A;
/// assert_eq!(format_checksum(checksum), "0A");
/// ```
pub fn format_checksum(checksum: u8) -> String {
    format!("{checksum:02X}")
}

/// Internal implementation of the NMEA 0183 parser.
///
/// This function handles the common NMEA message structure:
/// 1. Validates that input is ASCII-only
/// 2. Expects the message to start with '$'
/// 3. Extracts the message content (everything before '*' or '\r\n')
/// 4. Parses and validates the checksum using the provided checksum parser
/// 5. Calls the user-provided parser on the message content
///
/// # Arguments
///
/// * `f` - User-provided parser for the message content
/// * `cc_parser` - Parser for extracting and validating checksum
///
/// # Returns
///
/// A parser function that can be called with input to parse NMEA messages.
fn nmea0183_inner<'a, I, O, F, CC, E>(
    mut f: F,
    mut cc_parser: CC,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: Input + AsBytes + Compare<&'a str> + FindSubstring<&'a str>,
    <I as Input>::Item: AsChar,
    F: Parser<I, Output = O, Error = Error<I, E>>,
    CC: Parser<I, Output = Option<u8>, Error = Error<I, E>>,
    E: ParseError<I>,
{
    move |i: I| {
        if !i.as_bytes().is_ascii() {
            return Err(nom::Err::Error(Error::NonAscii));
        }

        let (i, _) = char('$').parse(i)?;

        let (cc, data) = alt((take_until("*"), take_until("\r\n"), rest)).parse(i)?;
        let (_, cc) = cc_parser.parse(cc)?;

        let (data, calc_cc) = checksum(data);

        if let Some(cc) = cc
            && cc != calc_cc
        {
            return Err(nom::Err::Error(Error::ChecksumMismatch {
                expected: calc_cc,
                found: cc,
            }));
        }

        f.parse(data)
    }
}

/// Ensures that the parser consumes all input.
///
/// This is a convenience function for the common case of wanting to ensure that
/// a parser consumes the entire input with no remainder.
///
/// # Arguments
///
/// * `f` - The parser to run
/// * `e` - Error kind to return if input is not fully consumed
///
/// # Examples
///
/// ```compile_fail
/// use nmea0183_parser::nmea0183::consumed;
/// use nom::{IResult, Parser, bytes::complete::take, error::ErrorKind};
///
/// // Parse all 3 bytes
/// let mut parser = consumed(take(3u8), ErrorKind::Count);
/// let result: IResult<_, _> = parser.parse("abc");
/// assert!(result.is_ok());
///
/// // This would fail because not all input is consumed
/// let result = parser.parse("abcd");
/// assert!(result.is_err());
/// ```
pub(crate) fn consumed<I, E: ParseError<I>, F>(
    f: F,
    e: ErrorKind,
) -> impl Parser<I, Output = <F as Parser<I>>::Output, Error = E>
where
    I: Input,
    F: Parser<I, Error = E>,
{
    terminated(
        f,
        verify(rest_len, |len| len == &0).or(move |i| Err(Err::Error(E::from_error_kind(i, e)))),
    )
}
