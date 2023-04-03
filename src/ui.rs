use crate::{
    camera::CustomCamera,
    egui_utils::{
        drag_f32, drag_f32_clamp, drag_u8, drag_usize, drag_vec3, drag_vec3_color,
        fmt_usize_separator,
    },
    renderer::Renderer,
    scene::Scene,
    Frametimes, ViewportEguiTexture, ViewportScale, ViewportSize,
};

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextureId},
    EguiContexts,
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
    mut egui_context: EguiContexts,
    mut tree: ResMut<DockTree>,
    time: Res<Time>,
    mut scene: ResMut<Scene>,
    viewport_egui_texture: Res<ViewportEguiTexture>,
    mut viewport_size: ResMut<ViewportSize>,
    render_dt: Res<Frametimes>,
    mut camera: ResMut<CustomCamera>,
    mut renderer: ResMut<Renderer>,
    mut viewport_scale: ResMut<ViewportScale>,
) {
    puffin::profile_function!();
    let mut tab_viewer = TabViewer {
        viewport_texture: viewport_egui_texture.0,
        viewport_size: &mut viewport_size.0,
        // TODO have diagnostic struct
        dt: time.delta_seconds(),
        frametimes: &render_dt,
        scene: &mut scene,
        camera: &mut camera,
        renderer: &mut renderer,
        viewport_scale: &mut viewport_scale.0,
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
    pub camera: &'a mut CustomCamera,
    pub renderer: &'a mut Renderer,
    pub viewport_scale: &'a mut f32,
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
                    ui.label("Ground Color");
                    reset |= drag_vec3_color(ui, &mut self.scene.sky.ground_color);
                    ui.end_row();
                    ui.label("Horizon Color");
                    reset |= drag_vec3_color(ui, &mut self.scene.sky.horizon_color);
                    ui.end_row();
                    ui.label("Zenith Color");
                    reset |= drag_vec3_color(ui, &mut self.scene.sky.zenith_color);
                    ui.end_row();

                    // ui.label("Direction");
                    // reset |= drag_vec3(ui, &mut self.scene.sky.sun_direction, 0.025);
                    // ui.end_row();
                    // ui.label("Focus");
                    // reset |= drag_f32(ui, &mut self.scene.sky.sun_focus, 0.025);
                    // ui.end_row();
                    // ui.label("Intensity");
                    // reset |= drag_f32_clamp(ui, &mut self.scene.sky.sun_intensity, 0.005, 0..=1);
                    // ui.end_row();
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
                            reset |= drag_f32_clamp(ui, &mut light.intensity, 0.025, 0.0..=1.0);
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
                            reset |= drag_f32_clamp(ui, &mut material.roughness, 0.025, 0.0..=1.0);
                            ui.end_row();

                            ui.label("Metallic");
                            reset |= drag_f32_clamp(ui, &mut material.metallic, 0.025, 0.0..=1.0);
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

                ui.heading("Meshes");
                for (i, mesh) in self.scene.meshes.iter_mut().enumerate() {
                    egui::Grid::new(format!("mesh_grid_{i}"))
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Position");
                            reset |= drag_vec3(ui, &mut mesh.transform.translation, 0.1);
                            ui.end_row();

                            ui.label("Material id");
                            reset |= drag_usize(
                                ui,
                                &mut mesh.material_id,
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

                ui.horizontal(|ui| {
                    ui.label("Viewport Scale");
                    reset |= drag_f32_clamp(ui, self.viewport_scale, 0.05, 0.1..=1.0);
                });
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
