/*!
This is a toy todo list application I have written to explore Rust.
*/
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_extern_crates)]

use anyhow::Result;
use cli_log::{debug, init_cli_log, warn};
use sift_core::save_name;
use sift_tui::run;

fn main() -> Result<()> {
    init_cli_log!();
    let save_name = save_name();
    debug!("save name {}", save_name.display());

    run(&save_name)?;

    Ok(())
}
