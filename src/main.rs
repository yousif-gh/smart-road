mod intersection;
mod route;
mod stats;
mod vehicle;
mod world;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::rwops::RWops;
use sdl2::surface::Surface;
use std::time::{Duration, Instant};
use vehicle::{CAR_LEN, CAR_W, Dir, Route, SPRITE_COUNT, Vehicle, can_spawn};

const CAR_SHEET: &[u8] = include_bytes!("../assets/cars.bmp");
const SHEET_W: u32 = 40;
const SHEET_H: u32 = 68;

const RANDOM_SPAWN_EVERY: f32 = 0.5;

fn try_spawn(cars: &mut Vec<Vehicle>, dir: Dir) {
    let route = Route::random();
    if can_spawn(cars, dir, route) {
        cars.push(Vehicle::new(dir, route));
    }
}

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("Smart Road", world::SIZE as u32, world::SIZE as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let mut events = sdl.event_pump()?;

    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "1");
    let tc = canvas.texture_creator();
    let sheet = Surface::load_bmp_rw(&mut RWops::from_bytes(CAR_SHEET)?)?;
    assert_eq!(sheet.width(), SHEET_W * SPRITE_COUNT as u32, "cars.bmp size");
    let sprites = tc
        .create_texture_from_surface(&sheet)
        .map_err(|e| e.to_string())?;

    let intersection = intersection::Intersection::new();
    let mut cars: Vec<Vehicle> = Vec::new();
    let mut stats = stats::Stats::new();
    let mut random_mode = false;
    let mut spawn_timer = 0.0;
    let mut last = Instant::now();

    'running: loop {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32().min(0.05);
        last = now;

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Up => try_spawn(&mut cars, Dir::North),
                    Keycode::Down => try_spawn(&mut cars, Dir::South),
                    Keycode::Right => try_spawn(&mut cars, Dir::East),
                    Keycode::Left => try_spawn(&mut cars, Dir::West),
                    Keycode::R => random_mode = !random_mode,
                    _ => {}
                },
                _ => {}
            }
        }

        if random_mode {
            spawn_timer += dt;
            if spawn_timer >= RANDOM_SPAWN_EVERY {
                spawn_timer = 0.0;
                try_spawn(&mut cars, Dir::random());
            }
        }

        vehicle::follow(&mut cars);
        intersection.update(&mut cars);
        for car in &mut cars {
            car.update(dt);
        }
        stats.observe(&cars);
        cars.retain(|c| {
            let gone = c.off_screen();
            if gone {
                stats.record_exit(c);
            }
            !gone
        });

        world::draw_roads(&mut canvas)?;
        for car in &cars {
            let dst = Rect::new(
                (car.x - CAR_W / 2.0) as i32,
                (car.y - CAR_LEN / 2.0) as i32,
                CAR_W as u32,
                CAR_LEN as u32,
            );
            canvas.copy_ex(
                &sprites,
                Rect::new(car.sprite as i32 * SHEET_W as i32, 0, SHEET_W, SHEET_H),
                dst,
                car.angle as f64,
                None,
                false,
                false,
            )?;
        }
        canvas.present();
        std::thread::sleep(Duration::from_millis(1));
    }

    canvas.window_mut().hide();
    stats::show(&video, &mut events, &stats)
}
