use crate::{points::{Point, PointType, Pos}, pruner::get_line_exiry, state::CarState};

use super::ObjectFinder;


pub struct FakePointProvider {

}

struct Lines {
    point_type: PointType,
    lines: Vec<[Pos; 2]>
}

impl ObjectFinder for FakePointProvider {
    fn get_points(&mut self, image: &opencv::core::Mat, state: &CarState) -> Result<Vec<Point>, opencv::Error> {
        let all_lines = vec![
            Lines {
                point_type: PointType::LeftLine,
                lines: vec![
                    [Pos {x: -0.5, y: 0.5}, Pos {x: -0.5, y: -3.5}],
                    [Pos {x: -0.5, y: -3.5}, Pos {x: 3.5, y: -3.5}],
                ]
            },
            Lines {
                point_type: PointType::RightLine,
                lines: vec![
                    [Pos {x: 0.5, y: -0.5}, Pos {x: 0.5, y: -2.5}],
                    [Pos {x: 0.5, y: -2.5}, Pos {x: 2.5, y: -2.5}],
                ]
            }
        ];

        let mut points = Vec::new();
        let expiry = get_line_exiry();
        for lines_of_type in all_lines {
            for line in lines_of_type.lines {
                let line_dist = line[0].dist(line[1]);
                for _ in 0..30 {
                    points.push(Point {
                        id: rand::random(),
                        expire_at: expiry,
                        point_type: lines_of_type.point_type,
                        pos: line[0].dist_along(line[1], rand::random::<f64>() * line_dist),
                    });
                }
            }
        }
        Ok(points)
    }
}
