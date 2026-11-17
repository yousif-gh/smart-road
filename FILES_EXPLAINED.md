# Smart Road — Caveman File Guide

Click file name, file open. Click function name, jump to line.

Read order good for brain: main → world → vehicle → route → intersection → stats.

---

## [main.rs](src/main.rs)

[main.rs](src/main.rs) is big chief file: it start SDL window, make car pictures, run game loop every frame, and open stats window when game end.

- [`try_spawn`](src/main.rs#L19) pick random route for new car, ask `can_spawn` if lane free, and push car into list only if yes.
- [`car_sprite`](src/main.rs#L27) draw one top-down car (body, glass, yellow headlights, red taillights) into texture once, so game reuse picture and not redraw every frame.
- [`main`](src/main.rs#L57) make window, make 5 colored car sprites, then loop forever: read keys (arrows spawn car, R toggle random spawn, Esc quit), call `follow` then `intersection.update` then `car.update`, tell stats what happen, throw away cars off screen, draw roads and cars, and at end hide window and call `stats::show`.

---

## [world.rs](src/world.rs)

[world.rs](src/world.rs) hold size numbers of world (800px window, 40px lane, intersection box edges `BOX_MIN`/`BOX_MAX`) and paint static road picture.

- [`draw_roads`](src/world.rs#L36) paint grass background, two gray asphalt roads crossing, white dashed lines between lanes, yellow center line outside box, and white stop line where each incoming road touch box.

---

## [vehicle.rs](src/vehicle.rs)

[vehicle.rs](src/vehicle.rs) define what car is (`Vehicle` struct, `Dir` north/south/east/west, `Route` left/straight/right), how car move, and how car not crash into car in front.

- [`Route::random`](src/vehicle.rs#L33) pick left, straight, or right by random dice.
- [`Dir::random`](src/vehicle.rs#L39) pick one of four directions by random dice.
- [`Vehicle::new`](src/vehicle.rs#L67) make new car with unique id, random paint, full speed `FAST`, and start position from `route::pose` at distance 0.
- [`Vehicle::update`](src/vehicle.rs#L86) move car forward `speed × dt` along its path, add time, and ask `route::pose` for new x, y, angle.
- [`Vehicle::off_screen`](src/vehicle.rs#L92) say true when car gone past window edge, so main loop can delete it.
- [`overlaps`](src/vehicle.rs#L101) check if two rotated car rectangles touch (separating-axis test), and can stretch first car forward by `ahead` px to check safe zone, not only real crash.
- [`speed_for_gap`](src/vehicle.rs#L129) turn gap to car ahead into speed: too close = stop, little close = `SLOW`, medium = `MEDIUM`, far = `FAST`.
- [`gap_ahead`](src/vehicle.rs#L143) find nearest car in front in same lane (same Dir + Route) and return bumper-to-bumper gap, or nothing if lane empty ahead.
- [`follow`](src/vehicle.rs#L153) for every car, get `gap_ahead`, feed to `speed_for_gap`, and set car speed — this is car-following brain.
- [`can_spawn`](src/vehicle.rs#L164) say true only if every car already in that lane moved at least one car length + `SAFE_GAP`, so new car not spawn on top of old car.

Tests in this file:
- [`spawn_guard_blocks_stacking`](src/vehicle.rs#L175) check new car blocked in busy lane, allowed in other lane, and allowed again after old car drive away.
- [`three_speed_levels_by_gap`](src/vehicle.rs#L184) check each gap size give right speed level.
- [`follower_never_closes_below_safe_gap`](src/vehicle.rs#L192) check fast car behind parked car slow down and stop without going under safe gap.

---

## [route.rs](src/route.rs)

[route.rs](src/route.rs) is path math: each (Dir, Route) pair have one fixed path, so car only need one number `distance` and this file turn that number into x, y, angle on screen.

- [`Dir::unit`](src/route.rs#L15) give direction arrow as (x, y) step, like north = (0, -1) because y go down on screen.
- [`Dir::angle`](src/route.rs#L24) give sprite rotation in degrees for direction (north 0, east 90, south 180, west 270).
- [`lane_offset`](src/route.rs#L35) say how far lane center is from road middle line: left lane closest, straight middle, right lane far.
- [`radius`](src/route.rs#L50) give turn circle size: 0 for straight, small circle for right turn, big circle for left turn.
- [`box_len`](src/route.rs#L59) give how long path is inside intersection box: full box width for straight, quarter-circle length for turns.
- [`pose`](src/route.rs#L67) is main path function: take dir, route, distance and return x, y, angle — straight line before box, quarter-circle arc inside box for turns, straight line on new road after.

Tests in this file:
- [`incoming_lanes_match_readme_layout`](src/route.rs#L109) check cars spawn in correct lane pixel positions.
- [`every_path_exits_into_its_own_outgoing_lane`](src/route.rs#L119) check all 12 paths leave box into own lane center, no two paths share exit lane.
- [`path_is_continuous`](src/route.rs#L141) check car never teleport — small step in distance always give small step on screen.
- [`world_box`](src/route.rs#L159) helper say if coordinate is inside intersection box.

---

## [intersection.rs](src/intersection.rs)

[intersection.rs](src/intersection.rs) is smart traffic cop with no lights: it cut box into 6×6 grid cells, give each car reservation for all cells it need, and make cars without reservation stop at line.

- [`lane`](src/intersection.rs#L37) turn (Dir, Route) into number 0..11 so file can index 12 paths in array.
- [`to_line`](src/intersection.rs#L42) give distance from car front bumper to its stop line (goes negative after car pass line).
- [`footprint`](src/intersection.rs#L46) walk path 1px at time and record every grid cell car body touch, plus last distance where car still touch each cell.
- [`Intersection::new`](src/intersection.rs#L72) precompute footprint for all 12 paths once at start.
- [`full_mask`](src/intersection.rs#L82) turn car's whole footprint into bit-mask (one bit per cell) for fast checking.
- [`held_mask`](src/intersection.rs#L89) give bit-mask of cells reserved car still need — cells already left behind get freed, so next car can go sooner.
- [`Intersection::update`](src/intersection.rs#L100) every frame: start timer when car enter sensor zone (`DETECT`), give reservation to waiting cars closest-first if none of their cells busy by other lane, and slow down cars without reservation so they stop before line.

Tests in this file:
- [`footprints_stay_inside_grid_and_conflict_where_paths_cross`](src/intersection.rs#L145) check footprints stay in grid, crossing paths share cells, opposite right turns not share.
- [`heavy_traffic_no_collisions_no_deadlock`](src/intersection.rs#L165) run 2 minutes of max traffic and check no crash, no car sneak past line, cars keep flowing, nobody wait forever.

---

## [stats.rs](src/stats.rs)

[stats.rs](src/stats.rs) count numbers during game (cars passed, fastest/slowest speed, longest/shortest crossing time, close calls) and show them in second window at end.

- [`Stats::new`](src/stats.rs#L24) make empty scoreboard with zero counts and min values set to infinity.
- [`Stats::observe`](src/stats.rs#L36) every frame, update max/min speed of moving cars and record each car pair that get too close (safe zone touch) only once.
- [`Stats::record_exit`](src/stats.rs#L54) when car leave screen, add one to passed count and update max/min time from sensor detect to exit.
- [`Stats::max_time`](src/stats.rs#L64) test-only helper give longest crossing time.
- [`Stats::lines`](src/stats.rs#L68) turn scoreboard into text lines like "Vehicles passed: 12", show "-" when no data yet.
- [`show`](src/stats.rs#L89) open stats window, load font, render text lines once into textures, and draw them until user press Esc, Enter, or close button.

Tests in this file:
- [`records_min_max_and_counts_close_call_once`](src/stats.rs#L160) check close call counted once and min/max speed and time recorded right.
