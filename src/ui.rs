use crate::scene::Scene;

use bevy::prelude::*;
use bevy_egui::egui::{self, TextureId};
use egui_dock::Tree;

#[derive(Debug, Clone)]
pub enum Tabs {
    Viewport(TextureId),
    Settings,
    Scene,
}

#[derive(Deref, DerefMut)]
pub struct DockTree(pub Tree<Tabs>);

#[derive(Default)]
pub struct TabViewer {
    pub viewport_size: Vec2,
    pub dt: f32,
    pub render_dt: f32,
    pub scene: Scene,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Tabs;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tabs::Viewport(texture_id) => {
                self.viewport_size = Vec2::from_array(ui.available_size().into());
                ui.image(*texture_id, ui.available_size());
            }
            Tabs::Settings => {
                ui.label(format!("Viewport size: {:?}", self.viewport_size));
                ui.label(format!("dt: {}ms", self.dt * 1000.0));
                ui.label(format!("render dt: {}ms", self.render_dt * 1000.0));
            }
            Tabs::Scene => {
                ui.label("Spheres");
                ui.separator();
                for (i, sphere) in self.scene.spheres.iter_mut().enumerate() {
                    egui::Grid::new(format!("sphere_grid_{i}"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Position");
                            drag_vec3(ui, &mut sphere.position, 0.1);
                            ui.end_row();

                            ui.label("Radius");
                            drag_f32(ui, &mut sphere.radius, 0.025);
                            ui.end_row();

                            ui.label("Albedo");
                            drag_vec3_color(ui, &mut sphere.albedo, 0.1);
                            ui.end_row();
                        });
                    ui.separator();
                }
            }
        };
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tabs::Viewport(_) => "Viewport",
            Tabs::Settings => "Settings",
            Tabs::Scene => "Scene",
        }
        .into()
    }
}

fn drag_vec3(ui: &mut egui::Ui, value: &mut Vec3, speed: f32) {
    ui.columns(3, |ui| {
        ui[0].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.x).speed(speed));
        ui[1].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.y).speed(speed));
        ui[2].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.z).speed(speed));
    });
}

fn drag_f32(ui: &mut egui::Ui, value: &mut f32, speed: f32) {
    ui.columns(1, |ui| {
        ui[0].add_sized([0.0, 0.0], egui::DragValue::new(value).speed(speed));
    });
}

fn drag_vec3_color(ui: &mut egui::Ui, value: &mut Vec3, speed: f32) {
    ui.columns(4, |ui| {
        ui[0].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.x).speed(speed));
        ui[1].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.y).speed(speed));
        ui[2].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.z).speed(speed));

        let mut color = value.to_array();
        ui[3].color_edit_button_rgb(&mut color);
        *value = Vec3::from_array(color);
    });
}
