//! Pure functions for merge-print CSV building and serial-number expansion.
//!
//! No I/O: callers own file/network access and pass in already-fetched
//! `ImportFrameItem` frames and user-supplied rows.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tepra_core::dto::template::ImportFrameItem;

/// One `{title, value}` pair supplied by the caller for a single tape (label).
// WHY-NOT: rename to `Field` — ambiguous outside this module; callers already
// qualify as `merge::MergeField`, matching the sibling DTO naming convention.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MergeField {
    /// Column title matching an `ImportFrameItem::title` (display header).
    pub title: String,
    /// Value to place in the bound frame for this tape.
    pub value: String,
}

/// Serial-number generation spec: fills `title`'s frame with a generated sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SerialSpec {
    /// Column title to bind the generated sequence to.
    pub title: String,
    /// First value in the sequence.
    pub start: i64,
    /// Number of values to generate.
    pub count: u32,
    /// Increment applied between successive values (may be negative).
    pub step: i64,
    /// Minimum digit width for the numeric part, zero-padded (0 = no padding).
    pub pad: u8,
}

/// CSV output character encoding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CsvEncoding {
    /// UTF-8 (default).
    #[default]
    Utf8,
    /// `Shift_JIS` (CP932).
    ShiftJis,
}

/// Builds a headerless RFC4180 CSV byte stream from `frames` (column order)
/// and `rows` (one tape per row).
///
/// # Errors
///
/// Returns an error when `frames` contains a duplicate `title` (ambiguous
/// binding), or when a row contains an unknown or duplicate `title`, or when
/// `encoding` is [`CsvEncoding::ShiftJis`] and a value contains characters
/// unmappable to `Shift_JIS` (CP932).
pub fn build_merge_csv(
    frames: &[ImportFrameItem],
    rows: &[Vec<MergeField>],
    encoding: CsvEncoding,
) -> anyhow::Result<Vec<u8>> {
    let titles: Vec<&str> = frames.iter().map(|f| f.title.as_str()).collect();
    let unique_titles: HashSet<&str> = titles.iter().copied().collect();
    anyhow::ensure!(
        unique_titles.len() == titles.len(),
        "frames contain duplicate title(s); title-to-column binding is ambiguous"
    );

    let mut text = String::new();
    for row in rows {
        let mut seen = HashSet::with_capacity(row.len());
        for field in row {
            anyhow::ensure!(
                unique_titles.contains(field.title.as_str()),
                "row references unknown title {:?} (not present in frames)",
                field.title
            );
            anyhow::ensure!(
                seen.insert(field.title.as_str()),
                "row contains duplicate title {:?}",
                field.title
            );
        }

        let line = frames
            .iter()
            .map(|frame| {
                let value = row
                    .iter()
                    .find(|f| f.title == frame.title)
                    .map_or("", |f| f.value.as_str());
                quote_csv_field(value)
            })
            .collect::<Vec<_>>()
            .join(",");
        text.push_str(&line);
        text.push_str("\r\n");
    }

    encode(&text, encoding)
}

/// Quotes `value` per RFC4180 when it contains a comma, double quote, CR, or LF.
fn quote_csv_field(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

/// Encodes `text` per `encoding`.
///
/// # Errors
///
/// Returns an error when `encoding` is [`CsvEncoding::ShiftJis`] and `text`
/// contains characters that cannot be represented in `Shift_JIS` (CP932) —
/// surfacing the mismatch rather than silently substituting `?` (lossy
/// encoding would corrupt printed labels).
fn encode(text: &str, encoding: CsvEncoding) -> anyhow::Result<Vec<u8>> {
    match encoding {
        CsvEncoding::Utf8 => Ok(text.as_bytes().to_vec()),
        CsvEncoding::ShiftJis => {
            let (bytes, _, had_unmappable) = encoding_rs::SHIFT_JIS.encode(text);
            anyhow::ensure!(
                !had_unmappable,
                "value contains character(s) not representable in Shift_JIS"
            );
            Ok(bytes.into_owned())
        }
    }
}

/// Expands `spec` into `spec.count` `MergeField`s, one per generated value.
#[must_use]
pub fn expand_serial(spec: &SerialSpec) -> Vec<MergeField> {
    (0..i64::from(spec.count))
        .map(|i| {
            let n = spec.start.saturating_add(spec.step.saturating_mul(i));
            MergeField {
                title: spec.title.clone(),
                value: pad_signed(n, spec.pad),
            }
        })
        .collect()
}

/// Formats `n` zero-padded to at least `pad` digits, preserving the sign.
fn pad_signed(n: i64, pad: u8) -> String {
    let pad = usize::from(pad);
    if n < 0 {
        format!("-{:0pad$}", n.unsigned_abs())
    } else {
        format!("{n:0pad$}")
    }
}
