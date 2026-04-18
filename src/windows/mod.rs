//! Suricate windows

use crate::SuricateApp;
use bladvak::ErrorManager;
use bladvak::eframe::egui;

mod search_table;

use search_table::SearchTable;

/// File info
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WindowsData {
    /// Search table
    pub(crate) search_table: SearchTable,
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            search_table: SearchTable::new(),
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.search_table.reset();
    }

    /// Ui top bar
    pub(crate) fn ui_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.toggle_value(&mut self.search_table.is_open, "Search Table");
    }
}

impl SuricateApp {
    /// Display windows
    pub(crate) fn ui_windows(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.windows_data
            .search_table
            .ui(&self.data, ui, error_manager);
    }
}
