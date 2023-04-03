#![allow(unused)]

use std::ops::RangeInclusive;

use bevy::{math::Vec3A, prelude::*};
use bevy_egui::egui::{self};

pub fn drag_vec3(ui: &mut egui::Ui, value: &mut Vec3, speed: f32) -> bool {
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

pub fn drag_vec3a(ui: &mut egui::Ui, value: &mut Vec3A, speed: f32) -> bool {
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

pub fn drag_vec3_clamp(
    ui: &mut egui::Ui,
    value: &mut Vec3,
    speed: f32,
    range: RangeInclusive<f32>,
) -> bool {
    let mut changed = false;
    ui.columns(3, |ui| {
        changed |= ui[0]
            .add_sized(
                [0.0, 0.0],
                egui::DragValue::new(&mut value.x)
                    .speed(speed)
                    .clamp_range(range.clone()),
            )
            .changed();
        changed |= ui[1]
            .add_sized(
                [0.0, 0.0],
                egui::DragValue::new(&mut value.y)
                    .speed(speed)
                    .clamp_range(range.clone()),
            )
            .changed();
        changed |= ui[2]
            .add_sized(
                [0.0, 0.0],
                egui::DragValue::new(&mut value.z)
                    .speed(speed)
                    .clamp_range(range.clone()),
            )
            .changed();
    });
    changed
}

pub fn drag_f32(ui: &mut egui::Ui, value: &mut f32, speed: f32) -> bool {
    let mut changed = false;
    ui.columns(1, |ui| {
        changed = ui[0]
            .add_sized([0.0, 0.0], egui::DragValue::new(value).speed(speed))
            .changed();
    });
    changed
}

pub fn drag_f32_clamp(
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

pub fn drag_u8(ui: &mut egui::Ui, value: &mut u8, speed: f32) -> bool {
    let mut changed = false;
    ui.columns(1, |ui| {
        changed |= ui[0]
            .add_sized([0.0, 0.0], egui::DragValue::new(value).speed(speed))
            .changed();
    });
    changed
}

pub fn drag_usize(ui: &mut egui::Ui, value: &mut usize, speed: f32, max: usize) -> bool {
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

pub fn drag_vec3_color(ui: &mut egui::Ui, value: &mut Vec3) -> bool {
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

pub fn fmt_usize_separator(value: usize) -> String {
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
