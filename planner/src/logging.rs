use crate::{config::plan::PLAN_STEP_SIZE_METERS, messages, planner, points};
use prost::Message;
use std::{fs::File, io::Write};
use time::OffsetDateTime;

// Something to log structured diagnostic and debgging info such as sensor readings, and planned actions
pub trait Logger {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic);

    // Converts the functional types to the protobuf types which are actually sent with send_messages
    fn send(
        &mut self,
        path: &planner::Path,
        new_points: &Vec<points::Point>,
        removed_points: &Vec<u32>,
        diagnostic: &messages::diagnostic::Diagnostic,
    ) {
        let path_dto = Some(messages::path::Path {
            point_interval: PLAN_STEP_SIZE_METERS as f32,
            points: path
                .points
                .iter()
                .map(|p| messages::path::PathPoint {
                    x: p.pos.x as f32,
                    y: p.pos.y as f32,
                    angle: p.angle as f32,
                    curvature: p.curvature as f32,
                })
                .collect(),
        });

        let new_points_dtos = new_points
            .iter()
            .map(|p| messages::path::MapPoint {
                x: p.pos.x as f32,
                y: p.pos.y as f32,
                point_type: match p.point_type {
                    points::PointType::LeftLine => messages::path::PointType::LineLeft,
                    points::PointType::RightLine => messages::path::PointType::LineRight,
                    points::PointType::ArrowLeft => messages::path::PointType::ArrowLeft,
                    points::PointType::ArrowRight => messages::path::PointType::ArrowRight,
                    points::PointType::Obstacle => messages::path::PointType::Obstacle,
                }
                .into(),
                id: p.id,
            })
            .collect();
        let map_update_dto = Some(messages::path::MapUpdate {
            points_added: new_points_dtos,
            removed_ids: removed_points.to_vec(),
        });

        let diagnostic_dto = Some(diagnostic.clone());

        self.send_core(&messages::diagnostic::FullDiagnostic {
            path: path_dto,
            map_update: map_update_dto,
            diagnostic: diagnostic_dto,
        });
    }
}

pub struct FileLogger {
    file: File,
}

impl FileLogger {
    pub fn new() -> FileLogger {
        FileLogger {
            file: File::create(get_new_log_file_name()).unwrap(),
        }
    }
}

pub fn get_new_log_file_name() -> String {
    let now = OffsetDateTime::now_utc();
    format!("session_{now}.log").to_string()
}

impl Logger for FileLogger {
    fn send_core(&mut self, message: &messages::diagnostic::FullDiagnostic) {
        let message = message.encode_length_delimited_to_vec();
        match self.file.write(&message) {
            Err(e) => println!("error writing log file {}", e),
            Ok(n) => {}
        };
    }
}
