use bevy::{input::mouse::MouseMotion, math::Vec4Swizzles, prelude::*};

#[derive(Debug, Default)]
pub struct ChernoCamera {
    pub projection: Mat4,
    pub view: Mat4,
    pub inverse_projection: Mat4,
    pub inverse_view: Mat4,

    pub position: Vec3,
    pub forward_direction: Vec3,

    pub ray_directions: Vec<Vec3>,

    vertical_fov: f32,
    near_clip: f32,
    far_clip: f32,

    viewport_width: u32,
    viewport_height: u32,
}

impl ChernoCamera {
    pub fn new(vertical_fov: f32, near_clip: f32, far_clip: f32) -> Self {
        Self {
            vertical_fov,
            near_clip,
            far_clip,
            forward_direction: Vec3::new(0.0, 0.0, -1.0),
            position: Vec3::new(0.0, 0.0, 3.0),
            ..Default::default()
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
        self.ray_directions.resize(
            (self.viewport_width * self.viewport_height) as usize,
            Vec3::ZERO,
        );

        for y in 0..self.viewport_height {
            for x in 0..self.viewport_width {
                let coord = Vec2::new(
                    x as f32 / self.viewport_width as f32,
                    y as f32 / self.viewport_height as f32,
                );
                let mut coord = coord * 2.0 - 1.0; // -1 .. 1
                coord.y = -coord.y;

                let target = self.inverse_projection * coord.extend(1.0).extend(1.0);
                let ray_direction =
                    (self.inverse_view * (target.xyz() / target.w).normalize().extend(0.0)).xyz(); // world space
                self.ray_directions[(x + y * self.viewport_width) as usize] = ray_direction;
            }
        }
    }
}

pub fn update_camera(
    mut camera: ResMut<ChernoCamera>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.primary_mut();
    if !mouse_button_input.pressed(MouseButton::Right) {
        window.set_cursor_visibility(true);
        window.set_cursor_lock_mode(false);
        return;
    }
    window.set_cursor_visibility(false);
    window.set_cursor_lock_mode(true);

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

    if let Some(mouse_motion) = mouse_motion_events.iter().next() {
        // rotation
        if mouse_motion.delta.x != 0.0 || mouse_motion.delta.y != 0.0 {
            let pitch_delta = mouse_motion.delta.y * rotation_speed * time.delta_seconds();
            let yaw_delta = mouse_motion.delta.x * rotation_speed * time.delta_seconds();
            let q = Quat::from_axis_angle(right_direction, -pitch_delta)
                * Quat::from_axis_angle(up_direction, -yaw_delta);
            camera.forward_direction = q.normalize() * forward_direction;

            moved = true;
        }
    }

    if moved {
        camera.recalculate_view();
        camera.recalculate_ray_directions();
    }
}
