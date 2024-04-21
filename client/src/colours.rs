use eframe::egui::Color32;

use crate::messages::path::PointType;

pub fn point_colour(point_type: &PointType) -> Color32 {
    match point_type {
        PointType::LineLeft => Color32::BLUE,
        PointType::LineRight => Color32::YELLOW,
        PointType::Obstacle => Color32::DARK_RED,
        PointType::ArrowLeft => Color32::WHITE,
        PointType::ArrowRight => Color32::WHITE,
    }
}
