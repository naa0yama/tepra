//! Pure functions for merge-print CSV building and serial-number expansion.
//!
//! No I/O: callers own file/network access and pass in already-fetched
//! `ImportFrameItem` frames and user-supplied rows.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use tepra_core::dto::{
    job::{DensityParam, ErrorMessageParam, PrintParameter},
    template::ImportFrameItem,
};

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

/// Parses a cell reference (e.g. `"A1"`, `"AA10"`, `"B"`) into a
/// `(column_key, row_key)` tuple for column-major sorting.
///
/// The leading alphabetic run is decoded as a base-26 column key (A=1, …,
/// Z=26, AA=27, …); the trailing numeric run becomes the row key (0 when
/// absent). A missing alphabetic run yields column key 0.
#[must_use]
pub fn parse_cell_ref(cell: &str) -> (u32, u32) {
    let alpha_len = cell
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(cell.len());
    let (alpha, digits) = cell.split_at(alpha_len);

    let column_key = alpha.chars().fold(0_u32, |acc, c| {
        let digit = u32::from(c.to_ascii_uppercase())
            .saturating_sub(u32::from(b'A'))
            .saturating_add(1);
        acc.saturating_mul(26).saturating_add(digit)
    });
    let row_key = digits.parse().unwrap_or(0);

    (column_key, row_key)
}

/// Sorts `frames` in place by cell-reference column order (column-major:
/// column key first, row key second), stable for equal keys.
///
/// Normalizes `import_frame`'s array order — which need not match cell
/// reference order — to the order the printer actually binds CSV columns
/// by, so CSV assembly and UI rendering share one column ordering.
pub fn sort_frames_by_column(frames: &mut [ImportFrameItem]) {
    frames.sort_by_key(|frame| parse_cell_ref(&frame.column));
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

/// Per-request overrides for [`PrintParameter`] fields; `None` keeps the
/// SDK default (see [`merge_print_parameter`]).
// WHY-NOT: rename to `Overrides` — matches spec's `MergePrintRequest` field
// naming and disambiguates from other override types outside this module.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct MergePrintOverrides {
    /// Number of print copies (1-999).
    pub copies: Option<u32>,
    /// Overrides `DensityParam.value` (mode stays 1 = specified density).
    pub density: Option<i32>,
    /// 1=not cut, 2=each label, 3=after job.
    pub tape_cut: Option<u32>,
    /// 1=no half-cut, 2=half-cut.
    pub half_cut: Option<u32>,
    /// 1=continuous (joined labels), 2=continuous (separated labels).
    pub half_cut_separate: Option<u32>,
    /// 1=high, 2=low, 3=middle.
    pub print_speed: Option<u32>,
    /// Left/right margin in 0.1mm units.
    pub margin_left_right: Option<u32>,
    /// 2=show tape-width confirmation, 1=hide.
    pub display_tape_width: Option<u32>,
    /// 2=show print-setting confirmation (also drives transfer-tape), 1=hide.
    pub display_print_setting: Option<u32>,
}

/// Builds a [`PrintParameter`] from SDK-default wire values, applying `overrides`.
///
/// Defaults mirror `tepraprint.js`'s `defaultPrintParameter` (:656-) mapped
/// through `tepraprint_getWebApiPrintParameter` to REST wire values.
///
/// Pure function: no I/O.
// WHY-NOT: rename to `parameter` — matches sibling `build_merge_csv` /
// `expand_serial` naming convention within this module.
#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn merge_print_parameter(overrides: &MergePrintOverrides) -> PrintParameter {
    PrintParameter {
        copies: overrides.copies.unwrap_or(1),
        tape_cut: overrides.tape_cut.unwrap_or(2),
        half_cut: overrides.half_cut.unwrap_or(2),
        print_speed: overrides.print_speed.unwrap_or(2),
        density: DensityParam {
            mode: 1,
            value: overrides.density.unwrap_or(0),
        },
        tape_id: 262,
        priority_cut_setting: 1,
        half_cut_separate: overrides.half_cut_separate.unwrap_or(1),
        margin_left_right: overrides.margin_left_right.unwrap_or(0),
        display_tape_width: overrides.display_tape_width.unwrap_or(1),
        error_message: ErrorMessageParam {
            mode: 2,
            file_output: 0,
            file_path: String::new(),
        },
        // WHY-NOT: separate override for transfer_tape — SDK keeps it locked to display_print_setting
        display_transfer_tape: overrides.display_print_setting.unwrap_or(1),
        display_print_setting: overrides.display_print_setting.unwrap_or(1),
        cut_title: 0,
        kana_zen: 0,
        display_print_preview: 1,
        stretch_image: 0,
    }
}
