//! TEPRA Creator `WebAPI` client — trait and in-process mock.

pub mod mock;
pub mod traits;

#[allow(clippy::module_name_repetitions)]
pub use mock::MockTepraClient;
#[allow(clippy::module_name_repetitions)]
pub use traits::TepraClient;
