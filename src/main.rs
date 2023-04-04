mod camera;
mod egui_utils;
mod math_utils;
mod renderer;
mod scene;
mod ui;

use std::time::Instant;

use bevy::{
    math::vec3,
    prelude::{shape::Cube, *},
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    window::PresentMode,
};
use bevy_egui::{egui::TextureId, EguiContexts, EguiPlugin};
use camera::{update_camera, CustomCamera};

use renderer::Renderer;
use scene::{Light, Material, Scene, Sky, Sphere, TriangleMesh};
use ui::{draw_dock_area, setup_ui};

#[derive(Resource)]
struct ViewportImage(Handle<Image>);
#[derive(Resource)]
pub struct ViewportEguiTexture(pub TextureId);
#[derive(Resource)]
pub struct ViewportSize(pub Vec2);
#[derive(Debug, Default, Resource)]
pub struct Frametimes {
    render: f32,
    image_copy: f32,
}

#[derive(Resource)]
pub struct RenderScale(pub f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Tracing".to_string(),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        // .add_plugin(PuffinTracePlugin::new())
        // .insert_resource(ShowProfiler(true))
        .add_plugin(EguiPlugin)
        .init_resource::<Frametimes>()
        .insert_resource(RenderScale(0.75))
        .insert_resource(CustomCamera::new(45.0, 0.1, 100.0))
        // TODO use bevy scene feature
        .insert_resource(Scene {
            sky: Sky {
                zenith_color: vec3(0.6, 0.7, 0.9),
                horizon_color: Vec3::ONE,
                ground_color: vec3(0.7, 0.7, 0.7),
            },
            // sky: Sky::BLACK,
            lights: vec![
            //     Light {
            //     direction: vec3(1.0, 1.0, 1.0),
            //     intensity: 1.0,
            // }
            ],
            materials: vec![
                Material {
                    albedo: vec3(1.0, 0.0, 1.0),
                    roughness: 0.0,
                    ..default()
                },
                Material {
                    albedo: vec3(0.0, 0.0, 0.0),
                    roughness: 1.0,
                    ..default()
                },
                Material {
                    albedo: vec3(1.0, 0.0, 0.0),
                    roughness: 1.0,
                    ..default()
                },
                Material {
                    albedo: vec3(0.0, 1.0, 0.0),
                    roughness: 1.0,
                    ..default()
                },
                Material {
                    albedo: vec3(0.0, 0.0, 1.0),
                    roughness: 1.0,
                    ..default()
                },
                Material {
                    emissive: vec3(1.0, 1.0, 1.0),
                    emissive_intensity: 2.0,
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
                    position: vec3(0.0, -201.0, 0.0),
                    radius: 200.0,
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
                Sphere {
                    position: vec3(20.0, 20.0, 20.0),
                    radius: 10.0,
                    material_id: 5,
                },
            ],
            meshes: vec![
            //     {
            //     let mesh: Mesh = Cube { size: 1.0 }.into();
            //     let aabb = mesh.compute_aabb().unwrap();
            //     TriangleMesh {
            //         transform: Transform::from_xyz(0.0, 0.0, 0.0),
            //         mesh,
            //         material_id: 0,
            //         aabb,
            //     }
            // }
            ],
        })
        .add_startup_system(setup_renderer)
        .add_startup_system(setup_ui)
        .add_system(draw_dock_area)
        .add_system(resize_image.after(draw_dock_area))
        .add_system(render.after(resize_image))
        .add_system(update_camera)
        // .add_system(show_profiler)
        .run();
}

// #[derive(Resource)]
// struct ShowProfiler(bool);

// fn show_profiler(mut ctx: ResMut<EguiContext>, mut show: ResMut<ShowProfiler>) {
//     // TODO toggle
//     let ctx = ctx.ctx_mut();
//     if show.0 {
//         show.0 = puffin_egui::profiler_window(ctx);
//     }
// }

fn setup_renderer(
    mut commands: Commands,
    mut egui_ctx: EguiContexts,
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
            view_formats: &[],
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

#[allow(clippy::too_many_arguments)]
fn resize_image(
    viewport_image: Res<ViewportImage>,
    viewport_size: Res<ViewportSize>,
    mut images: ResMut<Assets<Image>>,
    mut renderer: ResMut<Renderer>,
    mut camera: ResMut<CustomCamera>,
    render_scale: Res<RenderScale>,
    mut prev_scale: Local<f32>,
    mut prev_viewport_size: Local<Vec2>,
) {
    let image = images.get_mut(&viewport_image.0).unwrap();
    if viewport_size.0 != *prev_viewport_size || render_scale.0 != *prev_scale {
        println!("reset");
        let size = Extent3d {
            width: (viewport_size.0.x * render_scale.0) as u32,
            height: (viewport_size.0.y * render_scale.0) as u32,
            ..default()
        };

        // This also clears the image with 0
        image.resize(size);

        camera.resize(size.width, size.height);
        renderer.resize(size.width as usize, size.height as usize);
    }

    *prev_scale = render_scale.0;
    *prev_viewport_size = viewport_size.0;
}

fn render(
    viewport_image: Res<ViewportImage>,
    mut images: ResMut<Assets<Image>>,
    mut renderer: ResMut<Renderer>,
    mut frametimes: ResMut<Frametimes>,
    camera: Res<CustomCamera>,
    scene: Res<Scene>,
) {
    // TODO use diagnostic system
    let start = Instant::now();
    {
        let _span = info_span!("render").entered();
        renderer.render(&camera, &scene);
    }
    frametimes.render = start.elapsed().as_secs_f32();

    let start = Instant::now();
    let image = images.get_mut(&viewport_image.0).unwrap();
    {
        let _span = info_span!("image copy").entered();
        image.data = renderer.image_data.iter().flat_map(|p| *p).collect();
    }
    frametimes.image_copy = start.elapsed().as_secs_f32();
}
