//! Catfyn's logic, independent of the window: the Jellyfin API, the
//! screens' state, the player (mpv) and the on-screen keyboard. Everything
//! here is unit-tested without a display or a server.

pub mod env;
pub mod error;
pub mod fin;
pub mod input;
pub mod jellyfin;
pub mod mpv;
pub mod osk;
pub mod process;
pub mod scan;
pub mod util;

pub use error::{Error, Result};
