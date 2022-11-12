use std::time::Instant;

use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::PresentMode,
};
use bevy_egui::{
    egui::{self, TextureId},
    EguiContext, EguiPlugin,
};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
use renderer::Renderer;

mod renderer;

#[derive(Debug, Clone)]
enum Tabs {
    Viewport(TextureId),
    Settings,
    Scene,
}

#[derive(Deref, DerefMut)]
struct DockTree(Tree<Tabs>);

struct ViewportImage(Handle<Image>);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Cherno Tracing".to_string(),
            present_mode: PresentMode::AutoNoVsync,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .init_resource::<TabViewerRes>()
        .add_startup_system(setup_viewport)
        .add_system(draw_dock_area)
        .add_system(resize_image.after(draw_dock_area))
        .add_system(render.after(resize_image))
        .run();
}

fn draw_dock_area(
    mut egui_context: ResMut<EguiContext>,
    mut tree: ResMut<DockTree>,
    mut tab_viewer: ResMut<TabViewerRes>,
    time: Res<Time>,
) {
    tab_viewer.dt = time.delta_seconds();
    DockArea::new(&mut tree)
        .style(Style::from_egui(egui_context.ctx_mut().style().as_ref()))
        .show(egui_context.ctx_mut(), &mut *tab_viewer);
}

fn setup_viewport(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);
    let image_id = egui_ctx.add_image(image_handle.clone());
    commands.insert_resource(ViewportImage(image_handle));

    commands.insert_resource(Renderer::new(size.width as usize, size.height as usize));

    // Setup dock tree
    let mut tree = Tree::new(vec![Tabs::Viewport(image_id)]);
    let [_viewport, scene] = tree.split_right(NodeIndex::root(), 0.75, vec![Tabs::Scene]);
    tree.split_below(scene, 0.5, vec![Tabs::Settings]);
    commands.insert_resource(DockTree(tree));
}

fn resize_image(
    viewport_image: Res<ViewportImage>,
    mut images: ResMut<Assets<Image>>,
    tab_viewer: Res<TabViewerRes>,
    mut renderer: ResMut<Renderer>,
) {
    let image = images.get_mut(&viewport_image.0).unwrap();
    if image.size().x != tab_viewer.viewport_size.x || image.size().y != tab_viewer.viewport_size.y
    {
        let size = Extent3d {
            width: tab_viewer.viewport_size.x as u32,
            height: tab_viewer.viewport_size.y as u32,
            ..default()
        };
        // This also clears the image with 0
        image.resize(size);
        renderer.resize(size.width as usize, size.height as usize);
    }
}

fn render(
    viewport_image: Res<ViewportImage>,
    mut images: ResMut<Assets<Image>>,
    mut renderer: ResMut<Renderer>,
    mut tab_viewer: ResMut<TabViewerRes>,
) {
    let start = Instant::now();

    renderer.render();

    let elapsed = (Instant::now() - start).as_secs_f32();
    tab_viewer.render_dt = elapsed as f32;

    let image = images.get_mut(&viewport_image.0).unwrap();
    // There's probably a more efficient way to do this using unsafe code, but it's good enough
    image.data = renderer
        .image_data
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<u8>>();
}

#[derive(Default)]
struct TabViewerRes {
    viewport_size: Vec2,
    dt: f32,
    render_dt: f32,
}

impl egui_dock::TabViewer for TabViewerRes {
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
            Tabs::Scene => {}
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
