use opencv::{
    core::{Mat, MatTraitConst},
    highgui,
    videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst},
};

const SHOULD_DISPLAY_VIDEO: bool = false;

pub trait ImageProvider {
    fn get_frame(&mut self) -> Option<&Mat>;
}

pub struct Camera {
    cap: videoio::VideoCapture,
    frame: Mat,
}

impl Camera {
    pub fn new() -> Camera {
        let mut cap = videoio::VideoCapture::new(0, videoio::CAP_V4L2).unwrap();

        let opened = videoio::VideoCapture::is_opened(&cap).unwrap();
        if !opened {
            panic!("Unable to open default camera!");
        }
        cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 640.0).unwrap();
        cap.set(videoio::CAP_PROP_FRAME_WIDTH, 480.0).unwrap();
        let frame = Mat::default();

        Camera { cap: cap, frame: frame }
    }
}

impl ImageProvider for Camera {
    fn get_frame(&mut self) -> Option<&Mat> {
        puffin::profile_function!();

        self.cap.read(&mut self.frame).unwrap();
        if SHOULD_DISPLAY_VIDEO {
            if self.frame.size().unwrap().width > 0 {
                highgui::imshow("window", &self.frame).unwrap();
            }

            let key = highgui::wait_key(10).unwrap();
            if key > 0 && key != 255 {
                return None;
            }
        }

        Some(&self.frame)
    }
}

struct Video {}

impl Video {
    // pub fn new() -> Video {
    //     Video {}
    // }
}

impl ImageProvider for Video {
    fn get_frame(&mut self) -> Option<&Mat> {
        None
    }
}
