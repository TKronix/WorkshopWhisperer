#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 
use crate::app::MyApp;

mod steam;
mod parser;
mod spreadsheet;
mod config;
mod mods_table;
mod spreadsheet_section;
mod status_colors;
mod settings_window;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Steam Workshop Whisperer",
        options,
        Box::new(|_cc| {
            Ok::<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync + 'static>>(
                Box::new(MyApp::default()),
            )
        }),
    )
}