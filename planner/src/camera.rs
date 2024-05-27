use opencv::{
    core::{Mat, MatTraitConst},
    highgui,
    videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst, CAP_PROP_POS_FRAMES},
};

const SHOULD_DISPLAY_VIDEO: bool = true;

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
        cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 640.0).unwrap();
        cap.set(videoio::CAP_PROP_FRAME_WIDTH, 480.0).unwrap();

        Capture { inner: cap, frame: Mat::default(), needs_restarting: false }
    }

    pub fn video(filename: &str) -> Capture {
        println!("Opening file {}", filename);
        let cap = videoio::VideoCapture::from_file_def(filename).unwrap();

        let opened = videoio::VideoCapture::is_opened(&cap).unwrap();
        if !opened {
            panic!("Unable to open video file!");
        }

        Capture { inner: cap, frame: Mat::default(), needs_restarting: true }
    }

    pub fn get_frame(&mut self) -> Option<&Mat> {
        puffin::profile_function!();
        let got_frame = self.inner.read(&mut self.frame).unwrap_or_default();

        if !got_frame && self.needs_restarting {
            self.inner.set(CAP_PROP_POS_FRAMES, 0.0).ok()?;
        }
        
        if display_image_and_get_key(&self.frame) {
            return None;
        }

        return if got_frame {
            Some(&self.frame)
        } else {
            None
        }
    }
}

fn display_image_and_get_key(frame: &Mat) -> bool {
    if !SHOULD_DISPLAY_VIDEO {
        return false;
    }

    if frame.size().unwrap().width > 0 {
        highgui::imshow("window", &frame).unwrap();
    }

    let key = highgui::wait_key(10).unwrap();
    if key > 0 && key != 255 {
        println!("Got key press {key}");
        return true;
    }
    return false;
}
