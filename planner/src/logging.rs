use std::{fs::File, io::Write};
use prost::Message;
use time::OffsetDateTime;
use crate::{config::plan::PLAN_STEP_SIZE_METERS, messages::{self, diagnostic::Diagnostic}, planner, points, state};


// Something to log structured diagnostic and debgging info such as sensor readings, and planned actions
pub trait Logger {
    fn send_messages(&mut self, path: &messages::path::Path, new_points: &messages::path::MapUpdate, diagnostic: &messages::diagnostic::Diagnostic);

    // Converts the internal types to the protobuf types which are actually sent with send_messages
    fn send(&mut self, path: &planner::Path, new_points: &Vec<points::Point>, num_deleted: u32, diagnostic: &messages::diagnostic::Diagnostic){

        let path_dto = messages::path::Path {
            point_interval: PLAN_STEP_SIZE_METERS as f32,
            points: path.points.iter().map(|p| messages::path::PathPoint {
                x: p.pos.x as f32,
                y: p.pos.y as f32,
                angle: p.angle as f32,
                curvature: p.curvature as f32,
            }).collect()
        };
        
        let new_points_dtos = new_points.iter().map(|p| messages::path::MapPoint {
            x: p.pos.x as f32,
            y: p.pos.y as f32,
            point_type: match p.point_type {
                points::PointType::LeftLine => messages::path::PointType::LineLeft,
                points::PointType::RightLine => messages::path::PointType::LineRight,
                points::PointType::ArrowLeft => messages::path::PointType::ArrowLeft,
                points::PointType::ArrowRight => messages::path::PointType::ArrowRight,
                points::PointType::Obstacle => messages::path::PointType::Obstacle,
            }.into()
        }).collect();
        let map_update_dto = messages::path::MapUpdate {
            points_added: new_points_dtos,
            num_deleted: num_deleted,
        };

        let diagnostic_dto = diagnostic;

        self.send_messages(&path_dto, &map_update_dto, diagnostic_dto);
    }
}

pub struct FileLogger {
    file: File
}

impl FileLogger {
    pub fn new() -> FileLogger {
        FileLogger {
            file: File::create(get_new_log_file_name()).unwrap()
        }
    }
}

pub fn get_new_log_file_name() -> String {
    let now = OffsetDateTime::now_utc();
    format!("session_{now}.log").to_string()
}

impl Logger for FileLogger {
    fn send_messages(&mut self, path: &messages::path::Path, new_points: &messages::path::MapUpdate, diagnostic: &messages::diagnostic::Diagnostic) {
        let buffer = path.encode_to_vec();
        match self.file.write(&buffer) {
            Err(e) => println!("error! {}", e),
            Ok(n) => println!("write {} bytes", n),
        };
    }
}


// Simple struct to multicast logging
pub struct AggregateLogger {
    loggers: Vec<Box<dyn Logger>>
}

impl AggregateLogger {
    pub fn new(loggers: Vec<Box<dyn Logger>>) -> AggregateLogger {
        AggregateLogger { loggers: loggers }
    }
}

impl Logger for AggregateLogger {
    fn send_messages(&mut self, path: &messages::path::Path, new_points: &messages::path::MapUpdate, diagnostic: &messages::diagnostic::Diagnostic) {
        for logger in self.loggers.iter_mut() {
            logger.send_messages(path, new_points, diagnostic);
        }
    }
}
