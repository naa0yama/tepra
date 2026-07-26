//! Unit tests for `build_merge_csv`, `expand_serial`, and frame column
//! normalization (`parse_cell_ref` / `sort_frames_by_column`).

use tepra::merge::{
    CsvEncoding, MergeField, SerialSpec, build_merge_csv, expand_serial, parse_cell_ref,
    sort_frames_by_column,
};
use tepra_core::dto::{enums::ImportFrameAttribute, template::ImportFrameItem};

fn frame(column: &str, title: &str) -> ImportFrameItem {
    ImportFrameItem {
        column: column.to_owned(),
        title: title.to_owned(),
        attribute: ImportFrameAttribute::Text,
    }
}

fn field(title: &str, value: &str) -> MergeField {
    MergeField {
        title: title.to_owned(),
        value: value.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// build_merge_csv
// ---------------------------------------------------------------------------

#[test]
fn test_column_order_follows_frames() {
    let frames = [frame("B1", "Bち"), frame("A1", "AA")];
    let rows = [vec![field("AA", "tanaka"), field("Bち", "12345")]];

    let csv = build_merge_csv(&frames, &rows, CsvEncoding::Utf8).unwrap();

    assert_eq!(String::from_utf8(csv).unwrap(), "12345,tanaka\r\n");
}

#[test]
fn test_missing_title_yields_empty_cell() {
    let frames = [frame("A1", "AA"), frame("B1", "Bち")];
    let rows = [vec![field("AA", "tanaka")]];

    let csv = build_merge_csv(&frames, &rows, CsvEncoding::Utf8).unwrap();

    assert_eq!(String::from_utf8(csv).unwrap(), "tanaka,\r\n");
}

#[test]
fn test_duplicate_title_in_frames_is_err() {
    let frames = [frame("A1", "AA"), frame("B1", "AA")];
    let rows = [vec![field("AA", "tanaka")]];

    let result = build_merge_csv(&frames, &rows, CsvEncoding::Utf8);

    assert!(result.is_err());
}

#[test]
fn test_unknown_title_in_row_is_err() {
    let frames = [frame("A1", "AA")];
    let rows = [vec![field("Unknown", "tanaka")]];

    let result = build_merge_csv(&frames, &rows, CsvEncoding::Utf8);

    assert!(result.is_err());
}

#[test]
fn test_duplicate_title_in_row_is_err() {
    let frames = [frame("A1", "AA")];
    let rows = [vec![field("AA", "tanaka"), field("AA", "sato")]];

    let result = build_merge_csv(&frames, &rows, CsvEncoding::Utf8);

    assert!(result.is_err());
}

#[test]
fn test_quoting_comma_quote_newline() {
    let frames = [frame("A1", "AA")];
    let rows = [vec![field("AA", "a,b\"c\r\nd")]];

    let csv = build_merge_csv(&frames, &rows, CsvEncoding::Utf8).unwrap();

    assert_eq!(String::from_utf8(csv).unwrap(), "\"a,b\"\"c\r\nd\"\r\n");
}

// ---------------------------------------------------------------------------
// parse_cell_ref
// ---------------------------------------------------------------------------

#[test]
fn test_parse_cell_ref_single_letter_no_row() {
    assert_eq!(parse_cell_ref("A"), (1, 0));
    assert_eq!(parse_cell_ref("B"), (2, 0));
    assert_eq!(parse_cell_ref("Z"), (26, 0));
}

#[test]
fn test_parse_cell_ref_letter_with_row() {
    assert_eq!(parse_cell_ref("A1"), (1, 1));
}

#[test]
fn test_parse_cell_ref_double_letter() {
    assert_eq!(parse_cell_ref("AA"), (27, 0));
    assert_eq!(parse_cell_ref("AA10"), (27, 10));
}

#[test]
fn test_parse_cell_ref_empty_string() {
    assert_eq!(parse_cell_ref(""), (0, 0));
}

// ---------------------------------------------------------------------------
// sort_frames_by_column
// ---------------------------------------------------------------------------

#[test]
fn test_sort_frames_by_column_reorders_to_cell_reference_order() {
    let mut frames = vec![frame("B", "username"), frame("A", "URI")];

    sort_frames_by_column(&mut frames);

    assert_eq!(
        frames.iter().map(|f| f.title.as_str()).collect::<Vec<_>>(),
        vec!["URI", "username"]
    );
}

#[test]
fn test_sort_frames_by_column_is_stable_for_same_column() {
    let mut frames = vec![frame("A", "first"), frame("A", "second")];

    sort_frames_by_column(&mut frames);

    assert_eq!(
        frames.iter().map(|f| f.title.as_str()).collect::<Vec<_>>(),
        vec!["first", "second"]
    );
}

#[test]
fn test_multiple_rows_are_crlf_joined_without_header() {
    let frames = [frame("A1", "AA")];
    let rows = [vec![field("AA", "tanaka")], vec![field("AA", "sato")]];

    let csv = build_merge_csv(&frames, &rows, CsvEncoding::Utf8).unwrap();

    assert_eq!(String::from_utf8(csv).unwrap(), "tanaka\r\nsato\r\n");
}

#[test]
fn test_utf8_encoding_preserves_japanese_bytes() {
    let frames = [frame("A1", "名前")];
    let rows = [vec![field("名前", "田中")]];

    let csv = build_merge_csv(&frames, &rows, CsvEncoding::Utf8).unwrap();

    assert_eq!(String::from_utf8(csv).unwrap(), "田中\r\n");
}

#[test]
fn test_shift_jis_encoding_round_trips_japanese() {
    let frames = [frame("A1", "名前")];
    let rows = [vec![field("名前", "田中")]];

    let csv = build_merge_csv(&frames, &rows, CsvEncoding::ShiftJis).unwrap();
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(&csv);

    assert!(!had_errors);
    assert_eq!(decoded, "田中\r\n");
}

// ---------------------------------------------------------------------------
// expand_serial
// ---------------------------------------------------------------------------

#[test]
fn test_expand_serial_count_zero_yields_empty() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: 1,
        count: 0,
        step: 1,
        pad: 3,
    };

    assert!(expand_serial(&spec).is_empty());
}

#[test]
fn test_expand_serial_normal_sequence() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: 1,
        count: 3,
        step: 1,
        pad: 3,
    };

    let values: Vec<String> = expand_serial(&spec).into_iter().map(|f| f.value).collect();

    assert_eq!(values, vec!["001", "002", "003"]);
}

#[test]
fn test_expand_serial_negative_step() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: 10,
        count: 3,
        step: -2,
        pad: 2,
    };

    let values: Vec<String> = expand_serial(&spec).into_iter().map(|f| f.value).collect();

    assert_eq!(values, vec!["10", "08", "06"]);
}

#[test]
fn test_expand_serial_pad_zero_means_no_padding() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: 1,
        count: 2,
        step: 1,
        pad: 0,
    };

    let values: Vec<String> = expand_serial(&spec).into_iter().map(|f| f.value).collect();

    assert_eq!(values, vec!["1", "2"]);
}

#[test]
fn test_expand_serial_pad_overflow_is_not_truncated() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: 1000,
        count: 1,
        step: 1,
        pad: 2,
    };

    let values: Vec<String> = expand_serial(&spec).into_iter().map(|f| f.value).collect();

    assert_eq!(values, vec!["1000"]);
}

#[test]
fn test_expand_serial_negative_start_keeps_sign() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: -5,
        count: 2,
        step: 1,
        pad: 3,
    };

    let values: Vec<String> = expand_serial(&spec).into_iter().map(|f| f.value).collect();

    assert_eq!(values, vec!["-005", "-004"]);
}

#[test]
fn test_expand_serial_field_title_matches_spec() {
    let spec = SerialSpec {
        title: "Bち".to_owned(),
        start: 1,
        count: 1,
        step: 1,
        pad: 0,
    };

    let fields = expand_serial(&spec);

    assert_eq!(fields.first().map(|f| f.title.as_str()), Some("Bち"));
}
