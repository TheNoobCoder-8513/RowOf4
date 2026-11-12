// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod controller;
mod game;
mod history;

use std::error::Error;

use controller::GameController;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = App::new()?;

    let _controller = GameController::new(&ui);

    ui.run()?;

    Ok(())
}
