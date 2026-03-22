use tg_display::{FrameBuffer, PixelFormat};

const BG: (u8, u8, u8) = (248, 249, 252);
const O_C1: (u8, u8, u8) = (200, 20, 10);
const O_C2: (u8, u8, u8) = (255, 140, 0);
const O_C3: (u8, u8, u8) = (255, 210, 10);
const O_C4: (u8, u8, u8) = (100, 240, 40);
const O_C5: (u8, u8, u8) = (10, 200, 245);
const O_C6: (u8, u8, u8) = (60, 30, 240);
const O_C7: (u8, u8, u8) = (240, 80, 220);
const S_C1: (u8, u8, u8) = (240, 80, 220);
const S_C2: (u8, u8, u8) = (10, 200, 245);
const S_C3: (u8, u8, u8) = (60, 30, 240);
const S_C4: (u8, u8, u8) = (255, 210, 10);
const S_C5: (u8, u8, u8) = (200, 20, 10);
const S_C6: (u8, u8, u8) = (255, 140, 0);
const S_C7: (u8, u8, u8) = (100, 240, 40);

#[derive(Clone, Copy)]
struct Pt {
    x: i32,
    y: i32,
}

enum Shape {
    Tri([Pt; 3]),
    Quad([Pt; 4]),
}

pub(super) fn render_tangram_os(fb: &mut FrameBuffer<'_>) {
    let width = fb.width;
    let height = fb.height;
    let stride = fb.stride;
    let bpp = fb.bytes_per_pixel;
    let fmt = fb.format;
    if bpp < 4 || width == 0 || height == 0 {
        return;
    }

    let len = match stride.checked_mul(height) {
        Some(v) => v,
        None => return,
    };
    let data = fb.as_bytes_mut();
    if data.len() < len {
        return;
    }

    clear(data, width, height, stride, bpp, encode(BG, fmt));

    let scale = core::cmp::max(1, core::cmp::min(width / 240, height / 100));
    let unit = scale as i32;
    let total_w = 220 * unit;
    let total_h = 84 * unit;
    let ox = ((width as i32 - total_w) / 2).max(0);
    let oy = ((height as i32 - total_h) / 2).max(0);

    let o_shapes: [(Shape, (u8, u8, u8)); 7] = [
        (
            Shape::Tri([
                p(0, 0, ox, oy, unit),
                p(42, 0, ox, oy, unit),
                p(0, 42, ox, oy, unit),
            ]),
            O_C1,
        ),
        (
            Shape::Tri([
                p(0, 0, ox, oy, unit),
                p(42, 42, ox, oy, unit),
                p(0, 84, ox, oy, unit),
            ]),
            O_C2,
        ),
        (
            Shape::Tri([
                p(42, 42, ox, oy, unit),
                p(84, 0, ox, oy, unit),
                p(84, 84, ox, oy, unit),
            ]),
            O_C3,
        ),
        (
            Shape::Tri([
                p(18, 84, ox, oy, unit),
                p(60, 84, ox, oy, unit),
                p(39, 63, ox, oy, unit),
            ]),
            O_C4,
        ),
        (
            Shape::Quad([
                p(24, 24, ox, oy, unit),
                p(39, 9, ox, oy, unit),
                p(54, 24, ox, oy, unit),
                p(39, 39, ox, oy, unit),
            ]),
            O_C5,
        ),
        (
            Shape::Tri([
                p(24, 60, ox, oy, unit),
                p(54, 60, ox, oy, unit),
                p(39, 45, ox, oy, unit),
            ]),
            O_C6,
        ),
        (
            Shape::Tri([
                p(54, 24, ox, oy, unit),
                p(84, 24, ox, oy, unit),
                p(84, 54, ox, oy, unit),
            ]),
            O_C7,
        ),
    ];

    let sx = ox + 120 * unit;
    let s_shapes: [(Shape, (u8, u8, u8)); 7] = [
        (
            Shape::Tri([
                p(0, 0, sx, oy, unit),
                p(84, 0, sx, oy, unit),
                p(42, 42, sx, oy, unit),
            ]),
            S_C1,
        ),
        (
            Shape::Tri([
                p(0, 0, sx, oy, unit),
                p(42, 42, sx, oy, unit),
                p(0, 84, sx, oy, unit),
            ]),
            S_C2,
        ),
        (
            Shape::Tri([
                p(84, 0, sx, oy, unit),
                p(84, 42, sx, oy, unit),
                p(42, 42, sx, oy, unit),
            ]),
            S_C3,
        ),
        (
            Shape::Tri([
                p(0, 84, sx, oy, unit),
                p(84, 84, sx, oy, unit),
                p(42, 42, sx, oy, unit),
            ]),
            S_C4,
        ),
        (
            Shape::Quad([
                p(24, 24, sx, oy, unit),
                p(42, 6, sx, oy, unit),
                p(60, 24, sx, oy, unit),
                p(42, 42, sx, oy, unit),
            ]),
            S_C5,
        ),
        (
            Shape::Tri([
                p(24, 60, sx, oy, unit),
                p(42, 42, sx, oy, unit),
                p(60, 60, sx, oy, unit),
            ]),
            S_C6,
        ),
        (
            Shape::Tri([
                p(42, 42, sx, oy, unit),
                p(84, 42, sx, oy, unit),
                p(84, 84, sx, oy, unit),
            ]),
            S_C7,
        ),
    ];

    for (shape, c) in o_shapes {
        fill_shape(data, width, height, stride, bpp, fmt, &shape, c);
    }
    for (shape, c) in s_shapes {
        fill_shape(data, width, height, stride, bpp, fmt, &shape, c);
    }
}

fn p(x: i32, y: i32, ox: i32, oy: i32, u: i32) -> Pt {
    Pt {
        x: ox + x * u,
        y: oy + y * u,
    }
}

fn clear(data: &mut [u8], width: usize, height: usize, stride: usize, bpp: usize, px: [u8; 4]) {
    for y in 0..height {
        let row = y * stride;
        for x in 0..width {
            let off = row + x * bpp;
            if off + 4 <= data.len() {
                data[off..off + 4].copy_from_slice(&px);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_shape(
    data: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    bpp: usize,
    fmt: PixelFormat,
    shape: &Shape,
    color: (u8, u8, u8),
) {
    let pts4: [Pt; 4];
    let n = match shape {
        Shape::Tri(v) => {
            pts4 = [v[0], v[1], v[2], Pt { x: 0, y: 0 }];
            3
        }
        Shape::Quad(v) => {
            pts4 = [v[0], v[1], v[2], v[3]];
            4
        }
    };

    let (min_x, max_x, min_y, max_y) = bounds(&pts4, n, width, height);
    let px = encode(color, fmt);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if inside(x as i32, y as i32, &pts4, n) {
                let off = y * stride + x * bpp;
                if off + 4 <= data.len() {
                    data[off..off + 4].copy_from_slice(&px);
                }
            }
        }
    }
}

fn bounds(v: &[Pt; 4], n: usize, w: usize, h: usize) -> (usize, usize, usize, usize) {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for p in v.iter().take(n) {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    let min_x = if min_x <= 0 { 0 } else { min_x as usize };
    let min_y = if min_y <= 0 { 0 } else { min_y as usize };
    let max_x = ((max_x as usize).saturating_add(1)).min(w.saturating_sub(1));
    let max_y = ((max_y as usize).saturating_add(1)).min(h.saturating_sub(1));
    (min_x, max_x, min_y, max_y)
}

fn inside(px: i32, py: i32, pts: &[Pt; 4], n: usize) -> bool {
    let mut sign = 0i32;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let cross = (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x);
        let s = if cross > 0 {
            1
        } else if cross < 0 {
            -1
        } else {
            0
        };
        if s == 0 {
            continue;
        }
        if sign == 0 {
            sign = s;
        } else if sign != s {
            return false;
        }
    }
    true
}

fn encode((r, g, b): (u8, u8, u8), fmt: PixelFormat) -> [u8; 4] {
    match fmt {
        PixelFormat::Xrgb8888 => [b, g, r, 0x00],
        PixelFormat::Argb8888 => [b, g, r, 0xFF],
        PixelFormat::Bgrx8888 => [r, g, b, 0x00],
        PixelFormat::Bgra8888 => [r, g, b, 0xFF],
        PixelFormat::Unknown => [b, g, r, 0x00],
    }
}
