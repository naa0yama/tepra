//! Numeric enum constants mirroring the TEPRA Creator `WebAPI` SDK constants.
//!
//! Wire values follow `tepraprint.js` exactly. Each enum implements
//! `TryFrom<u32>` / `Into<u32>` for transparent serde conversion.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Macro: generate From<u32>/TryFrom<u32>/Into<u32> for plain discriminant enums
// ---------------------------------------------------------------------------

macro_rules! impl_u32_conv {
    ($ty:ident, $($variant:ident => $val:expr),+ $(,)?) => {
        impl TryFrom<u32> for $ty {
            type Error = u32;
            fn try_from(v: u32) -> Result<Self, u32> {
                match v {
                    $($val => Ok(Self::$variant),)+
                    other => Err(other),
                }
            }
        }
        impl From<$ty> for u32 {
            fn from(v: $ty) -> u32 {
                match v {
                    $($ty::$variant => $val,)+
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// TepraPrintError  §3.1.1
// ---------------------------------------------------------------------------

/// Creator API error code returned in `{ "errcode": N }` error bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum CreatorError {
    Success = 0,
    PrinterNotFound = 1,
    FileNotSupport = 2,
    FileNotFound = 3,
    InvalidParameter = 4,
    PrintJobNotFound = 5,
    PrinterAccessError = 100,
    PrintStartError = 101,
    PrintJobAccessError = 200,
    WebapiRequestError = 201,
    WebapiInternalError = 202,
    PrintModuleExecError = 203,
}

impl_u32_conv!(
    CreatorError,
    Success => 0,
    PrinterNotFound => 1,
    FileNotSupport => 2,
    FileNotFound => 3,
    InvalidParameter => 4,
    PrintJobNotFound => 5,
    PrinterAccessError => 100,
    PrintStartError => 101,
    PrintJobAccessError => 200,
    WebapiRequestError => 201,
    WebapiInternalError => 202,
    PrintModuleExecError => 203,
);

// ---------------------------------------------------------------------------
// TepraPrintTapeID  §3.1.2
// ---------------------------------------------------------------------------

/// Tape identifier values used in print parameters and printer info responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum TapeId {
    _04Mm = 274,
    _06Mm = 259,
    _09Mm = 260,
    _12Mm = 261,
    _18Mm = 262,
    _24Mm = 263,
    _36Mm = 264,
    _24MmCable = 275,
    _36MmCable = 276,
    _24MmIndex = 277,
    _36MmLabel1 = 299,
    _50Mm = 309,
    _100Mm = 310,
    _100MmLabel = 311,
    DcTurntell01 = 1559,
    DcTurntell02 = 1560,
    DcTurntell03 = 1561,
    DcTurntell04 = 1562,
    DcSelflami01 = 1659,
    DcSelflami02 = 1660,
    DcSelflami03 = 1661,
    DcSelflami04 = 1662,
}

impl_u32_conv!(
    TapeId,
    _04Mm => 274,
    _06Mm => 259,
    _09Mm => 260,
    _12Mm => 261,
    _18Mm => 262,
    _24Mm => 263,
    _36Mm => 264,
    _24MmCable => 275,
    _36MmCable => 276,
    _24MmIndex => 277,
    _36MmLabel1 => 299,
    _50Mm => 309,
    _100Mm => 310,
    _100MmLabel => 311,
    DcTurntell01 => 1559,
    DcTurntell02 => 1560,
    DcTurntell03 => 1561,
    DcTurntell04 => 1562,
    DcSelflami01 => 1659,
    DcSelflami02 => 1660,
    DcSelflami03 => 1661,
    DcSelflami04 => 1662,
);

// ---------------------------------------------------------------------------
// TepraPrintTapeCut  §3.1.3
// Wire values sent to REST API differ from JS SDK logical values.
// JS logical: EACH_LABEL=0, AFTER_JOB=1, NOT_CUT=2
// REST wire:  EACH_LABEL=2, AFTER_JOB=3, NOT_CUT=1
// ---------------------------------------------------------------------------

/// Tape cut mode — REST wire values (not JS SDK logical values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum TapeCutWire {
    /// テープカットしない (wire value 1)
    NotCut = 1,
    /// ラベル毎にテープカットする (wire value 2)
    EachLabel = 2,
    /// 印刷JOB毎にテープカットする (wire value 3)
    AfterJob = 3,
}

impl_u32_conv!(
    TapeCutWire,
    NotCut => 1,
    EachLabel => 2,
    AfterJob => 3,
);

// ---------------------------------------------------------------------------
// TepraPrintPrintSpeed  §3.1.4
// Wire values: HIGH=1, LOW=2, MIDDLE=3 (JS logical: HIGH=0, LOW=1, MIDDLE=2)
// ---------------------------------------------------------------------------

/// Print speed — REST wire values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum PrintSpeedWire {
    High = 1,
    Low = 2,
    Middle = 3,
}

impl_u32_conv!(
    PrintSpeedWire,
    High => 1,
    Low => 2,
    Middle => 3,
);

// ---------------------------------------------------------------------------
// TepraPrintTapeKind  §3.1.5
// Uses i32 because UNKNOWN = -1.
// ---------------------------------------------------------------------------

/// Tape material/kind returned in `lwstatus` response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub enum TapeKind {
    Normal = 0,
    Transfer = 1,
    Cable = 16,
    Index = 17,
    Braille = 64,
    Olefin = 80,
    ThermalPaper = 81,
    DieCutCircle = 96,
    DirCutEllipse = 97,
    DieCutRoundedCorners = 98,
    DieCutReserved1 = 99,
    DirCutReserved4 = 102,
    Hst = 112,
    Vinyl = 128,
    Cleaning = 144,
    EquipmentManagement = 145,
    Ribbon = 146,
    Magnet = 147,
    LuminousLight = 148,
    QualityPaper = 149,
    Iron = 150,
    BrPet = 201,
    Unknown = -1,
}

impl TryFrom<i32> for TapeKind {
    type Error = i32;
    fn try_from(v: i32) -> Result<Self, i32> {
        match v {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Transfer),
            16 => Ok(Self::Cable),
            17 => Ok(Self::Index),
            64 => Ok(Self::Braille),
            80 => Ok(Self::Olefin),
            81 => Ok(Self::ThermalPaper),
            96 => Ok(Self::DieCutCircle),
            97 => Ok(Self::DirCutEllipse),
            98 => Ok(Self::DieCutRoundedCorners),
            99 => Ok(Self::DieCutReserved1),
            102 => Ok(Self::DirCutReserved4),
            112 => Ok(Self::Hst),
            128 => Ok(Self::Vinyl),
            144 => Ok(Self::Cleaning),
            145 => Ok(Self::EquipmentManagement),
            146 => Ok(Self::Ribbon),
            147 => Ok(Self::Magnet),
            148 => Ok(Self::LuminousLight),
            149 => Ok(Self::QualityPaper),
            150 => Ok(Self::Iron),
            201 => Ok(Self::BrPet),
            -1 => Ok(Self::Unknown),
            other => Err(other),
        }
    }
}

impl From<TapeKind> for i32 {
    fn from(v: TapeKind) -> Self {
        match v {
            TapeKind::Normal => 0,
            TapeKind::Transfer => 1,
            TapeKind::Cable => 16,
            TapeKind::Index => 17,
            TapeKind::Braille => 64,
            TapeKind::Olefin => 80,
            TapeKind::ThermalPaper => 81,
            TapeKind::DieCutCircle => 96,
            TapeKind::DirCutEllipse => 97,
            TapeKind::DieCutRoundedCorners => 98,
            TapeKind::DieCutReserved1 => 99,
            TapeKind::DirCutReserved4 => 102,
            TapeKind::Hst => 112,
            TapeKind::Vinyl => 128,
            TapeKind::Cleaning => 144,
            TapeKind::EquipmentManagement => 145,
            TapeKind::Ribbon => 146,
            TapeKind::Magnet => 147,
            TapeKind::LuminousLight => 148,
            TapeKind::QualityPaper => 149,
            TapeKind::Iron => 150,
            TapeKind::BrPet => 201,
            TapeKind::Unknown => -1,
        }
    }
}

// ---------------------------------------------------------------------------
// TepraPrintStatusError  §3.1.6
// Uses u32 (hex values).
// ---------------------------------------------------------------------------

/// Device error code returned in `lwstatus.error` and `job/progress.statusError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
#[repr(u32)]
pub enum StatusError {
    NoError = 0x00,
    CutterError = 0x01,
    NoTapeCartridge = 0x06,
    HeadOverHeated = 0x15,
    PrinterCancel = 0x20,
    CoverOpen = 0x21,
    LowVoltage = 0x22,
    PowerOffCancel = 0x23,
    TapeEjectError = 0x24,
    TapeFeedError = 0x30,
    InkRibbonSlack = 0x40,
    InkRibbonShort = 0x41,
    TapeEnd = 0x42,
    CutLabelError = 0x43,
    TemperatureError = 0x44,
    InsufficientParameters = 0x45,
    HalfCutterBladeNotSet = 0x50,
    FullCutterBladeNotSet = 0x51,
    HalfCutterBladeOff = 0x52,
    FullCutterBladeOff = 0x53,
    WinderCoverOpen = 0x54,
    VinylTapeTemperatureError = 0x55,
    WinderError = 0x56,
    HalfCutAllCut = 0x57,
    BigrollRecognitionAbnormality = 0x58,
    BigrollNonCompliant = 0x59,
    StopPrintingByAutoPowerOff = 0x5c,
    StopPrintingByPowerSupplyChange = 0x5d,
    WinderSet = 0x5e,
    WinderNotSet = 0x5f,
    WinderHalfCutAllCut = 0x60,
    FirmwareUpdating = 0xffff_fffb,
    DeviceUsing = 0xffff_fffc,
    UnknownError = 0xffff_ffff,
}

impl_u32_conv!(
    StatusError,
    NoError => 0x00,
    CutterError => 0x01,
    NoTapeCartridge => 0x06,
    HeadOverHeated => 0x15,
    PrinterCancel => 0x20,
    CoverOpen => 0x21,
    LowVoltage => 0x22,
    PowerOffCancel => 0x23,
    TapeEjectError => 0x24,
    TapeFeedError => 0x30,
    InkRibbonSlack => 0x40,
    InkRibbonShort => 0x41,
    TapeEnd => 0x42,
    CutLabelError => 0x43,
    TemperatureError => 0x44,
    InsufficientParameters => 0x45,
    HalfCutterBladeNotSet => 0x50,
    FullCutterBladeNotSet => 0x51,
    HalfCutterBladeOff => 0x52,
    FullCutterBladeOff => 0x53,
    WinderCoverOpen => 0x54,
    VinylTapeTemperatureError => 0x55,
    WinderError => 0x56,
    HalfCutAllCut => 0x57,
    BigrollRecognitionAbnormality => 0x58,
    BigrollNonCompliant => 0x59,
    StopPrintingByAutoPowerOff => 0x5c,
    StopPrintingByPowerSupplyChange => 0x5d,
    WinderSet => 0x5e,
    WinderNotSet => 0x5f,
    WinderHalfCutAllCut => 0x60,
    FirmwareUpdating => 0xffff_fffb,
    DeviceUsing => 0xffff_fffc,
    UnknownError => 0xffff_ffff,
);

// ---------------------------------------------------------------------------
// TepraPrintImportFrameAttribute  §3.1.7
// ---------------------------------------------------------------------------

/// Template import frame attribute type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum ImportFrameAttribute {
    Text = 0,
    Image = 1,
    Jan8 = 2,
    Jan13 = 3,
    Code39 = 4,
    Code128 = 5,
    UpcA = 6,
    UpcE = 7,
    Nw7 = 8,
    Itf = 10,
    Custombar = 11,
    Ean128 = 12,
    Ean128Butuuryu = 13,
    QrCode = 14,
    Gs1Omni = 15,
    Gs1Truncated = 16,
    Gs1Stacked = 17,
    Gs1StackedOmni = 18,
    Gs1Limited = 19,
    Gs1Expanded = 20,
    Gs1ExpandedStacked = 21,
    MaxiCode = 22,
    Pdf417 = 23,
    DataMatrix = 24,
    MicroQrCode = 25,
}

impl_u32_conv!(
    ImportFrameAttribute,
    Text => 0,
    Image => 1,
    Jan8 => 2,
    Jan13 => 3,
    Code39 => 4,
    Code128 => 5,
    UpcA => 6,
    UpcE => 7,
    Nw7 => 8,
    Itf => 10,
    Custombar => 11,
    Ean128 => 12,
    Ean128Butuuryu => 13,
    QrCode => 14,
    Gs1Omni => 15,
    Gs1Truncated => 16,
    Gs1Stacked => 17,
    Gs1StackedOmni => 18,
    Gs1Limited => 19,
    Gs1Expanded => 20,
    Gs1ExpandedStacked => 21,
    MaxiCode => 22,
    Pdf417 => 23,
    DataMatrix => 24,
    MicroQrCode => 25,
);

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn creator_error_round_trip() {
        let e = CreatorError::InvalidParameter;
        let n: u32 = e.into();
        assert_eq!(n, 4);
        assert_eq!(CreatorError::try_from(4u32).unwrap(), e);
    }

    #[test]
    fn tape_id_round_trip() {
        let t = TapeId::_18Mm;
        let n: u32 = t.into();
        assert_eq!(n, 262);
        assert_eq!(TapeId::try_from(262u32).unwrap(), t);
    }

    #[test]
    fn tape_cut_wire_round_trip() {
        assert_eq!(u32::from(TapeCutWire::EachLabel), 2);
        assert_eq!(u32::from(TapeCutWire::AfterJob), 3);
        assert_eq!(u32::from(TapeCutWire::NotCut), 1);
    }

    #[test]
    fn tape_kind_unknown_is_minus_one() {
        let k = TapeKind::Unknown;
        let n: i32 = k.into();
        assert_eq!(n, -1);
        assert_eq!(TapeKind::try_from(-1i32).unwrap(), k);
    }

    #[test]
    fn status_error_round_trip() {
        let e = StatusError::CoverOpen;
        let n: u32 = e.into();
        assert_eq!(n, 0x21);
        assert_eq!(StatusError::try_from(0x21u32).unwrap(), e);
    }

    #[test]
    fn import_frame_attribute_round_trip() {
        let a = ImportFrameAttribute::QrCode;
        let n: u32 = a.into();
        assert_eq!(n, 14);
        assert_eq!(ImportFrameAttribute::try_from(14u32).unwrap(), a);
    }
}
