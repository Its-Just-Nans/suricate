//! Suricate Panels

use std::collections::HashMap;

use bladvak::eframe::egui::CollapsingHeader;
use bladvak::{BladvakApp, File, app::BladvakPanel, eframe::egui};

use crate::SuricateApp;
use crate::document::Document;
use crate::types::{Family, Id, Individual};

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
        let mut new_selected = None;
        if let Some(xref) = &document.selected
            && let Some(indi) = document.data.individuals.get(xref)
        {
            ui.strong(format!("{} - {}", indi.name, indi.id.to_string_pretty()))
                .on_hover_ui(|ui| {
                    ui.label(format!("{indi:#?}"));
                });
            for one_family_id in &indi.spouse_of {
                ui.separator();
                if let Some(family) = document.data.families.get(one_family_id) {
                    ui.label(format!("Family {}", one_family_id.to_string_pretty()));
                    if let Some(new_res) = ui_family(xref, family, &document.data.individuals, ui) {
                        new_selected = Some(new_res);
                    }
                } else {
                    ui.label("Family not found");
                }
            }
            for one_family_id in &indi.child_of {
                ui.separator();
                if let Some(family) = document.data.families.get(one_family_id) {
                    ui.label(format!("Family {}", one_family_id.to_string_pretty()));
                    if let Some(new_res) = ui_family(xref, family, &document.data.individuals, ui) {
                        new_selected = Some(new_res);
                    }
                } else {
                    ui.label("Family not found");
                }
            }
        }
        if let Some(new_selected) = new_selected {
            document.selected = Some(new_selected);
        }
    }
}

/// Display a single individual
#[must_use]
fn display_ind(
    ui: &mut egui::Ui,
    id: &Id,
    individuals: &HashMap<Id, Individual>,
    text: &str,
    is_current: bool,
) -> Option<Id> {
    let mut ret = None;
    if let Some(ind) = individuals.get(id) {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!is_current, |ui| {
                if ui.button(ind.id.to_string_pretty()).clicked() {
                    ret = Some(id.clone());
                }
            });
            let text = format!("{text}{}{}", ind.name, &ind.sex);
            if is_current {
                ui.strong(text);
            } else {
                ui.label(text);
            }
        });
    } else {
        let text = format!("{text}{id}");
        if is_current {
            ui.strong(text);
        } else {
            ui.label(text);
        }
    }
    ret
}

/// Display a family
#[must_use]
fn ui_family(
    current_xref: &Id,
    family: &Family,
    individuals: &HashMap<Id, Individual>,
    ui: &mut egui::Ui,
) -> Option<Id> {
    let mut ret = None;
    if let Some(ind1) = &family.indi1 {
        let is_current = ind1 == current_xref;
        if let Some(res) = display_ind(ui, ind1, individuals, "Partner1: ", is_current) {
            ret = Some(res);
        }
    } else {
        ui.label("No Partner1");
    }
    if let Some(ind2) = &family.indi2 {
        let is_current = ind2 == current_xref;
        if let Some(res) = display_ind(ui, ind2, individuals, "Partner2: ", is_current) {
            ret = Some(res);
        }
    } else {
        ui.label("No Partner2");
    }

    if family.children.is_empty() {
        ui.label("No children");
    } else {
        CollapsingHeader::new(format!("Children ({})", family.children.len()))
            .id_salt(format!("{current_xref}_{}", &family.id))
            .show(ui, |ui| {
                for one_child_xref in &family.children {
                    if let Some(res) = display_ind(
                        ui,
                        one_child_xref,
                        individuals,
                        "Child: ",
                        one_child_xref == current_xref,
                    ) {
                        ret = Some(res);
                    }
                }
            });
    }
    ret
    /*
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
