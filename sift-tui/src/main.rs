/*!
This is a toy todo list application I have written to explore Rust.
*/
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use anyhow::Result;
use cli_log::{debug, init_cli_log, warn};
use sift_core::save_name;

mod keys;
mod screen;
mod terminal_input;
mod toplevel;
mod tui;
mod ui_state;

fn main() -> Result<()> {
    init_cli_log!();
    let save_name = save_name();
    debug!("save name {}", save_name.display());

    toplevel::run(&save_name)?;

    Ok(())
}
