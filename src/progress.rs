use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct Progress {
    total_work: u64,
    current_progress: u64,
    output: Arc<Mutex<dyn Write + Send + Sync>>,
    progress_bar_display_length: u32,
    last_update_percentage: f32,
}

const UPDATE_DELTA: f32 = 0.009_f32;

const PROGRESS_PREFIX: &str = "\rProgress: <";
const FILLED_CHAR: char = '#';
const EMPTY_CHAR: char = ' ';

// TODO: Bubble errors through Result rather than use unwrap()
impl Progress {
    pub fn new(
        total_work: u64,
        output: Arc<Mutex<dyn Write + Send + Sync>>,
        progress_bar_display_length: u32,
    ) -> Progress {
        Progress {
            total_work: total_work,
            current_progress: 0_u64,
            output: output,
            progress_bar_display_length: progress_bar_display_length,
            last_update_percentage: 0.0_f32,
        }
    }

    pub fn update(&mut self, progress_made: u64) {
        self.current_progress =
            std::cmp::min(self.current_progress + progress_made, self.total_work);
        let new_percentage = (self.current_progress as f64 / self.total_work as f64) as f32;
        if (new_percentage - self.last_update_percentage).abs() > UPDATE_DELTA {
            self.print_progress(new_percentage);
        }
    }

    pub fn done(&mut self) {
        self.current_progress = 0_u64;
        self.last_update_percentage = 0.0_f32;
        (*self.output.lock().unwrap())
            .write("\n".as_bytes())
            .unwrap();
    }

    fn print_progress(&mut self, percentage: f32) {
        self.last_update_percentage = percentage;

        let num_filled = (percentage * self.progress_bar_display_length as f32).round() as u32;
        let mut p = String::with_capacity(
            PROGRESS_PREFIX.len() + self.progress_bar_display_length as usize + 8_usize,
        );

        p.push_str(PROGRESS_PREFIX);
        for _ in 0..num_filled {
            p.push(FILLED_CHAR);
        }
        for _ in 0..(self.progress_bar_display_length - num_filled) {
            p.push(EMPTY_CHAR);
        }
        p.push_str(format!("> ({}%)", (percentage * 100.0_f32).round() as u32).as_str());

        let mut_output = &mut *self.output.lock().unwrap();
        mut_output.flush().unwrap();
        mut_output.write(p.as_bytes()).unwrap();
        mut_output.flush().unwrap();
    }
}
