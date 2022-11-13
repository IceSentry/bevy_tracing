use std::ops::RangeInclusive;

use crate::{
    camera::ChernoCamera, renderer::Renderer, scene::Scene, Bounces, RenderDt, ViewportEguiTexture,
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
    // Setup dock tree
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
    render_dt: Res<RenderDt>,
    mut bounces: ResMut<Bounces>,
    mut camera: ResMut<ChernoCamera>,
    mut renderer: ResMut<Renderer>,
) {
    let mut tab_viewer = TabViewer {
        viewport_texture: viewport_egui_texture.0,
        viewport_size: viewport_size.0,
        dt: time.delta_seconds(),
        render_dt: render_dt.0,
        scene: scene.clone(),
        camera: camera.clone(),
        bounces: bounces.0,
        reset: false,
        frame_index: renderer.frame_index,
        accumulate: renderer.accumulate,
    };

    DockArea::new(&mut tree)
        .style(Style::from_egui(egui_context.ctx_mut().style().as_ref()))
        .show(egui_context.ctx_mut(), &mut tab_viewer);

    *scene = tab_viewer.scene.clone();
    *camera = tab_viewer.camera.clone();
    viewport_size.0 = tab_viewer.viewport_size;
    bounces.0 = tab_viewer.bounces;
    if tab_viewer.reset {
        renderer.reset_frame_index();
    }
    renderer.accumulate = tab_viewer.accumulate;
}

#[derive(Default)]
pub struct TabViewer {
    pub viewport_texture: TextureId,
    pub viewport_size: Vec2,
    pub dt: f32,
    pub render_dt: f32,
    pub scene: Scene,
    pub bounces: u8,
    pub camera: ChernoCamera,
    pub reset: bool,
    pub frame_index: usize,
    pub accumulate: bool,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Tabs;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tabs::Viewport => {
                self.viewport_size = Vec2::from_array(ui.available_size().into());
                ui.image(self.viewport_texture, ui.available_size());
            }
            Tabs::Scene => {
                ui.heading("Camera");
                egui::Grid::new("camera_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Position");
                        drag_vec3(ui, &mut self.camera.position, 0.1);
                        ui.end_row();
                    });
                ui.separator();

                ui.heading("Materials");
                for (i, material) in self.scene.materials.iter_mut().enumerate() {
                    egui::Grid::new(format!("material_grid_{i}"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Albedo");
                            drag_vec3_color(ui, &mut material.albedo);
                            ui.end_row();

                            ui.label("Roughness");
                            self.reset |= drag_f32_clamp(ui, &mut material.roughness, 0.025, 0..=1);
                            ui.end_row();

                            ui.label("Metallic");
                            self.reset |= drag_f32_clamp(ui, &mut material.metallic, 0.025, 0..=1);
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
                            drag_vec3(ui, &mut sphere.position, 0.1);
                            ui.end_row();

                            ui.label("Radius");
                            self.reset |= drag_f32(ui, &mut sphere.radius, 0.025);
                            ui.end_row();

                            ui.label("Material id");
                            drag_usize(
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
                ui.label(format!("Viewport size: {:?}", self.viewport_size));
                ui.label(format!("dt: {}ms", self.dt * 1000.0));
                ui.label(format!("render dt: {}ms", self.render_dt * 1000.0));
                ui.label(format!("Frame index: {}", self.frame_index));

                ui.horizontal(|ui| {
                    ui.label("Bounces");
                    drag_u8(ui, &mut self.bounces, 0.25);
                });

                ui.checkbox(&mut self.accumulate, "Accumulate");
                self.reset |= ui.button("Reset").clicked();
            }
        };
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }
}

fn drag_vec3(ui: &mut egui::Ui, value: &mut Vec3, speed: f32) {
    ui.columns(3, |ui| {
        ui[0].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.x).speed(speed));
        ui[1].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.y).speed(speed));
        ui[2].add_sized([0.0, 0.0], egui::DragValue::new(&mut value.z).speed(speed));
    });
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

fn drag_u8(ui: &mut egui::Ui, value: &mut u8, speed: f32) {
    ui.columns(1, |ui| {
        ui[0].add_sized([0.0, 0.0], egui::DragValue::new(value).speed(speed));
    });
}

fn drag_usize(ui: &mut egui::Ui, value: &mut usize, speed: f32, max: usize) {
    ui.columns(1, |ui| {
        ui[0].add_sized(
            [0.0, 0.0],
            egui::DragValue::new(value)
                .speed(speed)
                .clamp_range(0..=max),
        );
    });
}

fn drag_vec3_color(ui: &mut egui::Ui, value: &mut Vec3) {
    let speed = 0.0025;
    let size = [40.0, 20.0];
    ui.columns(4, |ui| {
        ui[0].add_sized(
            size,
            egui::DragValue::new(&mut value.x)
                .speed(speed)
                .prefix("R: ")
                .clamp_range(0.0..=1.0)
                .fixed_decimals(1),
        );
        ui[1].add_sized(
            size,
            egui::DragValue::new(&mut value.y)
                .speed(speed)
                .prefix("G: ")
                .clamp_range(0.0..=1.0)
                .fixed_decimals(1),
        );
        ui[2].add_sized(
            size,
            egui::DragValue::new(&mut value.z)
                .speed(speed)
                .prefix("B: ")
                .clamp_range(0.0..=1.0)
                .fixed_decimals(1),
        );

        let mut color = value.to_array();
        ui[3].color_edit_button_rgb(&mut color);
        *value = Vec3::from_array(color);
    });
}
