//! Round-trip JSON ↔ DTO tests for the TEPRA Creator `WebAPI` DTOs.
#![allow(missing_docs)]

use tepra_core::dto::{
    error::CreatorErrorBody,
    job::{
        FilePayload, JobControlRequest, JobInfoResponse, JobProgressResponse, PrintFiles,
        PrintRequest, PrintResponse,
    },
    printer::{
        AutoselectResponse, LwStatusResponse, OnlineStatusResponse, PrinterInfoResponse,
        PrinterListItem, VersionResponse,
    },
    template::{GetMarginRequest, GetMarginResponse, ImportFrameItem, ImportFrameRequest},
};

macro_rules! roundtrip {
    ($ty:ty, $path:expr) => {{
        let json = include_str!($path);
        let a: $ty = serde_json::from_str(json).expect(concat!("deser ", $path));
        let mid = serde_json::to_string(&a).expect(concat!("ser ", $path));
        let b: $ty = serde_json::from_str(&mid).expect(concat!("re-deser ", $path));
        assert_eq!(a, b, "round-trip mismatch for {}", $path);
    }};
}

#[test]
fn printer_list_res() {
    roundtrip!(Vec<PrinterListItem>, "fixtures/dto/printer_list_res.json");
}

#[test]
fn version_res() {
    roundtrip!(VersionResponse, "fixtures/dto/version_res.json");
}

#[test]
fn autoselect_res() {
    roundtrip!(AutoselectResponse, "fixtures/dto/autoselect_res.json");
}

#[test]
fn printer_info_res() {
    roundtrip!(PrinterInfoResponse, "fixtures/dto/printer_info_res.json");
}

#[test]
fn online_status_res() {
    roundtrip!(OnlineStatusResponse, "fixtures/dto/online_status_res.json");
}

#[test]
fn lw_status_res_with_options() {
    roundtrip!(LwStatusResponse, "fixtures/dto/lw_status_res.json");
}

#[test]
fn lw_status_res_no_options() {
    roundtrip!(LwStatusResponse, "fixtures/dto/lw_status_no_opt_res.json");
}

#[test]
fn print_req() {
    roundtrip!(PrintRequest, "fixtures/dto/print_req.json");
}

#[test]
fn print_res() {
    roundtrip!(PrintResponse, "fixtures/dto/print_res.json");
}

#[test]
fn job_progress_res() {
    roundtrip!(JobProgressResponse, "fixtures/dto/job_progress_res.json");
}

#[test]
fn job_info_res() {
    roundtrip!(JobInfoResponse, "fixtures/dto/job_info_res.json");
}

#[test]
fn job_control_req() {
    roundtrip!(JobControlRequest, "fixtures/dto/job_control_req.json");
}

#[test]
fn import_frame_req() {
    roundtrip!(ImportFrameRequest, "fixtures/dto/import_frame_req.json");
}

#[test]
fn import_frame_res() {
    roundtrip!(Vec<ImportFrameItem>, "fixtures/dto/import_frame_res.json");
}

#[test]
fn get_margin_req_with_template() {
    roundtrip!(GetMarginRequest, "fixtures/dto/get_margin_req.json");
}

#[test]
fn get_margin_req_no_template() {
    roundtrip!(
        GetMarginRequest,
        "fixtures/dto/get_margin_no_template_req.json"
    );
}

#[test]
fn get_margin_res() {
    roundtrip!(GetMarginResponse, "fixtures/dto/get_margin_res.json");
}

#[test]
fn error_res() {
    roundtrip!(CreatorErrorBody, "fixtures/dto/error_res.json");
}

#[test]
fn print_files_optional_fields_omitted_on_serialize() {
    let files = PrintFiles {
        template_file: Some(FilePayload {
            file_name: "t.lbx".into(),
            base64_str: "dA==".into(),
        }),
        csv_file: None,
        image_file: None,
    };
    let json = serde_json::to_string(&files).expect("serialize");
    assert!(!json.contains("csvFile"), "None fields must be omitted");
    assert!(!json.contains("imageFile"), "None fields must be omitted");
    let back: PrintFiles = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(files, back);
}

#[cfg(feature = "schema")]
mod schema {
    use tepra_core::dto::printer::PrinterListItem;
    use utoipa::PartialSchema;

    #[test]
    fn printer_list_item_schema_uses_camel_case_field_name() {
        let schema = PrinterListItem::schema();
        let json = serde_json::to_value(&schema).expect("serialize schema");
        let properties = json
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .expect("schema must describe an object with properties");
        assert!(
            properties.contains_key("printerName"),
            "expected camelCase field `printerName`, got {json}"
        );
    }
}
