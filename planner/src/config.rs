pub mod file {
    use std::{fs, time::SystemTime};

    pub struct ConfigReader<T> {
        last_edit_time: SystemTime,
        filename: String,
        reader: fn(&str) -> T,
        last_value: T,
    }

    impl<T> ConfigReader<T> {
        pub fn new(filename: &str, reader: fn(&str) -> T) -> ConfigReader<T> {
            ConfigReader {
                last_edit_time: Self::get_last_edit_time(filename),
                filename: filename.to_string(),
                reader: reader,
                last_value: Self::read_file(filename, reader),
            }
        }

        fn get_last_edit_time(filename: &str) -> SystemTime {
            fs::metadata(filename).and_then(|metadata| metadata.modified()).unwrap()
        }

        pub fn get_value(&mut self) -> &T {
            let new_last_edit_time = Self::get_last_edit_time(self.filename.as_str());
            if new_last_edit_time > self.last_edit_time {
                self.last_value = Self::read_file(self.filename.as_str(), self.reader);
                self.last_edit_time = new_last_edit_time;
            }
            &self.last_value
        }

        fn read_file(filename: &str, reader: fn(&str) -> T) -> T {
            println!("reading file");
            let file_contents = fs::read_to_string(filename).unwrap();
            reader(file_contents.as_str())
        }
    }
}

pub mod colours {
    use opencv::core::VecN;

    pub struct ColourRange {
        pub low: VecN<u8, 3>,
        pub high: VecN<u8, 3>,
    }

    // TODO: use config file
    const fn c<T>(a: T, b: T, c: T) -> VecN<T, 3> {
        VecN::<T, 3> { 0: [a, b, c] }
    }
    const fn r(l: VecN<u8, 3>, h: VecN<u8, 3>) -> ColourRange {
        ColourRange { low: l, high: h }
    }
    pub const YELLOW_MASK: ColourRange = r(c(36, 19, 154), c(58, 255, 255));
    pub const BLUE_MASK: ColourRange = r(c(87, 54, 95), c(122, 255, 255));
    // pub const BLACK_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    // pub const PURPLE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    // pub const RED_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
}

pub mod plan {
    pub const PLAN_STEP_SIZE_METERS: f64 = 0.2;
    pub const PLAN_MAX_LENGTH_METERS: f64 = 3.0;
    pub const PLAN_MAX_STEPS: u32 = (PLAN_MAX_LENGTH_METERS / PLAN_STEP_SIZE_METERS) as u32;

    pub const MAX_CURVATURE: f64 = 1.0 / 0.3;
}

pub mod image {
    use opencv::core::Rect;

    pub const TOP_CROP: i32 = 20;
    pub const EXCLUDE_RECT: Rect = Rect {
        x: 0, //x: 195,
        y: 360,
        width: 640, // width: 270,
        height: 120,
    };
}

pub mod display {
    use super::is_running_on_pi;

    pub const SHOULD_DISPLAY_RAW_VIDEO: bool = false && !is_running_on_pi();
}

#[inline]
pub const fn is_running_on_pi() -> bool {
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return true;
    }
    return false;
}
