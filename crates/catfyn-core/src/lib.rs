//! Catfyn's logic, independent of the window: the Jellyfin API, the
//! screens' state, the player (mpv) and the on-screen keyboard. Everything
//! here is unit-tested without a display or a server.

pub mod env;
pub mod error;
pub mod fin;
pub mod input;
pub mod jellyfin;
pub mod mpv;
pub mod net;
pub mod osk;
pub mod plex;
pub mod process;
pub mod scan;
pub mod upnext;
pub mod util;

pub use error::{Error, Result};
