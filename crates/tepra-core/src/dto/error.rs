//! Creator API error response shape.

use serde::{Deserialize, Serialize};

/// Error body returned by the Creator `WebAPI` on non-2xx responses.
///
/// Shape: `{ "errcode": <u32> }`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatorErrorBody {
    pub errcode: u32,
}
