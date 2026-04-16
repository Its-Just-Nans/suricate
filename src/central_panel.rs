//! Central panel
use bladvak::eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, UiBuilder};

use crate::SuricateApp;

impl SuricateApp {
    /// Central panel
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut bladvak::ErrorManager,
    ) {
        let _rect = ui.available_rect_before_wrap();
        egui::Scene::new()
            .max_inner_size([350.0, 1000.0])
            .zoom_range(0.1..=50.0)
            .show(ui, &mut self.scene_rect, |ui| {
                let bg_r: egui::Response = ui.response();
                if bg_r.rect.is_finite() {
                    self.grid.draw(&bg_r.rect, ui.painter());
                }
                //ui.label(format!("{:?}", self.data.individuals));
                let node_frame_rect =
                    Rect::from_min_max(Pos2::new(15.0, 15.0), Pos2::new(120.0, 20.0));
                let node_ui = &mut ui.new_child(
                    UiBuilder::new()
                        .max_rect(node_frame_rect)
                        .layout(Layout::top_down(Align::Center))
                        .id_salt("jkhnklj"),
                );

                egui::Frame::NONE.fill(Color32::RED).show(node_ui, |ui| {
                    ui.label("Label with red background");
                });
                // let _response = self.svg_render.show(ui);
                // if response.clicked() {
                //     println!("SVG clicked!");
                // }
            });
    }
}
