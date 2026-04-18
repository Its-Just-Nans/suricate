//! Search Table

use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::ErrorManager;
use ged_io::types::individual::Individual;

use crate::app::TreeData;

/// `SearchTable` data
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct SearchTable {
    /// is open
    pub(crate) is_open: bool,
    /// search field
    searching: String,
    /// selected row
    selected: Option<usize>,
    /// reverse the table
    reversed: bool,
}

impl SearchTable {
    /// Create empty `SearchTable`
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            searching: String::new(),
            selected: None,
            reversed: false,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.searching.clear();
    }

    /// Show the ui
    pub(crate) fn ui(
        &mut self,
        data: &TreeData,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Search table")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    ui.text_edit_singleline(&mut self.searching);
                    let row_height = 18.0;
                    let rows: Vec<&Individual> =
                        data.individuals.values().filter(|_u| true).collect();
                    let num_rows = rows.len();
                    let available_height = ui.available_height();
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        //.resizable(self.resizable)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto())
                        .column(
                            Column::remainder()
                                .at_least(40.0)
                                .clip(true)
                                .resizable(true),
                        )
                        .column(Column::auto())
                        .column(Column::remainder())
                        .column(Column::remainder())
                        .min_scrolled_height(0.0)
                        .max_scroll_height(available_height);
                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                egui::Sides::new().show(
                                    ui,
                                    |ui| {
                                        ui.strong("Row");
                                    },
                                    |ui| {
                                        self.reversed ^= ui
                                            .button(if self.reversed { "⬆" } else { "⬇" })
                                            .clicked();
                                    },
                                );
                            });
                            header.col(|ui| {
                                ui.strong("Clipped text");
                            });
                            header.col(|ui| {
                                ui.strong("Expanding content");
                            });
                            header.col(|ui| {
                                ui.strong("Interaction");
                            });
                            header.col(|ui| {
                                ui.strong("Content");
                            });
                        })
                        .body(|body| {
                            body.heterogeneous_rows(
                                (0..num_rows).map(|_x| row_height),
                                |mut row| {
                                    let row_index = if self.reversed {
                                        num_rows - 1 - row.index()
                                    } else {
                                        row.index()
                                    };
                                    let row_data = rows[row_index];
                                    if let Some(selected) = self.selected {
                                        row.set_selected(selected == row_index);
                                    }
                                    //row.set_overline(self.overline && row_index % 7 == 3);

                                    row.col(|ui| {
                                        ui.label(row_index.to_string());
                                    });
                                    row.col(|ui| {
                                        if let Some(xref) = &row_data.xref {
                                            ui.label(xref);
                                        }
                                    });
                                    row.col(|ui| {
                                        ui.label(format!("{row_data:?}"));
                                    });
                                    row.col(|ui| {
                                        let mut checked = false;
                                        ui.checkbox(&mut checked, "Click me");
                                    });
                                    row.col(|ui| {
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                        ui.label("Normal row");
                                    });
                                },
                            );
                        });
                });
            self.is_open = is_open;
        }
    }
}
