/// TUI module.
///
/// Sub-modules:
/// * [`app`]     – application state machine and event types.
/// * [`widgets`] – ratatui rendering helpers.
pub mod app;
pub mod widgets;

pub use app::{App, AppEvent};
