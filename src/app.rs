//! Suricate App

use bladvak::egui_extras;
use bladvak::utils::Documents;
use bladvak::{
    File,
    app::BladvakApp,
    errors::{AppError, ErrorManager},
    utils::grid::Grid,
};
use bladvak::{
    eframe::{CreationContext, egui},
    utils::is_native,
};
use std::fmt::Debug;

use crate::document::Document;
use crate::panels::{FileInfo, SelectionPanel};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct SuricateApp {
    /// documents
    #[serde(skip)]
    pub(crate) documents: Documents<Document>,
    /// Grid options
    pub(crate) grid: Grid,
}

impl Default for SuricateApp {
    fn default() -> Self {
        let mut documents = Documents::default();
        documents.push(Document::default());
        Self {
            grid: Grid::default(),
            documents,
        }
    }
}

impl BladvakApp<'_> for SuricateApp {
    fn panel_list(&self) -> Vec<Box<dyn bladvak::app::BladvakPanel<App = Self>>> {
        vec![Box::new(FileInfo), Box::new(SelectionPanel)]
    }

    fn is_side_panel(&self) -> bool {
        self.documents.is_some()
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        let document = Document::try_new(file.path, &file.data)?;
        self.documents.push(document);
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.separator();
        if let Some(document) = self.documents.get_current_doc_mut() {
            ui.menu_button("Windows", |ui| {
                document.windows_data.ui_top_bar(ui);
            });
            ui.separator();
        }
        self.documents.show_file_list(ui);
    }

    fn menu_file(&mut self, _ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        //self.app_menu_file(ui, error_manager);
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager);
        self.ui_windows(ui, error_manager);
    }

    fn name() -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn repo_url() -> String {
        "https://github.com/Its-Just-Nans/suricate".to_string()
    }

    fn icon() -> &'static [u8] {
        &include_bytes!("../assets/icon-256.png")[..]
    }

    fn try_new_with_args(
        saved_state: Self,
        cc: &CreationContext<'_>,
        args: &[String],
        error_manager: &mut ErrorManager,
    ) -> Result<Self, AppError> {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if is_native() && args.len() > 1 {
            use std::fs;
            let mut app = saved_state;
            app.documents.clear();
            for one_path in &args[1..] {
                let absolute_path = match fs::canonicalize(one_path) {
                    Ok(path) => path,
                    Err(e) => {
                        error_manager.add_error(format!("Unable to access path '{one_path}': {e}"));
                        continue;
                    }
                };
                let bytes = match std::fs::read(&absolute_path) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error_manager.add_error(format!(
                            "Unable to read file '{}': {e}",
                            absolute_path.display()
                        ));
                        continue;
                    }
                };
                let document = match Document::try_new(absolute_path, &bytes) {
                    Ok(d) => d,
                    Err(e) => {
                        error_manager.add_error(e);
                        continue;
                    }
                };
                app.documents.push(document);
            }
            Ok(app)
        } else {
            Ok(saved_state)
        }
    }
}
