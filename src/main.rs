/*!
This is a toy todo list application I have written to explore Rust.
*/
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args)]
#![deny(unused_extern_crates)]

use anyhow::Result;
use cli_log::{debug, init_cli_log, warn};

fn main() -> Result<()> {
    init_cli_log!();
    let save_name = sift::save_name();
    debug!("save name {}", save_name.display());

    sift::run(&save_name)?;

    Ok(())
}
