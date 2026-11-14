//! Optional informational logs. Warnings and errors bypass this switch.
use core::sync::atomic::{AtomicBool,Ordering};
static ENABLED:AtomicBool=AtomicBool::new(false);
pub fn enabled()->bool{ENABLED.load(Ordering::Relaxed)}
pub fn set(enabled:bool){ENABLED.store(enabled,Ordering::Relaxed);}
#[macro_export]
macro_rules! log_info{($($arg:tt)*)=>{if $crate::logging::enabled(){forge::log::prelude::info!($($arg)*);}}}
