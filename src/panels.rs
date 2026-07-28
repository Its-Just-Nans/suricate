//! Suricate Panels

use bladvak::{BladvakApp, File, app::BladvakPanel, eframe::egui};

use crate::SuricateApp;
use crate::document::Document;

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
            let (path, mut cursor) = Document::load_default_file();
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

/// Panel for selection
#[derive(Debug)]
pub(crate) struct SelectionPanel;

impl BladvakPanel for SelectionPanel {
    type App = SuricateApp;

    fn name(&self) -> &'static str {
        "Selection"
    }

    fn has_settings(&self) -> bool {
        false
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn ui_settings(
        &self,
        _app: &mut Self::App,
        _ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
    }

    fn ui(
        &self,
        app: &mut Self::App,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        egui::ScrollArea::both().show(ui, |ui| {
            Self::ui_side_panel(ui, app);
        });
    }
}

impl SelectionPanel {
    /// Side panel showing current selection
    fn ui_side_panel(ui: &mut egui::Ui, app: &mut SuricateApp) {
        let Some(document) = app.documents.get_current_doc_mut() else {
            return;
        };
        if let Some(xref) = &document.selected
            && let Some(indi) = document.data.individuals.get(xref)
        {
            ui.label("Selected");
            ui.label(format!("{indi}")).on_hover_ui(|ui| {
                ui.label(format!("{indi:#?}"));
            });
            for one_family in &indi.families {
                ui.separator();
                let resp = if let Some(family) = document.data.families.get(&one_family.xref) {
                    ui.label(format!("{family}"))
                } else {
                    ui.label("Family not found")
                };
                resp.on_hover_ui(|ui| {
                    ui.label(format!("{one_family:#?}"));
                });
            }
        }
    }
}
