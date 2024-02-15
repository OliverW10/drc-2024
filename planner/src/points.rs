
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
        OBSTACLE,
        ARROW_LEFT,
        ARROW_RIGHT
    }

    pub struct Point {
        pub pos: Pos,
        pub confidence: f64,
        pub point_type: PointType
    }

    trait PointMap {
        fn get_points(&self, around: Pos, max_dist: f64);
    }

    struct SimplePointMap {
        all_points: Vec<Point>,
    }

    impl PointMap for SimplePointMap {
        fn get_points(&self, around: Pos, max_dist: f64) -> Vec<Point>{
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
        grid: [[Vec<Point>; N]; N], // 2d array of vectors of points
    }

    impl PointMap for GridPointMap {
        fn get_points(&self, around: Pos, max_dist: f64) -> Vec<Point>{
            
        }
    }
}