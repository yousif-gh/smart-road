use crate::vehicle::{SAFE_GAP, Vehicle, overlaps};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::rwops::RWops;
use sdl2::{EventPump, VideoSubsystem};
use std::collections::HashSet;

const FONT: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

pub struct Stats {
    passed: u32,
    max_v: f32,
    min_v: f32,
    max_t: f32,
    min_t: f32,
    close_calls: HashSet<(u64, u64)>,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            passed: 0,
            max_v: 0.0,
            min_v: f32::INFINITY,
            max_t: 0.0,
            min_t: f32::INFINITY,
            close_calls: HashSet::new(),
        }
    }

    pub fn observe(&mut self, cars: &[Vehicle]) {
        for c in cars.iter().filter(|c| c.speed > 0.0) {
            self.max_v = self.max_v.max(c.speed);
            self.min_v = self.min_v.min(c.speed);
        }
        let zone = SAFE_GAP - 2.0;
        for (i, a) in cars.iter().enumerate() {
            for b in &cars[i + 1..] {
                if overlaps(a, zone, b) || overlaps(b, zone, a) {
                    self.close_calls.insert((a.id.min(b.id), a.id.max(b.id)));
                }
            }
        }
    }

    pub fn record_exit(&mut self, car: &Vehicle) {
        self.passed += 1;
        if let Some(start) = car.detected_at {
            let t = car.time - start;
            self.max_t = self.max_t.max(t);
            self.min_t = self.min_t.min(t);
        }
    }

    #[cfg(test)]
    pub fn max_time(&self) -> f32 {
        self.max_t
    }

    pub fn lines(&self) -> Vec<String> {
        let opt = |v: f32, unit: &str| {
            if v.is_finite() && self.passed > 0 {
                format!("{v:.1} {unit}")
            } else {
                "-".into()
            }
        };
        vec![
            format!("Vehicles passed: {}", self.passed),
            format!("Max velocity: {}", opt(self.max_v, "px/s")),
            format!("Min velocity: {}", opt(self.min_v, "px/s")),
            format!("Max time: {}", opt(self.max_t, "s")),
            format!("Min time: {}", opt(self.min_t, "s")),
            format!("Close calls: {}", self.close_calls.len()),
        ]
    }
}

pub fn show(video: &VideoSubsystem, events: &mut EventPump, stats: &Stats) -> Result<(), String> {
    let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let title = ttf.load_font_from_rwops(RWops::from_bytes(FONT)?, 30)?;
    let body = ttf.load_font_from_rwops(RWops::from_bytes(FONT)?, 22)?;

    let window = video
        .window("Smart Road - Statistics", 460, 380)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let tc = canvas.texture_creator();

    let white = Color::RGB(235, 235, 235);
    let mut texts = Vec::new();
    let mut y = 24;
    let rows = std::iter::once(("Statistics".to_string(), &title))
        .chain(stats.lines().into_iter().map(|l| (l, &body)));
    for (line, font) in rows {
        let surf = font
            .render(&line)
            .blended(white)
            .map_err(|e| e.to_string())?;
        let tex = tc
            .create_texture_from_surface(&surf)
            .map_err(|e| e.to_string())?;
        let h = surf.height();
        texts.push((tex, Rect::new(30, y, surf.width(), h)));
        y += h as i32 + if texts.len() == 1 { 20 } else { 12 };
    }

    loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape | Keycode::Return),
                    ..
                } => {
                    return Ok(());
                }
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(40, 44, 52));
        canvas.clear();
        for (tex, dst) in &texts {
            canvas.copy(tex, None, *dst)?;
        }
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vehicle::{Dir, Route};

    #[test]
    fn records_min_max_and_counts_close_call_once() {
        let mut s = Stats::new();
        let mut a = Vehicle::new(Dir::East, Route::Straight);
        a.update(1.0);
        let mut b = Vehicle::new(Dir::East, Route::Straight);
        b.update(1.0);
        b.distance -= crate::vehicle::CAR_LEN + 5.0;
        b.update(0.0);
        b.speed = 60.0;
        s.observe(&[a, b]);
        let cars = s.close_calls.len();
        assert_eq!(cars, 1);

        let mut c = Vehicle::new(Dir::North, Route::Left);
        c.detected_at = Some(1.0);
        c.time = 4.5;
        s.record_exit(&c);
        c.time = 2.0;
        s.record_exit(&c);
        assert_eq!((s.passed, s.max_t, s.min_t), (2, 3.5, 1.0));
        assert_eq!((s.max_v, s.min_v), (200.0, 60.0));
    }
}
