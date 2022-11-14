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
use bevy_egui::{egui::TextureId, EguiContext, EguiPlugin};
use camera::{update_camera, ChernoCamera};

use renderer::Renderer;
use scene::{Light, Material, Scene, Sphere};
use ui::{draw_dock_area, setup_ui};

#[derive(Resource)]
struct ViewportImage(Handle<Image>);
#[derive(Resource)]
pub struct ViewportEguiTexture(pub TextureId);
#[derive(Resource)]
pub struct ViewportSize(pub Vec2);
#[derive(Debug, Default, Resource)]
pub struct RenderDt(pub f32);
#[derive(Resource)]
pub struct Bounces(pub u8);
#[derive(Resource)]
pub struct SkyColor(pub Vec4);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Cherno Tracing".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .init_resource::<RenderDt>()
        .insert_resource(Bounces(5))
        .insert_resource(ChernoCamera::new(45.0, 0.1, 100.0))
        .insert_resource(Scene {
            sky_color: vec3(0.6, 0.7, 0.9),
            lights: vec![Light {
                direction: vec3(1.0, 1.0, 1.0),
                intensity: 1.0,
            }],
            materials: vec![
                Material {
                    albedo: vec3(1.0, 0.0, 1.0),
                    roughness: 0.0,
                    ..default()
                },
                Material {
                    albedo: vec3(0.0, 0.0, 0.0),
                    roughness: 0.1,
                    ..default()
                },
                Material {
                    albedo: vec3(1.0, 0.0, 0.0),
                    roughness: 0.0,
                    ..default()
                },
                Material {
                    albedo: vec3(0.0, 1.0, 0.0),
                    roughness: 0.0,
                    ..default()
                },
                Material {
                    albedo: vec3(0.0, 0.0, 1.0),
                    roughness: 0.0,
                    ..default()
                },
            ],
            spheres: vec![
                // Sphere {
                //     position: Vec3::ZERO,
                //     radius: 1.0,
                //     material_id: 0,
                // },
                Sphere {
                    position: vec3(0.0, -101.0, 0.0),
                    radius: 100.0,
                    material_id: 1,
                },
                Sphere {
                    position: vec3(-1.25, -0.5, 0.0),
                    radius: 0.5,
                    material_id: 2,
                },
                Sphere {
                    position: vec3(0.0, -0.5, 0.0),
                    radius: 0.5,
                    material_id: 3,
                },
                Sphere {
                    position: vec3(1.25, -0.5, 0.0),
                    radius: 0.5,
                    material_id: 4,
                },
            ],
        })
        .add_startup_system(setup_renderer)
        .add_startup_system(setup_ui)
        .add_system(draw_dock_area)
        .add_system(resize_image.after(draw_dock_area))
        .add_system(render.after(resize_image))
        .add_system(update_camera)
        .run();
}

fn setup_renderer(
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
    commands.insert_resource(ViewportImage(image_handle.clone()));
    commands.insert_resource(ViewportEguiTexture(egui_ctx.add_image(image_handle)));
    commands.insert_resource(ViewportSize(Vec2::new(
        size.width as f32,
        size.height as f32,
    )));

    commands.insert_resource(Renderer::new(size.width as usize, size.height as usize));
}

fn resize_image(
    viewport_image: Res<ViewportImage>,
    viewport_size: Res<ViewportSize>,
    mut images: ResMut<Assets<Image>>,
    mut renderer: ResMut<Renderer>,
    mut camera: ResMut<ChernoCamera>,
) {
    let image = images.get_mut(&viewport_image.0).unwrap();
    if image.size().x != viewport_size.0.x || image.size().y != viewport_size.0.y {
        let size = Extent3d {
            width: viewport_size.0.x as u32,
            height: viewport_size.0.y as u32,
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
    mut render_dt: ResMut<RenderDt>,
    camera: Res<ChernoCamera>,
    scene: Res<Scene>,
    bounces: Res<Bounces>,
) {
    let start = Instant::now();

    {
        let _render_span = info_span!("render").entered();
        renderer.render(&camera, &scene, bounces.0);
    }

    let elapsed = (Instant::now() - start).as_secs_f32();
    render_dt.0 = elapsed as f32;

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
