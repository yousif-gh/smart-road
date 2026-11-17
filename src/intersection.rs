use crate::route::{TO_BOX, box_len, pose};
use crate::vehicle::{CAR_LEN, Dir, Route, SAFE_GAP, Vehicle, speed_for_gap};
use crate::world::{BOX_MIN, LANE};

const GRID: usize = 6;
const CELL: f32 = LANE as f32;
const FOOT_R: f32 = 20.0;
pub const DETECT: f32 = 150.0;
const STOP_MARGIN: f32 = 4.0;

type Footprint = Vec<(usize, f32)>;

pub struct Intersection {
    paths: Vec<Footprint>,
}

fn lane(dir: Dir, route: Route) -> usize {
    dir as usize * 3 + route as usize
}

pub fn to_line(car: &Vehicle) -> f32 {
    TO_BOX - car.distance - CAR_LEN / 2.0
}

fn footprint(dir: Dir, route: Route) -> Footprint {
    let mut last = [None::<f32>; GRID * GRID];
    let start = TO_BOX - FOOT_R - 1.0;
    let end = TO_BOX + box_len(route) + FOOT_R + 1.0;
    let mut s = start;
    while s <= end {
        let (x, y, _) = pose(dir, route, s);
        for (i, slot) in last.iter_mut().enumerate() {
            let x0 = BOX_MIN as f32 + (i % GRID) as f32 * CELL;
            let y0 = BOX_MIN as f32 + (i / GRID) as f32 * CELL;
            let dx = x - x.clamp(x0, x0 + CELL);
            let dy = y - y.clamp(y0, y0 + CELL);
            if dx * dx + dy * dy < FOOT_R * FOOT_R {
                *slot = Some(s);
            }
        }
        s += 1.0;
    }
    last.iter()
        .enumerate()
        .filter_map(|(i, l)| l.map(|l| (i, l)))
        .collect()
}

impl Intersection {
    pub fn new() -> Intersection {
        let mut paths = vec![Vec::new(); 12];
        for dir in [Dir::North, Dir::South, Dir::East, Dir::West] {
            for route in [Route::Left, Route::Straight, Route::Right] {
                paths[lane(dir, route)] = footprint(dir, route);
            }
        }
        Intersection { paths }
    }

    fn full_mask(&self, car: &Vehicle) -> u64 {
        self.paths[lane(car.dir, car.route)]
            .iter()
            .fold(0, |m, &(c, _)| m | 1 << c)
    }

    fn held_mask(&self, car: &Vehicle) -> u64 {
        if !car.reserved {
            return 0;
        }
        self.paths[lane(car.dir, car.route)]
            .iter()
            .filter(|&&(_, last)| last >= car.distance)
            .fold(0, |m, &(c, _)| m | 1 << c)
    }

    pub fn update(&self, cars: &mut [Vehicle]) {
        for car in cars.iter_mut() {
            if car.detected_at.is_none() && to_line(car) <= DETECT {
                car.detected_at = Some(car.time);
            }
        }

        let mut waiting: Vec<usize> = (0..cars.len())
            .filter(|&i| !cars[i].reserved && to_line(&cars[i]) <= DETECT)
            .collect();
        waiting.sort_by(|&a, &b| cars[b].distance.total_cmp(&cars[a].distance));
        for i in waiting {
            let me = &cars[i];
            let busy = cars
                .iter()
                .filter(|c| (c.dir, c.route) != (me.dir, me.route))
                .fold(0, |m, c| m | self.held_mask(c));
            if self.full_mask(me) & busy == 0 {
                cars[i].reserved = true;
            }
        }

        for car in cars.iter_mut().filter(|c| !c.reserved) {
            car.speed = car
                .speed
                .min(speed_for_gap(to_line(car) - STOP_MARGIN + SAFE_GAP));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;
    use crate::vehicle::{can_spawn, follow, overlaps};

    const DIRS: [Dir; 4] = [Dir::North, Dir::South, Dir::East, Dir::West];
    const ROUTES: [Route; 3] = [Route::Left, Route::Straight, Route::Right];

    #[test]
    fn footprints_stay_inside_grid_and_conflict_where_paths_cross() {
        let ix = Intersection::new();
        for p in &ix.paths {
            assert!(!p.is_empty() && p.iter().all(|&(c, _)| c < GRID * GRID));
        }
        let m = |d, r| {
            ix.paths[lane(d, r)]
                .iter()
                .fold(0u64, |m, &(c, _)| m | 1 << c)
        };
        assert_ne!(
            m(Dir::North, Route::Straight) & m(Dir::East, Route::Straight),
            0
        );
        assert_eq!(m(Dir::North, Route::Right) & m(Dir::South, Route::Right), 0);
    }

    #[test]
    fn heavy_traffic_no_collisions_no_deadlock() {
        let ix = Intersection::new();
        let mut cars: Vec<Vehicle> = Vec::new();
        let mut passed = 0;
        let mut stats = Stats::new();
        let mut ok_at_line = true;
        let dt = 1.0 / 60.0;
        for tick in 0..60 * 120 {
            if tick % 10 == 0 {
                for d in DIRS {
                    for r in ROUTES {
                        if can_spawn(&cars, d, r) {
                            cars.push(Vehicle::new(d, r));
                        }
                    }
                }
            }
            follow(&mut cars);
            ix.update(&mut cars);
            for c in &mut cars {
                c.update(dt);
            }
            for i in 0..cars.len() {
                for j in i + 1..cars.len() {
                    assert!(
                        !overlaps(&cars[i], 0.0, &cars[j]),
                        "collision at tick {tick}"
                    );
                }
            }
            stats.observe(&cars);
            ok_at_line &= cars.iter().all(|c| c.reserved || to_line(c) >= 0.0);
            let before = cars.len();
            cars.retain(|c| {
                let gone = c.off_screen();
                if gone {
                    stats.record_exit(c);
                }
                !gone
            });
            passed += before - cars.len();
        }
        assert!(ok_at_line, "an unreserved car crossed its stop line");
        assert!(
            passed > 200,
            "only {passed} cars passed: deadlock or starvation"
        );
        println!("{passed} passed; {:?}", stats.lines());
        assert!(stats.max_time() < 10.0, "starvation: {:?}", stats.lines());
    }
}
