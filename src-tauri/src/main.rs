// Prevent console window in addition to the Slint window in Windows release builds when, e.g., launching the app from file manager
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aeromail::run;

fn main() {
    run();
}
