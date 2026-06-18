//! Suricate windows

use crate::SuricateApp;
use bladvak::ErrorManager;
use bladvak::eframe::egui;

mod search_families;
mod search_table;

use search_families::SearchFamilies;
use search_table::SearchTable;

/// File info
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WindowsData {
    /// Search table
    pub(crate) search_table: SearchTable,
    /// Search families
    pub(crate) search_families: SearchFamilies,
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            search_table: SearchTable::new(),
            search_families: SearchFamilies::new(),
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.search_table.reset();
        self.search_families.reset();
    }

    /// Ui top bar
    pub(crate) fn ui_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.toggle_value(&mut self.search_table.is_open, "Search Table");
        ui.toggle_value(&mut self.search_families.is_open, "Search Families");
    }
}

impl SuricateApp {
    /// Display windows
    pub(crate) fn ui_windows(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        let old_selection = self.selected.clone();
        if let Some(user_selected) =
            self.windows_data
                .search_table
                .ui(&self.data, ui, error_manager)
        {
            if let Some(node) = self
                .nodes
                .iter_mut()
                .find(|one_node| one_node.data.xref == user_selected)
            {
                node.selected = true;
            }
            self.selected = Some(user_selected);
        } else if let Some(user_selected) =
            self.windows_data
                .search_families
                .ui(&self.data, ui, error_manager)
        {
            if let Some(node) = self
                .nodes
                .iter_mut()
                .find(|one_node| one_node.data.xref == user_selected)
            {
                node.selected = true;
            }
            self.selected = Some(user_selected);
        } else if let Some(old_selected) = old_selection
            && let Some(node) = self
                .nodes
                .iter_mut()
                .find(|one_node| one_node.data.xref == old_selected)
        {
            node.selected = true;
        }
    }
}
