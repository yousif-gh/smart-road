use crate::route;
use crate::world::SIZE;
use std::sync::atomic::{AtomicU64, Ordering};

pub const CAR_W: f32 = 20.0;
pub const CAR_LEN: f32 = 34.0;
pub const SAFE_GAP: f32 = 20.0;
pub const SLOW: f32 = 60.0;
pub const MEDIUM: f32 = 120.0;
pub const FAST: f32 = 200.0;
pub const SPRITE_COUNT: usize = 5;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Dir {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    Left,
    Straight,
    Right,
}

impl Route {
    pub fn random() -> Route {
        [Route::Left, Route::Straight, Route::Right][rand::random_range(0..3)]
    }
}

impl Dir {
    pub fn random() -> Dir {
        [Dir::North, Dir::South, Dir::East, Dir::West][rand::random_range(0..4)]
    }
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct Vehicle {
    pub id: u64,
    pub dir: Dir,
    pub route: Route,
    pub speed: f32,
    pub distance: f32,
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub sprite: usize,
    pub reserved: bool,
    pub detected_at: Option<f32>,
}

impl Vehicle {
    pub fn new(dir: Dir, route: Route) -> Vehicle {
        let (x, y, angle) = route::pose(dir, route, 0.0);
        let sprite = rand::random_range(0..SPRITE_COUNT);
        Vehicle {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            dir,
            route,
            speed: FAST,
            distance: 0.0,
            time: 0.0,
            x,
            y,
            angle,
            sprite,
            reserved: false,
            detected_at: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.distance += self.speed * dt;
        self.time += dt;
        (self.x, self.y, self.angle) = route::pose(self.dir, self.route, self.distance);
    }

    pub fn off_screen(&self) -> bool {
        let m = CAR_LEN;
        let s = SIZE as f32;
        self.x < -m || self.x > s + m || self.y < -m || self.y > s + m
    }
}

pub fn overlaps(a: &Vehicle, ahead: f32, b: &Vehicle) -> bool {
    let axes = |v: &Vehicle| {
        let r = v.angle.to_radians();
        [(r.sin(), -r.cos()), (r.cos(), r.sin())]
    };
    let [af, _] = axes(a);
    let a_box = (
        a.x + af.0 * ahead / 2.0,
        a.y + af.1 * ahead / 2.0,
        (CAR_LEN + ahead) / 2.0,
    );
    let b_box = (b.x, b.y, CAR_LEN / 2.0);
    let project = |v: &Vehicle, (cx, cy, half_len): (f32, f32, f32), ax: (f32, f32)| {
        let [f, r] = axes(v);
        let c = cx * ax.0 + cy * ax.1;
        let e = half_len * (f.0 * ax.0 + f.1 * ax.1).abs()
            + CAR_W / 2.0 * (r.0 * ax.0 + r.1 * ax.1).abs();
        (c - e, c + e)
    };
    axes(a).into_iter().chain(axes(b)).all(|ax| {
        let (a0, a1) = project(a, a_box, ax);
        let (b0, b1) = project(b, b_box, ax);
        a0 < b1 && b0 < a1
    })
}

pub fn speed_for_gap(gap: f32) -> f32 {
    if gap <= SAFE_GAP {
        0.0
    } else if gap <= SAFE_GAP + 30.0 {
        SLOW
    } else if gap <= SAFE_GAP + 80.0 {
        MEDIUM
    } else {
        FAST
    }
}

pub fn gap_ahead(cars: &[Vehicle], i: usize) -> Option<f32> {
    let me = &cars[i];
    cars.iter()
        .filter(|c| c.dir == me.dir && c.route == me.route && c.distance > me.distance)
        .map(|c| c.distance - me.distance - CAR_LEN)
        .min_by(f32::total_cmp)
}

pub fn follow(cars: &mut [Vehicle]) {
    let speeds: Vec<f32> = (0..cars.len())
        .map(|i| gap_ahead(cars, i).map_or(FAST, speed_for_gap))
        .collect();
    for (car, s) in cars.iter_mut().zip(speeds) {
        car.speed = s;
    }
}

pub fn can_spawn(cars: &[Vehicle], dir: Dir, route: Route) -> bool {
    cars.iter()
        .filter(|c| c.dir == dir && c.route == route)
        .all(|c| c.distance >= CAR_LEN + SAFE_GAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_guard_blocks_stacking() {
        let mut cars = vec![Vehicle::new(Dir::North, Route::Left)];
        assert!(!can_spawn(&cars, Dir::North, Route::Left));
        assert!(can_spawn(&cars, Dir::North, Route::Right));
        cars[0].update(1.0);
        assert!(can_spawn(&cars, Dir::North, Route::Left));
    }

    #[test]
    fn three_speed_levels_by_gap() {
        assert_eq!(speed_for_gap(SAFE_GAP), 0.0);
        assert_eq!(speed_for_gap(SAFE_GAP + 10.0), SLOW);
        assert_eq!(speed_for_gap(SAFE_GAP + 50.0), MEDIUM);
        assert_eq!(speed_for_gap(500.0), FAST);
    }

    #[test]
    fn follower_never_closes_below_safe_gap() {
        let mut leader = Vehicle::new(Dir::East, Route::Straight);
        leader.update(1.0);
        let mut cars = vec![leader, Vehicle::new(Dir::East, Route::Straight)];
        let dt = 1.0 / 60.0;
        for _ in 0..600 {
            follow(&mut cars);
            cars[0].speed = 0.0;
            cars[1].update(dt);
            let gap = gap_ahead(&cars, 1).unwrap();
            assert!(gap > SAFE_GAP - FAST * dt, "gap fell to {gap}");
        }
        assert_eq!(cars[1].speed, 0.0, "follower should end up stopped");
    }
}
