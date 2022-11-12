use crate::{scene::Scene, RenderDt, ViewportEguiTexture, ViewportSize};

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

#[derive(Deref, DerefMut)]
pub struct DockTree(pub Tree<Tabs>);

pub fn setup_ui(mut commands: Commands) {
    // Setup dock tree
    let mut tree = Tree::new(vec![Tabs::Viewport]);
    let [_viewport, scene] = tree.split_right(NodeIndex::root(), 0.8, vec![Tabs::Scene]);
    tree.split_below(scene, 0.5, vec![Tabs::Settings]);
    commands.insert_resource(DockTree(tree));
}

pub fn draw_dock_area(
    mut egui_context: ResMut<EguiContext>,
    mut tree: ResMut<DockTree>,
    time: Res<Time>,
    mut scene: ResMut<Scene>,
    viewport_egui_texture: Res<ViewportEguiTexture>,
    mut viewport_size: ResMut<ViewportSize>,
    render_dt: Res<RenderDt>,
) {
    let mut tab_viewer = TabViewer {
        viewport_texture: viewport_egui_texture.0,
        viewport_size: viewport_size.0,
        dt: time.delta_seconds(),
        render_dt: render_dt.0,
        scene: scene.clone(),
    };

    DockArea::new(&mut tree)
        .style(Style::from_egui(egui_context.ctx_mut().style().as_ref()))
        .show(egui_context.ctx_mut(), &mut tab_viewer);

    *scene = tab_viewer.scene.clone();
    viewport_size.0 = tab_viewer.viewport_size;
}

#[derive(Default)]
pub struct TabViewer {
    pub viewport_texture: TextureId,
    pub viewport_size: Vec2,
    pub dt: f32,
    pub render_dt: f32,
    pub scene: Scene,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Tabs;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tabs::Viewport => {
                self.viewport_size = Vec2::from_array(ui.available_size().into());
                ui.image(self.viewport_texture, ui.available_size());
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
