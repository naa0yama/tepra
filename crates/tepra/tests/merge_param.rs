//! Unit tests for `merge_print_parameter`.

use tepra::merge::{MergePrintOverrides, merge_print_parameter};

#[test]
fn test_defaults_match_sdk_wire_mapping() {
    let param = merge_print_parameter(&MergePrintOverrides::default());

    assert_eq!(param.copies, 1);
    assert_eq!(param.tape_cut, 2);
    assert_eq!(param.half_cut, 2);
    assert_eq!(param.print_speed, 1);
    assert_eq!(param.density.mode, 1);
    assert_eq!(param.density.value, 0);
    assert_eq!(param.tape_id, 262);
    assert_eq!(param.priority_cut_setting, 1);
    assert_eq!(param.half_cut_separate, 1);
    assert_eq!(param.margin_left_right, 0);
    assert_eq!(param.display_tape_width, 2);
    assert_eq!(param.error_message.mode, 2);
    assert_eq!(param.error_message.file_output, 0);
    assert_eq!(param.error_message.file_path, "");
    assert_eq!(param.display_transfer_tape, 2);
    assert_eq!(param.display_print_setting, 2);
    assert_eq!(param.cut_title, 0);
    assert_eq!(param.kana_zen, 0);
    assert_eq!(param.display_print_preview, 1);
    assert_eq!(param.stretch_image, 0);
}

#[test]
fn test_copies_override_changes_only_copies() {
    let overrides = MergePrintOverrides {
        copies: Some(5),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.copies, 5);
    assert_eq!(param.tape_cut, 2);
}

#[test]
fn test_density_override_sets_value_and_keeps_mode_one() {
    let overrides = MergePrintOverrides {
        density: Some(3),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.density.mode, 1);
    assert_eq!(param.density.value, 3);
}

#[test]
fn test_tape_cut_override_changes_only_tape_cut() {
    let overrides = MergePrintOverrides {
        tape_cut: Some(3),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.tape_cut, 3);
    assert_eq!(param.copies, 1);
}

#[test]
fn test_half_cut_override_changes_only_half_cut() {
    let overrides = MergePrintOverrides {
        half_cut: Some(1),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.half_cut, 1);
    assert_eq!(param.tape_cut, 2);
}

#[test]
fn test_half_cut_separate_override_changes_only_half_cut_separate() {
    let overrides = MergePrintOverrides {
        half_cut_separate: Some(2),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.half_cut_separate, 2);
    assert_eq!(param.half_cut, 2);
}

#[test]
fn test_print_speed_override_changes_only_print_speed() {
    let overrides = MergePrintOverrides {
        print_speed: Some(2),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.print_speed, 2);
    assert_eq!(param.copies, 1);
}

#[test]
fn test_margin_left_right_override_changes_only_margin() {
    let overrides = MergePrintOverrides {
        margin_left_right: Some(10),
        ..MergePrintOverrides::default()
    };
    let param = merge_print_parameter(&overrides);

    assert_eq!(param.margin_left_right, 10);
    assert_eq!(param.copies, 1);
}
