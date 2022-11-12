mod camera;
mod renderer;
mod scene;
mod ui;

use std::time::Instant;

use bevy::{
    math::vec3,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::PresentMode,
};
use bevy_egui::{EguiContext, EguiPlugin};
use camera::{update_camera, ChernoCamera};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
use renderer::Renderer;
use scene::{Scene, Sphere};
use ui::{DockTree, TabViewer, Tabs};

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
        .init_resource::<TabViewer>()
        .insert_resource(ChernoCamera::new(45.0, 0.1, 100.0))
        .insert_resource(Scene {
            spheres: vec![
                Sphere {
                    position: Vec3::ZERO,
                    radius: 0.5,
                    albedo: vec3(1.0, 0.0, 1.0),
                },
                Sphere {
                    position: vec3(1.0, 0.0, -5.0),
                    radius: 1.5,
                    albedo: vec3(0.2, 0.3, 1.0),
                },
            ],
        })
        .add_startup_system(setup_viewport)
        .add_system(draw_dock_area)
        .add_system(resize_image.after(draw_dock_area))
        .add_system(render.after(resize_image))
        .add_system(update_camera)
        .run();
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
    let [_viewport, scene] = tree.split_right(NodeIndex::root(), 0.8, vec![Tabs::Scene]);
    tree.split_below(scene, 0.5, vec![Tabs::Settings]);
    commands.insert_resource(DockTree(tree));
}

fn draw_dock_area(
    mut egui_context: ResMut<EguiContext>,
    mut tree: ResMut<DockTree>,
    mut tab_viewer: ResMut<TabViewer>,
    time: Res<Time>,
    mut scene: ResMut<Scene>,
) {
    tab_viewer.dt = time.delta_seconds();
    tab_viewer.scene = scene.clone();
    DockArea::new(&mut tree)
        .style(Style::from_egui(egui_context.ctx_mut().style().as_ref()))
        .show(egui_context.ctx_mut(), &mut *tab_viewer);
    *scene = tab_viewer.scene.clone();
}

fn resize_image(
    viewport_image: Res<ViewportImage>,
    mut images: ResMut<Assets<Image>>,
    tab_viewer: Res<TabViewer>,
    mut renderer: ResMut<Renderer>,
    mut camera: ResMut<ChernoCamera>,
) {
    let image = images.get_mut(&viewport_image.0).unwrap();
    let viewport_size = tab_viewer.viewport_size;
    if image.size().x != viewport_size.x || image.size().y != viewport_size.y {
        let size = Extent3d {
            width: viewport_size.x as u32,
            height: viewport_size.y as u32,
            ..default()
        };
        // This also clears the image with 0
        image.resize(size);

        camera.resize(size.width, size.height);
        renderer.resize(size.width as usize, size.height as usize);
    }
}

fn render(
    viewport_image: Res<ViewportImage>,
    mut images: ResMut<Assets<Image>>,
    mut renderer: ResMut<Renderer>,
    mut tab_viewer: ResMut<TabViewer>,
    camera: Res<ChernoCamera>,
    scene: Res<Scene>,
) {
    let start = Instant::now();

    {
        let _render_span = info_span!("render").entered();
        renderer.render(&camera, &scene);
    }

    let elapsed = (Instant::now() - start).as_secs_f32();
    tab_viewer.render_dt = elapsed as f32;

    let image = images.get_mut(&viewport_image.0).unwrap();
    {
        let _image_span = info_span!("update image").entered();
        // There's probably a more efficient way to do this using unsafe code, but it's good enough
        image.data = renderer
            .image_data
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<u8>>();
    }
}
