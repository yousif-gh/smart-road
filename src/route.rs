use crate::vehicle::{CAR_LEN, Dir, Route};
use crate::world::{CENTER, LANE, ROAD_HALF, SIZE};
use std::f32::consts::FRAC_PI_2;

const H: f32 = ROAD_HALF as f32;

impl Dir {
    pub fn unit(self) -> (f32, f32) {
        match self {
            Dir::North => (0.0, -1.0),
            Dir::South => (0.0, 1.0),
            Dir::East => (1.0, 0.0),
            Dir::West => (-1.0, 0.0),
        }
    }
    fn angle(self) -> f32 {
        match self {
            Dir::North => 0.0,
            Dir::East => 90.0,
            Dir::South => 180.0,
            Dir::West => 270.0,
        }
    }
}

pub fn lane_offset(route: Route) -> f32 {
    let i = match route {
        Route::Left => 0.0,
        Route::Straight => 1.0,
        Route::Right => 2.0,
    };
    LANE as f32 * i + LANE as f32 / 2.0
}

const U_START: f32 = -(SIZE as f32 / 2.0) - CAR_LEN / 2.0;
pub const TO_BOX: f32 = -H - U_START;

fn radius(route: Route) -> f32 {
    match route {
        Route::Straight => 0.0,
        Route::Right => H - lane_offset(route),
        Route::Left => H + lane_offset(route),
    }
}

pub fn box_len(route: Route) -> f32 {
    match route {
        Route::Straight => 2.0 * H,
        _ => radius(route) * FRAC_PI_2,
    }
}

pub fn pose(dir: Dir, route: Route, s: f32) -> (f32, f32, f32) {
    let off = lane_offset(route);
    let rho = radius(route);
    let arc = box_len(route);

    let (u, v, turn) = match route {
        _ if route == Route::Straight || s <= TO_BOX => (U_START + s, off, 0.0),
        _ if s <= TO_BOX + arc => {
            let th = (s - TO_BOX) / rho;
            let (sin, cos) = th.sin_cos();
            if route == Route::Right {
                (-H + rho * sin, H - rho * cos, th.to_degrees())
            } else {
                (-H + rho * sin, -H + rho * cos, -th.to_degrees())
            }
        }
        _ => {
            let rest = s - TO_BOX - arc;
            if route == Route::Right {
                (-H + rho, H + rest, 90.0)
            } else {
                (-H + rho, -H - rest, -90.0)
            }
        }
    };

    let (fx, fy) = dir.unit();
    let (rx, ry) = (-fy, fx);
    let c = CENTER as f32;
    (c + fx * u + rx * v, c + fy * u + ry * v, dir.angle() + turn)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIRS: [Dir; 4] = [Dir::North, Dir::South, Dir::East, Dir::West];
    const ROUTES: [Route; 3] = [Route::Left, Route::Straight, Route::Right];

    #[test]
    fn incoming_lanes_match_readme_layout() {
        assert_eq!(pose(Dir::North, Route::Left, 0.0).0, 420.0);
        assert_eq!(pose(Dir::North, Route::Right, 0.0).0, 500.0);
        assert_eq!(pose(Dir::South, Route::Right, 0.0).0, 300.0);
        assert_eq!(pose(Dir::West, Route::Left, 0.0).1, 380.0);
        assert_eq!(pose(Dir::East, Route::Left, 0.0).1, 420.0);
    }

    #[test]
    fn every_path_exits_into_its_own_outgoing_lane() {
        let mut exits = Vec::new();
        for d in DIRS {
            for r in ROUTES {
                let (x, y, a) = pose(d, r, TO_BOX + box_len(r) + 50.0);
                let a = a.rem_euclid(360.0).round() as i32;
                let lat = if a % 180 == 0 { x } else { y };
                exits.push((a, lat.round() as i32));
                let outside = !(world_box(x) && world_box(y));
                assert!(outside, "{d:?} {r:?} still in box at {x},{y}");
                assert_eq!((lat as i32 - CENTER).abs() % LANE, LANE / 2, "{d:?} {r:?}");
            }
        }
        let n = exits.len();
        exits.sort();
        exits.dedup();
        assert_eq!(exits.len(), n, "two paths share an outgoing lane");
    }

    #[test]
    fn path_is_continuous() {
        for d in DIRS {
            for r in ROUTES {
                let mut prev = pose(d, r, 0.0);
                for i in 1..2000 {
                    let p = pose(d, r, i as f32 * 0.5);
                    let jump = ((p.0 - prev.0).powi(2) + (p.1 - prev.1).powi(2)).sqrt();
                    assert!(
                        jump < 0.51,
                        "{d:?} {r:?} jumps {jump} at s={}",
                        i as f32 * 0.5
                    );
                    prev = p;
                }
            }
        }
    }

    fn world_box(p: f32) -> bool {
        p > CENTER as f32 - H && p < CENTER as f32 + H
    }
}
