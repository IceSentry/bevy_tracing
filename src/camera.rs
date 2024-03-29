use bevy::{
    input::mouse::MouseMotion,
    math::{Vec3A, Vec4Swizzles},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::renderer::Renderer;

#[derive(Debug, Default, Clone, Resource)]
pub struct CustomCamera {
    pub projection: Mat4,
    pub view: Mat4,
    pub inverse_projection: Mat4,
    pub inverse_view: Mat4,

    pub position: Vec3,
    pub forward_direction: Vec3,

    pub ray_directions: Vec<Vec3A>,

    vertical_fov: f32,
    near_clip: f32,
    far_clip: f32,

    viewport_width: u32,
    viewport_height: u32,
}

impl CustomCamera {
    pub fn new(vertical_fov: f32, near_clip: f32, far_clip: f32) -> Self {
        Self {
            vertical_fov,
            near_clip,
            far_clip,
            forward_direction: Vec3::NEG_Z,
            position: Vec3::new(0.0, 0.0, 6.0),
            ..default()
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.viewport_width == width && self.viewport_height == height {
            return;
        }

        self.viewport_width = width;
        self.viewport_height = height;

        self.recalculate_projection();
        self.recalculate_ray_directions();
    }

    fn recalculate_projection(&mut self) {
        self.projection = Mat4::perspective_rh(
            self.vertical_fov.to_radians(),
            self.viewport_width as f32 / self.viewport_height as f32,
            self.near_clip,
            self.far_clip,
        );
        self.inverse_projection = self.projection.inverse();
    }

    fn recalculate_view(&mut self) {
        self.view = Mat4::look_at_rh(
            self.position,
            self.position + self.forward_direction,
            Vec3::Y,
        );
        self.inverse_view = self.view.inverse();
    }

    fn recalculate_ray_directions(&mut self) {
        let _span = info_span!("recalculate ray directions").entered();
        self.ray_directions.resize(
            (self.viewport_width * self.viewport_height) as usize,
            Vec3A::ZERO,
        );

        // This is called every time the camera moves so it's important to make it fast
        self.ray_directions
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, ray_dir)| {
                let x = i % self.viewport_width as usize + 1;
                let y = i / self.viewport_width as usize + 1;
                let coord = Vec2::new(
                    x as f32 / self.viewport_width as f32,
                    y as f32 / self.viewport_height as f32,
                );
                let mut coord = coord * 2.0 - 1.0; // -1 .. 1
                coord.y = -coord.y;

                let target = self.inverse_projection * coord.extend(1.0).extend(1.0);
                // world space
                *ray_dir = (self.inverse_view * (target.xyz() / target.w).normalize().extend(0.0))
                    .xyz()
                    .into();
            });
    }
}

pub fn update_camera(
    mut camera: ResMut<CustomCamera>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut renderer: ResMut<Renderer>,
) {
    let mouse_motion_delta = mouse_motion_events
        .iter()
        .map(|mouse_motion| mouse_motion.delta)
        .last();

    let mut window = primary_window.single_mut();
    if !mouse_button_input.pressed(MouseButton::Right) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
        return;
    }
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Confined;

    let mut moved = false;

    let up_direction = Vec3::Y;
    let forward_direction = camera.forward_direction;
    let right_direction = camera.forward_direction.cross(up_direction);

    let speed = 5.0;
    let rotation_speed = 1.0;

    if keyboard_input.pressed(KeyCode::W) {
        camera.position += forward_direction * speed * time.delta_seconds();
        moved = true;
    } else if keyboard_input.pressed(KeyCode::S) {
        camera.position -= forward_direction * speed * time.delta_seconds();
        moved = true;
    }

    if keyboard_input.pressed(KeyCode::A) {
        camera.position -= right_direction * speed * time.delta_seconds();
        moved = true;
    } else if keyboard_input.pressed(KeyCode::D) {
        camera.position += right_direction * speed * time.delta_seconds();
        moved = true;
    }

    if keyboard_input.pressed(KeyCode::Q) {
        camera.position -= up_direction * speed * time.delta_seconds();
        moved = true;
    } else if keyboard_input.pressed(KeyCode::E) {
        camera.position += up_direction * speed * time.delta_seconds();
        moved = true;
    }

    // rotation
    if let Some(delta) = mouse_motion_delta {
        if delta.x != 0.0 || delta.y != 0.0 {
            let pitch_delta = delta.y * rotation_speed * time.delta_seconds();
            let yaw_delta = delta.x * rotation_speed * time.delta_seconds();
            let q = Quat::from_axis_angle(right_direction, -pitch_delta)
                * Quat::from_axis_angle(up_direction, -yaw_delta);
            camera.forward_direction = q.normalize() * forward_direction;

            moved = true;
        }
    }

    if moved {
        camera.recalculate_view();
        camera.recalculate_ray_directions();
        renderer.reset_frame_index();
    }
}
