//! TEPRA Creator `WebAPI` client — trait, HTTP implementation, and in-process mock.

pub mod mock;
#[allow(clippy::module_name_repetitions)]
pub mod reqwest_client;
pub mod traits;

#[allow(clippy::module_name_repetitions)]
pub use mock::MockTepraClient;
#[allow(clippy::module_name_repetitions)]
pub use reqwest_client::ReqwestTepraClient;
#[allow(clippy::module_name_repetitions)]
pub use traits::TepraClient;
