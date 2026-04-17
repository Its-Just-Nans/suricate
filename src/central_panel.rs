//! Central panel
use bladvak::eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, Sense, UiBuilder};

use crate::SuricateApp;
use crate::app::Node;
use ged_io::types::individual::Individual;
use ged_io::types::family::Family;
use ged_io::types::individual::family_link::FamilyLinkType;

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

                for node in &mut self.nodes {
                    let node_rect = egui::Rect::from_center_size(node.pos, node.size);

                    let node_ui = &mut ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(node_rect)
                            .layout(egui::Layout::top_down(egui::Align::Min))
                            .id_salt(node.id)
                            .sense(egui::Sense::click_and_drag()),
                    );

                    egui::Frame::new()
                        .inner_margin(5)
                        .fill(if node.selected {
                            egui::Color32::from_rgb(70, 130, 200)
                        } else {
                            egui::Color32::from_rgb(45, 45, 55)
                        })
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(100, 100, 120),
                        ))
                        .corner_radius(6.0)
                        .show(node_ui, |ui| {
                            // Title bar
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&node.title)
                                        .color(egui::Color32::WHITE)
                                        .strong(),
                                );
                            });
                            ui.separator();
                            // Node content goes here
                            ui.label("Input");
                            ui.label("Output");
                            ui.label(format!("{:?}", node));
                        });
                    let response = ui.interact(
                        node_rect,
                        egui::Id::new(format!("{}-int", node.id.value())),
                        Sense::click_and_drag(),
                    );
                    // Drag to move
                    // if response.dragged() {
                    //     node.pos += response.drag_delta();
                    // }

                    // Click to select (deselect others outside this loop if needed)
                    if response.clicked() {
                        node.selected = !node.selected;
                        if node.selected {
                            self.selected = Some(node.title.clone());
                        }
                    } else {
                        if let Some(nn) = &self.selected {
                            if *nn != node.title {
                                node.selected = false;
                            }
                        } else {
                            node.selected = false;
                        }
                    }
                }
            });
    }

}




use std::collections::HashMap;

const NODE_W: f32 = 180.0;
const NODE_H: f32 = 60.0;
const H_GAP: f32 = 30.0;  // horizontal gap between nodes
const V_GAP: f32 = 80.0;  // vertical gap between generations


pub fn build_family_nodes(
    individuals: &HashMap<String, Individual>,
    families: &HashMap<String, Family>,
) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut placed: HashMap<String, Pos2> = HashMap::new();

    // 1. Find root individuals: those who appear only as Spouse, never as Child
    let children_xrefs: std::collections::HashSet<String> = individuals
        .values()
        .filter_map(|ind| {
            ind.families.iter().find(|fl| fl.family_link_type == FamilyLinkType::Child)
                .map(|_| ind.xref.clone().unwrap())
        })
        .collect();

    let roots: Vec<&Individual> = individuals
        .values()
        .filter(|ind| {
            let xref = ind.xref.as_deref().unwrap_or("");
            !children_xrefs.contains(xref)
        })
        .collect();

    // 2. Place generation by generation using BFS
    let mut generation: Vec<Vec<String>> = Vec::new();
    let mut visited_fams: std::collections::HashSet<String> = Default::default();

    // Generation 0: root individuals grouped by their spouse family
    let root_xrefs: Vec<String> = roots
        .iter()
        .filter_map(|i| i.xref.clone())
        .collect();
    generation.push(root_xrefs.clone());

    // Walk down through families
    let mut current_gen = root_xrefs;
    loop {
        let mut next_gen: Vec<String> = Vec::new();
        for xref in &current_gen {
            if let Some(ind) = individuals.get(xref) {
                for fl in &ind.families {
                    if fl.family_link_type != FamilyLinkType::Spouse { continue; }
                    if visited_fams.contains(&fl.xref) { continue; }
                    visited_fams.insert(fl.xref.clone());

                    if let Some(fam) = families.get(&fl.xref) {
                        for child_xref in &fam.children {
                            if !next_gen.contains(child_xref) {
                                next_gen.push(child_xref.clone());
                            }
                        }
                    }
                }
            }
        }
        if next_gen.is_empty() { break; }
        generation.push(next_gen.clone());
        current_gen = next_gen;
    }

    // 3. Assign positions: center each generation horizontally
    for (gen_idx, members) in generation.iter().enumerate() {
        let total_w = members.len() as f32 * NODE_W
            + (members.len().saturating_sub(1)) as f32 * H_GAP;
        let start_x = -total_w / 2.0;
        let y = gen_idx as f32 * (NODE_H + V_GAP);

        for (i, xref) in members.iter().enumerate() {
            let x = start_x + i as f32 * (NODE_W + H_GAP);
            let pos = Pos2::new(x, y);
            placed.insert(xref.clone(), pos);

            let label = individuals
                .get(xref)
                .and_then(|ind| ind.name.as_ref())
                .map(|n| n.value.clone().unwrap()) // adapt to your Name type
                .unwrap_or_else(|| xref.clone());

            nodes.push(Node { id: egui::Id::new(xref), pos, size: egui::vec2(180.0, 80.0), title: xref.clone(), selected: false});
        }
    }

    // 4. Insert spouse pairs side-by-side within their generation
    // (optional: nudge ind1 and ind2 of each family closer together)
    for fam in families.values() {
        let ind1 = fam.individual1.as_deref();
        let ind2 = fam.individual2.as_deref();
        if let (Some(a), Some(b)) = (ind1, ind2) {
            let a_id = egui::Id::new(a);
            let b_id = egui::Id::new(b);
            if let (Some(pa), Some(pb)) = (placed.get(a), placed.get(b)) {
                // If they ended up on the same row, nudge them adjacent
                if (pa.y - pb.y).abs() < 1.0 {
                    let mid_x = (pa.x + pb.x) / 2.0;
                    if let Some(node) = nodes.iter_mut().find(|n| n.id == a_id) {

                        node.pos.x = mid_x - (NODE_W + H_GAP) / 2.0;
                    }
                    if let Some(node) = nodes.iter_mut().find(|n| n.id == b_id) {
                        node.pos.x = mid_x + (NODE_W + H_GAP) / 2.0;
                    }
                }
            }
        }
    }

    nodes
}
