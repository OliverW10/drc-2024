use crate::{
    config::file::ConfigReader, points::{Point, PointMap, PointType, Pos}, pruner::get_point_expiry, state::CarState
};

use super::{perspective::PerspectiveTransformPoints, ObjectFinder};

pub struct FakePointProvider {}

struct Lines {
    point_type: PointType,
    lines: Vec<[Pos; 2]>,
}

fn jitter() -> Pos {
    Pos {
        x: (rand::random::<f64>() - 0.5) * 0.1,
        y: (rand::random::<f64>() - 0.5) * 0.1,
    }
}

impl ObjectFinder for FakePointProvider {
    fn get_points(&mut self, _: &opencv::core::Mat, _: &CarState, _: &mut ConfigReader<PerspectiveTransformPoints>, point_map: &dyn PointMap) -> Result<Vec<Point>, opencv::Error> {
        let all_lines = vec![
            Lines {
                point_type: PointType::LeftLine,
                lines: vec![
                    [Pos { x: -0.5, y: 0.5 }, Pos { x: -0.5, y: -3.5 }],
                    [Pos { x: -0.5, y: -3.5 }, Pos { x: 3.5, y: -3.5 }],
                    [Pos { x: 3.5, y: -3.5 }, Pos { x: 3.5, y: 0.5 }],
                    [Pos { x: 3.5, y: 0.5 }, Pos { x: -0.5, y: 0.5 }],
                    [Pos { x: -4.0, y: -4.0 }, Pos { x: -4.0, y: 4.0 }],
                ],
            },
            Lines {
                point_type: PointType::RightLine,
                lines: vec![
                    [Pos { x: 0.5, y: -0.5 }, Pos { x: 0.5, y: -2.5 }],
                    [Pos { x: 0.5, y: -2.5 }, Pos { x: 2.5, y: -2.5 }],
                    [Pos { x: 2.5, y: -2.5 }, Pos { x: 2.5, y: -0.5 }],
                    [Pos { x: 2.5, y: -0.5 }, Pos { x: 0.5, y: -0.5 }],
                    [Pos { x: -1.5, y: -4.0 }, Pos { x: -1.5, y: 4.0 }],
                ],
            },
        ];

        let mut points = vec![
            Point {
                id: rand::random(),
                expire_at: get_point_expiry(Pos { x: -2.75, y: 0.0 }, point_map),
                pos: Pos { x: -2.75, y: 0.0 },
                point_type: PointType::ArrowLeft,
            },
            Point {
                id: rand::random(),
                expire_at: get_point_expiry(Pos { x: -2.75, y: -2.5 }, point_map),
                pos: Pos { x: -2.75, y: -2.5 },
                point_type: PointType::ArrowRight,
            },
        ];
        for lines_of_type in all_lines {
            for line in lines_of_type.lines {
                let line_dist = line[0].dist(line[1]);
                for _ in 0..10 {
                    let pos = line[0].dist_along(line[1], rand::random::<f64>() * line_dist) + jitter();
                    points.push(Point {
                        id: rand::random(),
                        expire_at: get_point_expiry(pos, point_map),
                        point_type: lines_of_type.point_type,
                        pos
                    });
                }
            }
        }
        Ok(points)
    }
}
