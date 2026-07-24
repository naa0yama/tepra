//! Human-readable Japanese labels for tape ID / tape kind wire values.
//!
//! Source of truth: `tmp/tc_webapi/tepraprint.js` `TepraPrintTapeID`
//! (L50-95) and `TepraPrintTapeKind` (L122-169) comments.

use super::enums::{TapeId, TapeKind};

/// Converts a `tapeID` wire value (see [`TapeId`]) into a display label.
///
/// Unknown IDs fall back to `format!("ID:{tape_id}")`.
#[must_use]
pub fn tape_id_label(tape_id: u32) -> String {
    let Ok(id) = TapeId::try_from(tape_id) else {
        return format!("ID:{tape_id}");
    };
    match id {
        TapeId::_04Mm => "4mm",
        TapeId::_06Mm => "6mm",
        TapeId::_09Mm => "9mm",
        TapeId::_12Mm => "12mm",
        TapeId::_18Mm => "18mm",
        TapeId::_24Mm => "24mm",
        TapeId::_36Mm => "36mm",
        TapeId::_24MmCable => "24mm(ｹｰﾌﾞﾙ用)",
        TapeId::_36MmCable => "36mm(ｹｰﾌﾞﾙ用)",
        TapeId::_24MmIndex => "24mm(ｲﾝﾃﾞｯｸｽ用)",
        TapeId::_36MmLabel1 => "カットラベル",
        TapeId::_50Mm => "50mm",
        TapeId::_100Mm => "100mm",
        TapeId::_100MmLabel => "宛名ラベル",
        TapeId::DcTurntell01 => "PANDUIT 回転ﾗﾍﾞﾙ Ø3.0~4.1",
        TapeId::DcTurntell02 => "PANDUIT 回転ﾗﾍﾞﾙ Ø4.1~5.6",
        TapeId::DcTurntell03 => "PANDUIT 回転ﾗﾍﾞﾙ Ø5.6~7.1",
        TapeId::DcTurntell04 => "PANDUIT 回転ﾗﾍﾞﾙ Ø7.1~9.9",
        TapeId::DcSelflami01 => "PANDUIT ｾﾙﾌﾗﾐ Ø2.0~4.1",
        TapeId::DcSelflami02 => "PANDUIT ｾﾙﾌﾗﾐ Ø3.0~7.1",
        TapeId::DcSelflami03 => "PANDUIT ｾﾙﾌﾗﾐ Ø4.1~8.1",
        TapeId::DcSelflami04 => "PANDUIT ｾﾙﾌﾗﾐ Ø6.1~12.2",
    }
    .to_owned()
}

/// Converts a `tapeKind` wire value (see [`TapeKind`]) into a display label.
///
/// Unknown values fall back to `"不明"`.
#[must_use]
pub fn tape_kind_label(tape_kind: i32) -> &'static str {
    let Ok(kind) = TapeKind::try_from(tape_kind) else {
        return "不明";
    };
    match kind {
        TapeKind::Normal => "標準ラベル",
        TapeKind::Transfer => "転写テープ",
        TapeKind::Cable => "ケーブル表示ラベル",
        TapeKind::Index => "インデックスラベル",
        TapeKind::Braille => "点字テープ",
        TapeKind::Olefin => "Grandテープ",
        TapeKind::ThermalPaper => "Grand用宛名ラベル",
        TapeKind::DieCutCircle => "カットラベル丸形",
        TapeKind::DirCutEllipse => "カットラベル楕円",
        TapeKind::DieCutRoundedCorners => "カットラベル角丸",
        TapeKind::DieCutReserved1 => "カットラベル・パンドウィット回転ラベル",
        TapeKind::DirCutReserved4 => "カットラベル・パンドウィットセルフラミネートラベル",
        TapeKind::Hst => "熱収縮チューブ",
        TapeKind::Vinyl => "屋外に強いラベル",
        TapeKind::Cleaning => "クリーニングテープ",
        TapeKind::EquipmentManagement => "備品管理ラベル",
        TapeKind::Ribbon => "りぼん",
        TapeKind::Magnet => "マグネット",
        TapeKind::LuminousLight => "蓄光ラベル",
        TapeKind::QualityPaper => "上質紙ラベル/ クラフトラベル",
        TapeKind::Iron => "アイロンラベル",
        TapeKind::BrPet => "EXロングテープ",
        TapeKind::Unknown => "不明",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tape_id_label_known_values() {
        assert_eq!(tape_id_label(261), "12mm");
        assert_eq!(tape_id_label(263), "24mm");
        assert_eq!(tape_id_label(275), "24mm(ｹｰﾌﾞﾙ用)");
    }

    #[test]
    fn tape_id_label_all_variants() {
        let cases = [
            (274u32, "4mm"),
            (259, "6mm"),
            (260, "9mm"),
            (261, "12mm"),
            (262, "18mm"),
            (263, "24mm"),
            (264, "36mm"),
            (275, "24mm(ｹｰﾌﾞﾙ用)"),
            (276, "36mm(ｹｰﾌﾞﾙ用)"),
            (277, "24mm(ｲﾝﾃﾞｯｸｽ用)"),
            (299, "カットラベル"),
            (309, "50mm"),
            (310, "100mm"),
            (311, "宛名ラベル"),
            (1559, "PANDUIT 回転ﾗﾍﾞﾙ Ø3.0~4.1"),
            (1560, "PANDUIT 回転ﾗﾍﾞﾙ Ø4.1~5.6"),
            (1561, "PANDUIT 回転ﾗﾍﾞﾙ Ø5.6~7.1"),
            (1562, "PANDUIT 回転ﾗﾍﾞﾙ Ø7.1~9.9"),
            (1659, "PANDUIT ｾﾙﾌﾗﾐ Ø2.0~4.1"),
            (1660, "PANDUIT ｾﾙﾌﾗﾐ Ø3.0~7.1"),
            (1661, "PANDUIT ｾﾙﾌﾗﾐ Ø4.1~8.1"),
            (1662, "PANDUIT ｾﾙﾌﾗﾐ Ø6.1~12.2"),
        ];
        for (id, label) in cases {
            assert_eq!(tape_id_label(id), label, "mismatch for tape_id={id}");
        }
    }

    #[test]
    fn tape_id_label_unknown_falls_back_to_numeric() {
        assert_eq!(tape_id_label(0), "ID:0");
        assert_eq!(tape_id_label(999_999), "ID:999999");
    }

    #[test]
    fn tape_kind_label_known_values() {
        assert_eq!(tape_kind_label(0), "標準ラベル");
        assert_eq!(tape_kind_label(16), "ケーブル表示ラベル");
        assert_eq!(tape_kind_label(-1), "不明");
    }

    #[test]
    fn tape_kind_label_all_variants() {
        let cases: &[(i32, &str)] = &[
            (0, "標準ラベル"),
            (1, "転写テープ"),
            (16, "ケーブル表示ラベル"),
            (17, "インデックスラベル"),
            (64, "点字テープ"),
            (80, "Grandテープ"),
            (81, "Grand用宛名ラベル"),
            (96, "カットラベル丸形"),
            (97, "カットラベル楕円"),
            (98, "カットラベル角丸"),
            (99, "カットラベル・パンドウィット回転ラベル"),
            (102, "カットラベル・パンドウィットセルフラミネートラベル"),
            (112, "熱収縮チューブ"),
            (128, "屋外に強いラベル"),
            (144, "クリーニングテープ"),
            (145, "備品管理ラベル"),
            (146, "りぼん"),
            (147, "マグネット"),
            (148, "蓄光ラベル"),
            (149, "上質紙ラベル/ クラフトラベル"),
            (150, "アイロンラベル"),
            (201, "EXロングテープ"),
            (-1, "不明"),
        ];
        for &(kind, label) in cases {
            assert_eq!(
                tape_kind_label(kind),
                label,
                "mismatch for tape_kind={kind}"
            );
        }
    }

    #[test]
    fn tape_kind_label_unknown_falls_back() {
        assert_eq!(tape_kind_label(9999), "不明");
        assert_eq!(tape_kind_label(-2), "不明");
    }
}
