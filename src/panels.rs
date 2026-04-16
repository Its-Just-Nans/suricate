//! Suricate Panels
use bladvak::{BladvakApp, File, app::BladvakPanel, eframe::egui};

use crate::SuricateApp;

/// Panel for file information
#[derive(Debug)]
pub(crate) struct FileInfo;

impl BladvakPanel for FileInfo {
    type App = SuricateApp;

    fn name(&self) -> &'static str {
        "File infos"
    }

    fn has_settings(&self) -> bool {
        true
    }

    fn has_ui(&self) -> bool {
        false
    }

    fn ui_settings(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        error_manager: &mut bladvak::ErrorManager,
    ) {
        if ui.button("Default file").clicked() {
            use std::io::Read;
            let (path, mut cursor) = SuricateApp::load_default_file();
            let mut data = Vec::new();
            match cursor.read_to_end(&mut data) {
                Ok(_num_read) => {
                    if let Err(e) = app.handle_file(File { path, data }) {
                        error_manager.add_error(e);
                    }
                }
                Err(e) => {
                    error_manager.add_error(e);
                }
            }
        }
    }

    fn ui(
        &self,
        _app: &mut Self::App,
        _ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
    }
}
