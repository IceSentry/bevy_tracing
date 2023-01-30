use std::ops::RangeInclusive;

use crate::{
    camera::ChernoCamera, renderer::Renderer, scene::Scene, Frametimes, ViewportEguiTexture,
    ViewportSize,
};

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextureId},
    EguiContext,
};
use egui_dock::{DockArea, NodeIndex, Style, Tree};

#[derive(Debug, Clone)]
pub enum Tabs {
    Viewport,
    Settings,
    Scene,
}

#[derive(Deref, DerefMut, Resource)]
pub struct DockTree(pub Tree<Tabs>);

pub fn setup_ui(mut commands: Commands) {
    // Setup dock tree to look like this:
    //  __________________________
    // |               | Scene    |
    // |               |          |
    // |   Viewport    |__________|
    // |               | Settings |
    // |_______________|__________|

    let mut tree = Tree::new(vec![Tabs::Viewport]);
    let [_viewport, scene] = tree.split_right(NodeIndex::root(), 0.75, vec![Tabs::Scene]);
    tree.split_below(scene, 0.85, vec![Tabs::Settings]);

    commands.insert_resource(DockTree(tree));
}

#[allow(clippy::too_many_arguments)]
pub fn draw_dock_area(
    mut egui_context: ResMut<EguiContext>,
    mut tree: ResMut<DockTree>,
    time: Res<Time>,
    mut scene: ResMut<Scene>,
    viewport_egui_texture: Res<ViewportEguiTexture>,
    mut viewport_size: ResMut<ViewportSize>,
    render_dt: Res<Frametimes>,
    mut camera: ResMut<ChernoCamera>,
    mut renderer: ResMut<Renderer>,
) {
    let mut tab_viewer = TabViewer {
        viewport_texture: viewport_egui_texture.0,
        viewport_size: &mut viewport_size.0,
        // TODO have diagnostic struct
        dt: time.delta_seconds(),
        frametimes: &render_dt,
        scene: &mut scene,
        camera: &mut camera,
        renderer: &mut renderer,
    };

    DockArea::new(&mut tree)
        .style(Style::from_egui(egui_context.ctx_mut().style().as_ref()))
        .show(egui_context.ctx_mut(), &mut tab_viewer);
}

pub struct TabViewer<'a> {
    pub viewport_texture: TextureId,
    pub viewport_size: &'a mut Vec2,
    pub dt: f32,
    pub frametimes: &'a Frametimes,
    pub scene: &'a mut Scene,
    pub camera: &'a mut ChernoCamera,
    pub renderer: &'a mut Renderer,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Tabs;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let mut reset = false;
        match tab {
            Tabs::Viewport => {
                *self.viewport_size = Vec2::from_array(ui.available_size().into());
                ui.image(self.viewport_texture, ui.available_size());
            }
            Tabs::Scene => {
                ui.heading("Camera");
                egui::Grid::new("camera_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Position");
                        reset |= drag_vec3(ui, &mut self.camera.position, 0.1);
                        ui.end_row();
                    });
                ui.separator();

                ui.heading("Sky");
                egui::Grid::new("sky_grid").num_columns(2).show(ui, |ui| {
                    ui.label("Color");
                    reset |= drag_vec3_color(ui, &mut self.scene.sky_color);
                    ui.end_row();
                });
                ui.separator();

                ui.heading("Lights");
                for (i, light) in self.scene.lights.iter_mut().enumerate() {
                    egui::Grid::new(format!("light_grid_{i}"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Direction");
                            reset |= drag_vec3(ui, &mut light.direction, 0.025);
                            ui.end_row();

                            ui.label("Intensity");
                            reset |= drag_f32_clamp(ui, &mut light.intensity, 0.025, 0..=1);
                            ui.end_row();
                        });
                    ui.separator();
                }

                ui.heading("Materials");
                for (i, material) in self.scene.materials.iter_mut().enumerate() {
                    egui::Grid::new(format!("material_grid_{i}"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Albedo");
                            reset |= drag_vec3_color(ui, &mut material.albedo);
                            ui.end_row();

                            ui.label("Roughness");
                            reset |= drag_f32_clamp(ui, &mut material.roughness, 0.025, 0..=1);
                            ui.end_row();

                            ui.label("Metallic");
                            reset |= drag_f32_clamp(ui, &mut material.metallic, 0.025, 0..=1);
                            ui.end_row();
                        });
                    ui.separator();
                }

                ui.heading("Spheres");
                for (i, sphere) in self.scene.spheres.iter_mut().enumerate() {
                    egui::Grid::new(format!("sphere_grid_{i}"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Position");
                            reset |= drag_vec3(ui, &mut sphere.position, 0.1);
                            ui.end_row();

                            ui.label("Radius");
                            reset |= drag_f32(ui, &mut sphere.radius, 0.025);
                            ui.end_row();

                            ui.label("Material id");
                            reset |= drag_usize(
                                ui,
                                &mut sphere.material_id,
                                1.0,
                                self.scene.materials.len() - 1,
                            );
                            ui.end_row();
                        });
                    ui.separator();
                }
            }
            Tabs::Settings => {
                ui.label(format!(
                    "Viewport size: {}x{} ({} pixels)",
                    self.viewport_size.x,
                    self.viewport_size.y,
                    fmt_usize_separator((self.viewport_size.x * self.viewport_size.y) as usize)
                ));
                ui.label(format!("dt: {:.2}ms", self.dt * 1000.0));
                ui.label(format!(
                    "Render dt: {:.2}ms",
                    self.frametimes.render * 1000.0
                ));
                ui.label(format!(
                    "Image copy dt: {:.2}ms",
                    self.frametimes.image_copy * 1000.0
                ));
                ui.label(format!("Samples: {}", self.renderer.samples));

                ui.horizontal(|ui| {
                    ui.label("Bounces");
                    reset |= drag_u8(ui, &mut self.renderer.bounces, 0.25);
                });

                ui.checkbox(&mut self.renderer.accumulate, "Accumulate");
                reset |= ui.button("Reset").clicked();
            }
        };
        if reset {
            self.renderer.reset_frame_index();
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }
}

fn drag_vec3(ui: &mut egui::Ui, value: &mut Vec3, speed: f32) -> bool {
    let mut changed = false;
    ui.columns(3, |ui| {
        changed |= ui[0]
            .add_sized([0.0, 0.0], egui::DragValue::new(&mut value.x).speed(speed))
            .changed();
        changed |= ui[1]
            .add_sized([0.0, 0.0], egui::DragValue::new(&mut value.y).speed(speed))
            .changed();
        changed |= ui[2]
            .add_sized([0.0, 0.0], egui::DragValue::new(&mut value.z).speed(speed))
            .changed();
    });
    changed
}

fn drag_f32(ui: &mut egui::Ui, value: &mut f32, speed: f32) -> bool {
    let mut changed = false;
    ui.columns(1, |ui| {
        changed = ui[0]
            .add_sized([0.0, 0.0], egui::DragValue::new(value).speed(speed))
            .changed();
    });
    changed
}

fn drag_f32_clamp(
    ui: &mut egui::Ui,
    value: &mut f32,
    speed: f32,
    range: RangeInclusive<usize>,
) -> bool {
    let mut changed = false;
    ui.columns(1, |ui| {
        changed = ui[0]
            .add_sized(
                [0.0, 0.0],
                egui::DragValue::new(value).speed(speed).clamp_range(range),
            )
            .changed();
    });
    changed
}

fn drag_u8(ui: &mut egui::Ui, value: &mut u8, speed: f32) -> bool {
    let mut changed = false;
    ui.columns(1, |ui| {
        changed |= ui[0]
            .add_sized([0.0, 0.0], egui::DragValue::new(value).speed(speed))
            .changed();
    });
    changed
}

fn drag_usize(ui: &mut egui::Ui, value: &mut usize, speed: f32, max: usize) -> bool {
    let mut changed = false;
    ui.columns(1, |ui| {
        changed |= ui[0]
            .add_sized(
                [0.0, 0.0],
                egui::DragValue::new(value)
                    .speed(speed)
                    .clamp_range(0..=max),
            )
            .changed();
    });
    changed
}

fn drag_vec3_color(ui: &mut egui::Ui, value: &mut Vec3) -> bool {
    let mut changed = false;
    let speed = 0.0025;
    let size = [0.0, 0.0];
    ui.columns(4, |ui| {
        changed |= ui[0]
            .add_sized(
                size,
                egui::DragValue::new(&mut value.x)
                    .speed(speed)
                    .prefix("R: ")
                    .clamp_range(0.0..=1.0)
                    .fixed_decimals(1),
            )
            .changed();
        changed |= ui[1]
            .add_sized(
                size,
                egui::DragValue::new(&mut value.y)
                    .speed(speed)
                    .prefix("G: ")
                    .clamp_range(0.0..=1.0)
                    .fixed_decimals(1),
            )
            .changed();
        changed |= ui[2]
            .add_sized(
                size,
                egui::DragValue::new(&mut value.z)
                    .speed(speed)
                    .prefix("B: ")
                    .clamp_range(0.0..=1.0)
                    .fixed_decimals(1),
            )
            .changed();

        let mut color = value.to_array();
        changed |= ui[3].color_edit_button_rgb(&mut color).changed();
        *value = Vec3::from_array(color);
    });
    changed
}

fn fmt_usize_separator(value: usize) -> String {
    let mut s = String::new();
    let str = value.to_string();
    for (i, val) in str.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            s.insert(0, '_');
        }
        s.insert(0, val);
    }
    s
}
