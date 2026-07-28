//! Suricate Panels

use std::collections::HashMap;

use bladvak::{BladvakApp, File, app::BladvakPanel, eframe::egui};
use ged_io::types::family::Family;
use ged_io::types::individual::Individual;

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
                if let Some(family) = document.data.families.get(&one_family.xref) {
                    ui.label(format!("Family {}", one_family.xref));
                    ui_family(xref, family, &document.data.individuals, ui);
                } else {
                    ui.label("Family not found");
                }
            }
        }
    }
}

/// Display a family
fn ui_family(
    current_xref: &str,
    family: &Family,
    individuals: &HashMap<String, Individual>,
    ui: &mut egui::Ui,
) {
    let partner = if let Some(ref ind1) = family.individual1
        && ind1 != current_xref
    {
        Some(ind1)
    } else if let Some(ref ind2) = family.individual2
        && ind2 != current_xref
    {
        Some(ind2)
    } else {
        None
    };

    if let Some(ind_xref) = partner {
        if let Some(ind) = individuals.get(ind_xref) {
            ui.label(format!("Partner: {ind}"));
        } else {
            ui.label(format!("Partner: {ind_xref}"));
        }
    } else {
        ui.label("(No partners)");
    }

    /*
        if !self.children.is_empty() {
            write!(f, " [{} child(ren)]", self.children.len())?;
        }
        let mut marriage_date: Option<&str> = None;
        let mut engagement_date: Option<&str> = None;
        let mut separated_date: Option<&str> = None;
        let mut divorce_date: Option<&str> = None;
        let mut annulment_date: Option<&str> = None;

        for event in &self.events {
            match event.event {
                crate::types::event::Event::Marriage if marriage_date.is_none() => {
                    marriage_date = event.date.as_ref().and_then(|d| d.value.as_deref());
                }
                crate::types::event::Event::Engagement if engagement_date.is_none() => {
                    engagement_date = event.date.as_ref().and_then(|d| d.value.as_deref());
                }
                crate::types::event::Event::Separated if separated_date.is_none() => {
                    separated_date = event.date.as_ref().and_then(|d| d.value.as_deref());
                }
                crate::types::event::Event::Divorce if divorce_date.is_none() => {
                    divorce_date = event.date.as_ref().and_then(|d| d.value.as_deref());
                }
                crate::types::event::Event::Annulment if annulment_date.is_none() => {
                    annulment_date = event.date.as_ref().and_then(|d| d.value.as_deref());
                }
                _ => {}
            }
        }

        if let Some(date) = marriage_date {
            write!(f, ", m. {date}")?;
        } else if let Some(date) = engagement_date {
            write!(f, ", rel. {date}")?;
        } else if let Some(date) = separated_date {
            write!(f, ", sep. {date}")?;
        } else if let Some(date) = divorce_date {
            write!(f, ", div. {date}")?;
        } else if let Some(date) = annulment_date {
            write!(f, ", anul. {date}")?;
        }
    */
}
