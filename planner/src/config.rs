pub mod file {
    use std::{fs, time::{Duration, SystemTime}};
    use serde::{Deserialize, Serialize};

    use super::colours::ColourRange;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PerspectiveConfig {
        pub image: Vec<Vec<f32>>,
        pub ground: Vec<Vec<f32>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ColourConfig {
        pub low: Vec<i32>,
        pub high: Vec<i32>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DriveConfig {
        pub odom_speed_fudge: f32,
        pub odom_turn_fudge: f32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Config {
        pub perspective: PerspectiveConfig,
        pub yellow_colour: ColourConfig,
        pub blue_colour: ColourConfig,
        pub drive_cfg: DriveConfig,
    }

    pub enum LineColour {
        YELLOW, BLUE
    }

    impl Config {
        pub fn colour_for_line(&self, col: &LineColour) -> ColourRange {
            match col {
                LineColour::BLUE => self.blue_colour.to_opencv_range(),
                LineColour::YELLOW => self.yellow_colour.to_opencv_range(),
            }
        }
    }
    pub struct ConfigReader<T> {
        last_edit_time: SystemTime,
        filename: String,
        reader: fn(&str) -> T,
        last_value: T,
    }

    const FILE_READ_MIN: Duration = Duration::from_millis(200);

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
            if self.last_edit_time.elapsed().unwrap() > FILE_READ_MIN {
                return &self.last_value;
            }

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

    use super::file::ColourConfig;

    pub struct ColourRange {
        pub low: VecN<u8, 3>,
        pub high: VecN<u8, 3>,
    }

    impl ColourConfig {
        pub fn to_opencv_range(&self) -> ColourRange{
            ColourRange {
                low: VecN::<u8, 3> { 0: [ self.low[0] as u8, self.low[1] as u8, self.low[2] as u8] },
                high: VecN::<u8, 3> { 0: [ self.high[0] as u8, self.high[1] as u8, self.high[2] as u8] },
            }
        }
    }

    // pub const BLACK_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    // pub const PURPLE_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
    // pub const RED_MASK: ColourRange = r(c(110, 50, 100), c(120, 255, 255));
}

pub mod image {
    use opencv::core::Rect;

    pub const TOP_CROP: i32 = 90;
    pub const BOTTOM_CROP: i32 = 30;
    pub const EXCLUDE_RECT: Rect = Rect {
        x: 0, //x: 195,
        y: 360,
        width: 640, // width: 270,
        height: 120,
    };
}

pub mod display {
    use super::is_running_on_pi;

    pub const SHOULD_DISPLAY_RAW_VIDEO: bool = true && !is_running_on_pi();
}

#[inline]
pub const fn is_running_on_pi() -> bool {
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return true;
    }
    return false;
}
