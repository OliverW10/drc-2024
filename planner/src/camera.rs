use std::collections::HashSet;

use opencv::{
    core::{Mat, MatTraitConst, Vector}, highgui, imgcodecs::imwrite, videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst, CAP_PROP_POS_FRAMES}
};
use time::OffsetDateTime;

use crate::{config::display::SHOULD_DISPLAY_RAW_VIDEO, display::annotate_video, messages};

pub struct Capture {
    inner: videoio::VideoCapture,
    frame: Mat,
    needs_restarting: bool,
}

impl Capture {
    pub fn camera() -> Capture {
        println!("Opening camera");
        let mut cap = videoio::VideoCapture::new(0, videoio::CAP_V4L2).unwrap();

        let opened = videoio::VideoCapture::is_opened(&cap).unwrap();
        if !opened {
            panic!("Unable to open default camera!");
        }

        // if config::is_running_on_pi() {
            cap.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0).unwrap();
            cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0).unwrap();
        // } else {
        //     cap.set(videoio::CAP_PROP_FRAME_WIDTH, 480.0).unwrap();
        //     cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 640.0).unwrap();
        // }

        Capture {
            inner: cap,
            frame: Mat::default(),
            needs_restarting: false,
        }
    }

    pub fn video(filename: &str) -> Capture {
        println!("Opening file {}", filename);
        let mut cap = videoio::VideoCapture::from_file_def(filename).unwrap();

        let opened = videoio::VideoCapture::is_opened(&cap).unwrap();
        if !opened {
            panic!("Unable to open video file!");
        }
        cap.set(videoio::CAP_PROP_FPS, 60.0).unwrap();

        Capture {
            inner: cap,
            frame: Mat::default(),
            needs_restarting: true,
        }
    }

    pub fn get_frame(&mut self) -> Option<&Mat> {
        puffin::profile_function!();
        let got_frame = self.inner.read(&mut self.frame).unwrap_or_default();

        if !got_frame && self.needs_restarting {
            self.inner.set(CAP_PROP_POS_FRAMES, 0.0).ok()?;
            println!("restarting video");
            return self.get_frame();
        }

        if display_image_and_get_key(&self.frame) {
            return None;
        }

        return if got_frame { Some(&self.frame) } else { None };
    }
}

pub fn display_image_and_get_key(_frame: &Mat) -> bool {
    if !SHOULD_DISPLAY_RAW_VIDEO {
        return false;
    }
    // dont want to draw on actual image
    let mut frame = _frame.clone();
    annotate_video(&mut frame);
    if frame.size().unwrap().width > 0 {
        highgui::imshow("window", &frame).unwrap();
    }

    let key = highgui::wait_key(20).unwrap();
    if key > 0 && key != 255 {
        println!("Got key press {key}");
        return true;
    }
    return false;
}

#[derive(Default)]
pub struct Recorder {
    images: i32,
    blues: i32,
    yellows: i32,
    needs: HashSet<String>
}

impl Recorder {
    pub fn enqueue_images(&mut self, cmd: &messages::command::DriveCommand) {
        if self.images < cmd.images_frame as i32 {
            self.images += 1;
            self.needs.insert("image".to_owned());
        }
        if self.blues < cmd.images_blue as i32 {
            self.blues += 1;
            self.needs.insert("blue".to_owned());
        }
        if self.yellows < cmd.images_yellow as i32 {
            self.yellows += 1;
            self.needs.insert("yellow".to_owned());
        }
    }

    pub fn record_image(&mut self, img: &Mat, desc: &str) {
        if self.needs.contains(desc) {
            println!("taking image of {}", desc);
            let now = OffsetDateTime::now_utc();
            let params = Vector::<i32>::new(); // required arguemtn
            let _ = imwrite(format!("images/{}-{}.png", desc, now.to_string()).as_str(), img, &params);
            self.needs.remove(desc);
        }
    }
}