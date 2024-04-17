use eframe::egui::Color32;

use crate::messages::path::PointType;

pub const DRIVE_COMMAND_BALL: Color32 = Color32::from_gray(200);
pub const DRIVE_ACTUAL_BALL: Color32 = Color32::from_gray(100);
pub const CONNECTED_TEXT: Color32 = Color32::from_rgb(50, 255, 75);
pub const NOT_CONNECTED_TEXT: Color32 = Color32::from_rgb(255, 100, 75);

pub const SHADE: Color32 = Color32::from_gray(50);

pub fn point_colour(point_type: &PointType) -> Color32 {
    match point_type {
        PointType::LineLeft => Color32::from_rgb(0, 0, 255),
        PointType::LineRight => Color32::from_rgb(255, 255, 0),
        PointType::Obstacle => Color32::from_rgb(255, 0, 255),
        PointType::ArrowLeft => Color32::from_rgb(100, 100, 100),
        PointType::ArrowRight => Color32::from_rgb(100, 100, 100),
    }
}
