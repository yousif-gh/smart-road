use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub const SIZE: i32 = 800;
pub const LANE: i32 = 40;
pub const CENTER: i32 = SIZE / 2;
pub const ROAD_HALF: i32 = 3 * LANE;
pub const BOX_MIN: i32 = CENTER - ROAD_HALF;
pub const BOX_MAX: i32 = CENTER + ROAD_HALF;

const GRASS: Color = Color::RGB(70, 120, 60);
const ASPHALT: Color = Color::RGB(55, 55, 60);
const WHITE: Color = Color::RGB(230, 230, 230);
const YELLOW: Color = Color::RGB(230, 200, 40);

const DASH: i32 = 20;
const GAP: i32 = 20;
const LINE_W: u32 = 2;
const STOP_W: u32 = 4;

pub fn draw_roads(canvas: &mut Canvas<Window>) -> Result<(), String> {
    let road_w = (2 * ROAD_HALF) as u32;
    canvas.set_draw_color(GRASS);
    canvas.clear();

    canvas.set_draw_color(ASPHALT);
    canvas.fill_rect(Rect::new(BOX_MIN, 0, road_w, SIZE as u32))?;
    canvas.fill_rect(Rect::new(0, BOX_MIN, SIZE as u32, road_w))?;

    let arms = [(0, BOX_MIN), (BOX_MAX, SIZE)];

    canvas.set_draw_color(WHITE);
    for k in [1, 2, 4, 5] {
        let p = BOX_MIN + k * LANE - LINE_W as i32 / 2;
        for (a, b) in arms {
            let mut t = a;
            while t < b {
                let len = DASH.min(b - t) as u32;
                canvas.fill_rect(Rect::new(p, t, LINE_W, len))?;
                canvas.fill_rect(Rect::new(t, p, len, LINE_W))?;
                t += DASH + GAP;
            }
        }
    }

    canvas.set_draw_color(YELLOW);
    let c = CENTER - LINE_W as i32;
    for (a, b) in arms {
        let len = (b - a) as u32;
        canvas.fill_rect(Rect::new(c, a, LINE_W * 2, len))?;
        canvas.fill_rect(Rect::new(a, c, len, LINE_W * 2))?;
    }

    canvas.set_draw_color(WHITE);
    let half = ROAD_HALF as u32;
    let s = STOP_W as i32;
    canvas.fill_rect(Rect::new(BOX_MIN, BOX_MIN - s, half, STOP_W))?;
    canvas.fill_rect(Rect::new(CENTER, BOX_MAX, half, STOP_W))?;
    canvas.fill_rect(Rect::new(BOX_MAX, BOX_MIN, STOP_W, half))?;
    canvas.fill_rect(Rect::new(BOX_MIN - s, CENTER, STOP_W, half))?;
    Ok(())
}
