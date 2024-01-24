
mod points{

    pub struct Pos {
        pub x: f64,
        pub y: f64
    }

    impl Pos {
        fn dist(&self, other: Pos) -> f64{
            let dx = self.x - other.x;
            let dy = self.y - other.y;
            (dx * dx + dy * dy).sqrt()
        }
    }

    enum PointType {
        LEFT,
        RIGHT,
        OBSTACLE
    }

    pub struct Point {
        pub pos: Pos,
        pub confidence: f64
    }

    trait PointMap {
        fn get_points(&self, around: Pos, max_dist: f64, point_type: PointType);
    }

    struct SimplePointMap {
        left_line: Vec<Point>,
        right_line: Vec<Point>,
        obstacles: Vec<Point>
    }

    impl PointMap for SimplePointMap {
        fn get_points(&self, around: Pos, max_dist: f64, point_type: PointType){
            let relevant_points = match point_type {
                PointType::LEFT => self.left_line,
                PointType::RIGHT => self.right_line,
                PointType::OBSTACLE => self.obstacles,
            };
            let output_points = Vec::new();
            for point in relevant_points {
                if(point.dist(around) < max_dist){
                    output_points.push(point);
                }
            }
            output_points
        }
    }

    struct GridPointMap<N: usize>{
        left_line_grid: [[Vec<Point>; N]; N],
        right_line_grid: [[Vec<Point>; N]; N],
        obstacles_grid: [[Vec<Point>; N]; N],
    }

    impl PointMap for GridPointMap {
        fn get_points(&self, around: Pos, max_dist: f64, point_type: PointType){
            
        }
    }
}