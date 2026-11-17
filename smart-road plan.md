# Smart Road: Starting Report and Build Plan (FDGS, Mentor mode)

## Context

- The spec is `README(7).md` (01-edu "smart-road"). The goal is a traffic simulation with no traffic lights: autonomous vehicles cross a 4-way, 3-lane-per-direction intersection under a smart algorithm, with no collisions and little congestion.
- The repo is empty apart from the README. The stack is fixed by the spec: **Rust + sdl2**.
- Environment check: `rustc`/`cargo` 1.98 are installed. **SDL2 dev libraries are NOT installed** (`pkg-config sdl2` fails). This is the first blocker.
- Mode: **Mentor** (skill default). You write the core logic. I give hints, review your code and record evidence in `.growth/`.

## 1. Requirements checklist (from spec)

**Functional**

1. Cross intersection. Each road has 3 incoming lanes with fixed routes `r`, `s` and `l`. No lane changes.
2. Vehicles have at least 3 velocities, and the smart intersection controls speed.
3. Vehicles keep a safety distance (> 0) from each other and never collide.
4. Physics: each vehicle tracks `time`, `distance` and `velocity` (v = d/t).
5. Animation: sprites for road and cars, and cars **rotate while turning** (not just a static image).
6. Keys: ↑ (S→N), ↓ (N→S), → (W→E), ← (E→W) spawn a car with a random route. `R` toggles continuous random spawning. `Esc` ends the run and opens the stats window.
7. Spawn spam guard: cars never spawn on top of each other.
8. Stats window: total cars passed, max/min velocity, max/min time to cross (from detection to removal) and close calls.

**Bonus (later):** own assets, more stats, acceleration and braking.

**Out of scope:** emergency vehicles, networking, persistence.

## 2. What you must learn / do before coding

- Install SDL2: `sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev`. Use ttf only if you draw text in the stats window.
- Rust basics you'll use: structs and enums, `Vec` ownership/borrowing, `impl` blocks, modules, `match` and `rand`.
- sdl2 basics: window, canvas, event pump, game loop, textures and `copy_ex` (rotation).
- Math: 2D coordinates, vectors, angles in radians and degrees, arc motion for turns, distance between points.
- Watch the spec's example video and sketch the intersection on paper with coordinates.

## 3. Proposed architecture (simple, single binary)

```
smart-road/
  Cargo.toml          # deps: sdl2 (features image, ttf), rand
  assets/             # car + road sprites
  src/
    main.rs           # init SDL, game loop, input → spawn, draw, Esc → stats
    vehicle.rs        # Vehicle struct: pos, angle, route, dir, speed level, time/distance, path following
    route.rs          # waypoints/turn geometry per (direction, route) – 12 paths
    intersection.rs   # smart algorithm: decides speed for each car each tick
    stats.rs          # collect + render stats window
```

- **Loop:** handle events → spawn → `intersection.update(&mut cars, dt)` (sets speeds) → move cars → remove cars that leave → record stats → render.
- Use a fixed timestep with `dt` from `Instant` so the physics doesn't depend on FPS.

**Recommended algorithm: cell reservation (simplest one that is provably safe)**

- Split the intersection box into a grid (for example 6×6 cells, one per lane width).
- Each of the 12 routes has a precomputed list of cells it passes through.
- A car approaching the box asks to reserve all of its cells. If all are free, it reserves them and drives fast. If not, it slows down or stops at the line and retries.
- Cells are released as the car leaves them. This gives no collisions by construction.
- Upgrade path for less congestion: time-slot reservation (cells × time windows, AIM-style). Do this only if the simple version is too slow.
- Car following on lanes: if the gap to the car ahead is under the safety distance, slow down or stop. Count a close call when the gap drops below the safe distance.

## 4. Roadmap (milestones = vertical, testable slices)

| # | Milestone | Done when | Learning focus |
| --- | ----------- | ----------- | ---------------- |
| M0 | Setup | SDL2 installed, `cargo run` opens an empty window, Esc closes it | cargo, sdl2 init, event loop |
| M1 | Static world | Road and intersection drawn with the coordinate system decided | world coords, textures |
| M2 | Spawn and move straight | Arrow keys spawn cars on correct lanes, cars move and despawn off-screen, spam guard works | structs, Vec mgmt, dt physics |
| M3 | Routes and turning | All 12 paths work, sprite rotates smoothly through turns | geometry, angles, `copy_ex` |
| M4 | Safety distance | Same-lane cars never overlap, 3+ speed levels used | car-following logic |
| M5 | Smart intersection | Cell reservation works, R-mode runs for minutes with zero collisions | algorithm design, conflict detection |
| M6 | Stats | Esc opens a second window with all 7 stats | data aggregation, text rendering |
| M7 | Polish and review | Code review, `cargo clippy` clean, edge cases tested | review, refactor |
| M8 (opt) | Bonus | Accel/braking, extra stats, own assets | physics |

Rule for every milestone: run it, watch R-mode, and check that nothing overlaps before moving on.

## 5. Tests / verification

- Unit tests (`cargo test`) for pure logic: route cell lists, the reservation conflict check, the safety-gap function and stats min/max math.
- Manual: R-mode stress run, key spam in one direction, all 12 routes seen at least once.
- `cargo clippy` and `cargo fmt`.

## 6. Growth tracking (create when M0 starts)

- `.growth/project-brief.md` holds the requirements above.
- `.growth/roadmap.md` holds the milestone table.
- `.growth/progress.md` and `.growth/architecture-decisions.md` record the algorithm choice and why.
- `.growth/competency-baseline.md` starts with everything marked **Not assessed** and fills in from your explanations and code.
- Each task gets an assistance label: Independent / AI-assisted, understood / etc.

## 7. Key risks

- SDL2 setup on Linux (missing libs, linking errors).
- Turn geometry and rotation math take the most time.
- Deadlocks: cars waiting forever on each other's cells. Reserving the whole path atomically avoids this.
- Asset licensing: use free assets or draw simple rectangles first.

## 8. Your first action

1. Install the SDL2 dev libraries (command above).
2. `cargo init` in `smart-road/` and add `sdl2` + `rand` to `Cargo.toml`.
3. Mentor question before M1: **how would you define the coordinates of each lane, and how would you represent a car's direction?** Tell me your idea and I'll review it.
