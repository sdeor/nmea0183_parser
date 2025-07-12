# A Flexible NMEA Framing Parser for Rust

**A zero-allocation NMEA 0183 parser that separates message framing from content parsing, giving you full control over how to handle your data.**

This Rust crate provides a generic and configurable parser for NMEA 0183-style messages, with the typical format:

```text
$HHH,D1,D2,...,Dn*CC\r\n
```

It focuses on parsing and validating the framing of NMEA 0183-style sentences (start character, optional checksum, and optional CRLF), allowing you to plug in your own domain-specific content parsers — or use built-in ones for common NMEA sentence types.

---

## ✨ Why Use This Crate?

Unlike traditional NMEA crates that tightly couple format and content parsing, `nmea0183_parser` lets you:

- ✅ Choose your compliance level (strict vs lenient)
- ✅ Plug in your own payload parser (GNSS, marine, custom protocols)
- ✅ Support both `&str` and `&[u8]` inputs
- ✅ Parse without allocations, built on top of [`nom`](https://github.com/Geal/nom), a parser combinator library in Rust."

Perfect for:

- GNSS/GPS receiver integration
- Marine electronics parsing
- IoT devices consuming NMEA 0183-style protocols
- Debugging or testing tools for embedded equipment
- Legacy formats that resemble NMEA but don’t strictly comply

## 📦 Key Features

- ✅ ASCII-only validation
- ✅ Required or optional checksum validation
- ✅ Required or forbidden CRLF ending enforcement
- ✅ Zero-allocation parsing
- ✅ Built on `nom` combinators
- ✅ Fully pluggable content parser (you bring the domain logic)
- ✅ Optional built-in support for common NMEA sentences

---

## 🚀 Getting Started

Add the crate to your project:

```toml
[dependencies]
nmea0183_parser = { git = "https://github.com/sdeor/nmea0183_parser" }
```

### ⚡ Quick Start

Here's a minimal example to get you started with parsing NMEA 0183-style sentences:

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

// Simple content parser that splits fields by comma
fn parse_fields(input: &str) -> IResult<&str, Vec<&str>> {
    Ok(("", input.split(',').collect()))
}

// Create parser with strict validation (checksum + CRLF required)
let parser_factory = nmea0183(ChecksumMode::Required, LineEndingMode::Required);
let mut parser = parser_factory(parse_fields);

// Parse a GPS sentence
let result =
    parser.parse("$GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,*73\r\n");

match result {
    Ok((_remaining, fields)) => {
        println!("Success! Parsed {} fields", fields.len()); // 15 fields
        println!("Sentence type: {}", fields[0]); // "GPGGA"
    }
    Err(e) => println!("Parse error: {:?}", e),
}
```

For custom parsing logic, you can define your own content parser. The `nmea0183` function creates a parser factory that you then call with your content parser:

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

// Your custom logic for the inner data portion (after "$" and before "*CC").
// The parse_content function should return an IResult<Input, Output> type so it can be used with the `nmea0183` factory.
// The `Output` type can be any Rust type you define, such as a struct or enum.
fn parse_content(input: &str) -> IResult<&str, Vec<&str>> {
    // You can decode fields here, split by commas, etc.
    Ok(("", input.split(',').collect()))
}

// The `nmea0183` function returns a factory that takes your content parser
let parser_factory = nmea0183(ChecksumMode::Required, LineEndingMode::Required);
let mut parser = parser_factory(parse_content);

// Or combine into one line:
// let mut parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(parse_content);

// Try parsing a message
match parser.parse("$Header,field1,field2*3C\r\n") {
    Ok((remaining, fields)) => {
        assert_eq!(remaining, "");
        assert_eq!(fields, vec!["Header", "field1", "field2"]);
    }
    Err(e) => println!("Parse error: {:?}", e),
}
```

## 🧐 How It Works

1. **Framing parser** handles the outer structure:

   - ASCII-only validation
   - Start delimiter (`$`)
   - Optional checksum validation (`*CC`)
   - Optional CRLF endings (`\r\n`)

2. **Your content parser**, or built-in ones, handle the inner data (`D1,D2,...,Dn`):

   - Field parsing and validation
   - Type conversion
   - Domain-specific logic

You get full control over how sentence content is interpreted.

In the above example, `parse_content` is your custom logic that processes the inner data of the sentence. The `nmea0183` function creates a parser that handles the framing, while you focus on the content.

---

## 🔧 Configuration Options

You can configure the parser's behavior using `ChecksumMode` and `LineEndingMode`:

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

fn content_parser(input: &str) -> IResult<&str, bool> {
    Ok((input, true))
}

// Strict: checksum and CRLF both required
let mut strict = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(content_parser);
assert!(strict.parse("$GPGGA,data*6A\r\n").is_ok());
assert!(strict.parse("$GPGGA,data*6A").is_err());  // Missing CRLF
assert!(strict.parse("$GPGGA,data\r\n").is_err()); // Missing checksum

// Checksum required, CRLF forbidden
let mut no_crlf = nmea0183(ChecksumMode::Required, LineEndingMode::Forbidden)(content_parser);
assert!(no_crlf.parse("$GPGGA,data*6A").is_ok());
assert!(no_crlf.parse("$GPGGA,data*6A\r\n").is_err());
assert!(no_crlf.parse("$GPGGA,data").is_err());

// Checksum optional, CRLF required
let mut optional_checksum = nmea0183(ChecksumMode::Optional, LineEndingMode::Required)(content_parser);
assert!(optional_checksum.parse("$GPGGA,data*6A\r\n").is_ok());
assert!(optional_checksum.parse("$GPGGA,data\r\n").is_ok());
assert!(optional_checksum.parse("$GPGGA,data*99\r\n").is_err());
assert!(optional_checksum.parse("$GPGGA,data*6A").is_err());

// Lenient: checksum optional, CRLF forbidden
let mut lenient = nmea0183(ChecksumMode::Optional, LineEndingMode::Forbidden)(content_parser);
assert!(lenient.parse("$GPGGA,data*6A").is_ok());
assert!(lenient.parse("$GPGGA,data").is_ok());
assert!(lenient.parse("$GPGGA,data*99").is_err());
assert!(lenient.parse("$GPGGA,data\r\n").is_err());
```

---

## 🔍 Parsing Both String and Byte Inputs

The parser can handle both `&str` and `&[u8]` inputs. You can define your content parser to work with either type, and the factory will adapt accordingly.

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

fn parse_content_str(input: &str) -> IResult<&str, Vec<&str>> {
    Ok(("", input.split(',').collect()))
}

let mut parser_str = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(parse_content_str);

// Parse from string
let string_input = "$Header,field1,field2*3C\r\n";
let result = parser_str.parse(string_input);

assert!(result.is_ok());
assert_eq!(result.unwrap().1, vec!["Header", "field1", "field2"]);

fn parse_content_bytes(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, first_byte) = nom::number::complete::u8(input)?;
    Ok((input, first_byte))
}

let mut parser_bytes = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(parse_content_bytes);

// Parse from bytes
let byte_input = b"$Header,field1,field2*3C\r\n";
let result_bytes = parser_bytes.parse(byte_input);

assert!(result_bytes.is_ok());
assert_eq!(result_bytes.unwrap().1, 72); // 'H' is the first byte of the content
```

---

## 🧱 Built-in NMEA Sentence Content Parser

Alongside the flexible framing parser, this crate provides a built-in content parser for common NMEA 0183 sentence types.

This parser parses only the content. It does not handle framing — such as the initial `$`, optional checksum (`*CC`), or optional CRLF (`\r\n`). That responsibility belongs to the factory function, which wraps around the content parser.

To parse a complete NMEA sentence, you can use the `nmea0183` factory with the built-in content parser:

```rust
#[cfg(feature = "nmea-content")]
{
use nmea0183_parser::{
    ChecksumMode, LineEndingMode,
    nmea_content::{NmeaSentence, Parsable, Quality},
    nmea0183,
};
use nom::Parser;

let mut nmea_parser =
    nmea0183(ChecksumMode::Required, LineEndingMode::Required)(NmeaSentence::parser);

let result =
    nmea_parser.parse("$GPGGA,123456.00,4916.29,N,12311.76,W,1,08,0.9,545.4,M,46.9,M,,*73\r\n");

assert!(
    result.is_ok(),
    "Failed to parse NMEA sentence: {:?}",
    result.unwrap_err()
);

let sentence = result.unwrap().1;

match sentence {
    NmeaSentence::GGA(gga) => {
        assert_eq!(gga.latitude, Some(49.2715));
        assert_eq!(gga.longitude, Some(-123.196));
        assert_eq!(gga.fix_quality, Quality::GPSFix);
        assert_eq!(gga.satellite_count, Some(8));
        assert_eq!(gga.hdop, Some(0.9));
        // etc...
    }
    _ => {
        println!("Parsed other NMEA sentence");
    }
}
}
```

Enable the content parser with the `nmea-content` feature:

```toml
[dependencies]
nmea0183_parser = { git = "https://github.com/sdeor/nmea0183_parser", features = ["nmea-content"] }
```

**Note:** While the `nmea0183` framing parser can accept both `&str` and `&[u8]` inputs, the built-in content parser only accepts `&str`, as it is designed specifically for text-based NMEA sentences.

### Supported NMEA Sentences

- [`DBT`](https://gpsd.gitlab.io/gpsd/NMEA.html#_dbt_depth_below_transducer) - Depth Below Transducer
- [`DPT`](https://gpsd.gitlab.io/gpsd/NMEA.html#_dpt_depth_of_water) - Depth of Water
- [`GGA`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gga_global_positioning_system_fix_data) - Global Positioning System Fix Data
- [`GLL`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gll_geographic_position_latitudelongitude) - Geographic Position: Latitude/Longitude
- [`GSA`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gsa_gps_dop_and_active_satellites) - GPS DOP and Active Satellites
- [`GSV`](https://gpsd.gitlab.io/gpsd/NMEA.html#_gsv_satellites_in_view) - Satellites in View
- [`RMC`](https://gpsd.gitlab.io/gpsd/NMEA.html#_rmc_recommended_minimum_navigation_information) - Recommended Minimum Navigation Information
- [`VTG`](https://gpsd.gitlab.io/gpsd/NMEA.html#_vtg_track_made_good_and_ground_speed) - Track made good and Ground speed
- [`ZDA`](https://gpsd.gitlab.io/gpsd/NMEA.html#_zda_time_date_utc_day_month_year_and_local_time_zone) - Time & Date: UTC, day, month, year and local time zone

### NMEA Version Support

Different NMEA versions may include additional fields in certain sentence types. You can choose the version that matches your equipment by enabling the appropriate feature flags.

| Feature Flag   | NMEA Version | When to Use                |
| -------------- | ------------ | -------------------------- |
| `nmea-content` | Pre-2.3      | Standard NMEA parsing      |
| `nmea-v2-3`    | NMEA 2.3     | Older GPS/marine equipment |
| `nmea-v3-0`    | NMEA 3.0     | Mid-range equipment        |
| `nmea-v4-11`   | NMEA 4.11    | Modern equipment           |

For specific field differences between versions, please refer to the [NMEA 0183 standard documentation](https://gpsd.gitlab.io/gpsd/NMEA.html).

Example configuration:

```toml
[dependencies]
# For basic NMEA parsing
nmea0183_parser = { git = "https://github.com/sdeor/nmea0183_parser", features = ["nmea-content"] }

# For modern equipment with NMEA 4.11 support
nmea0183_parser = { git = "https://github.com/sdeor/nmea0183_parser", features = ["nmea-content", "nmea-v4-11"] }
```

You can read more about the NMEA 0183 standard [here](https://gpsd.gitlab.io/gpsd/NMEA.html).

---

## 🛠️ Contributing

Contributions are very welcome! Open an issue or PR for:

- Bug fixes
- Integration tests and samples
- Documentation improvements
- New content parsers for additional NMEA sentences

You can submit [issues](https://github.com/sdeor/nmea0183_parser/issues) or [pull requests](https://github.com/sdeor/nmea0183_parser/pulls) to contribute.

---

## 📚 Documentation

- [NMEA 0183 Standard Reference](https://gpsd.gitlab.io/gpsd/NMEA.html)
- [`nom` Parser Combinators](https://docs.rs/nom/latest/nom/)
