use tg_display::{Display, FrameBuffer, PixelFormat, DEFAULT_VIRTIO_MMIO_BASE};

const BG: (u8, u8, u8) = (248, 249, 252);

#[derive(Clone, Copy)]
struct Pt {
    x: i32,
    y: i32,
}

struct Tri {
    p0: Pt,
    p1: Pt,
    p2: Pt,
    color: (u8, u8, u8),
}

const CHUNK_SIZE: usize = 3;
const TOTAL_TRIANGLES: usize = 24;

pub(super) fn render_piece_from_app_index(app_index: usize) {
    let draw_count = core::cmp::min(TOTAL_TRIANGLES, (app_index + 1) * CHUNK_SIZE);
    if draw_count == 0 {
        return;
    }

    let mut display = match Display::new_virtio_gpu(DEFAULT_VIRTIO_MMIO_BASE) {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut fb = match display.framebuffer() {
        Ok(fb) => fb,
        Err(_) => return,
    };

    clear(&mut fb, BG);
    let tris = build_triangles(&fb);
    for tri in tris.iter().take(draw_count) {
        fill_triangle(&mut fb, tri);
    }
    let _ = display.flush();
}

fn build_triangles(fb: &FrameBuffer<'_>) -> [Tri; TOTAL_TRIANGLES] {
    let w = fb.width;
    let h = fb.height;
    let u = core::cmp::max(2, core::cmp::min(w / 120, h / 56)) as i32;
    let ow = 40 * u;
    let oh = 40 * u;
    let sw = 40 * u;
    let sh = 40 * u;
    let total_w = ow + 24 * u + sw;
    let total_h = core::cmp::max(oh, sh);
    let ox = ((w as i32 - total_w) / 2).max(0);
    let oy = ((h as i32 - total_h) / 2).max(0);
    let sx = ox + ow + 24 * u;
    let sy = oy;

    let red = (200, 20, 10);
    let orange = (255, 140, 0);
    let yellow = (255, 210, 10);
    let green = (100, 240, 40);
    let cyan = (10, 200, 245);
    let blue = (60, 30, 240);
    let magenta = (240, 80, 220);

    [
        // O: left/top ring
        tri(
            p(0, 0, ox, oy, u),
            p(12, 0, ox, oy, u),
            p(0, 12, ox, oy, u),
            red,
        ),
        tri(
            p(12, 0, ox, oy, u),
            p(24, 0, ox, oy, u),
            p(24, 8, ox, oy, u),
            orange,
        ),
        tri(
            p(24, 0, ox, oy, u),
            p(36, 0, ox, oy, u),
            p(36, 12, ox, oy, u),
            yellow,
        ),
        // O: right side
        tri(
            p(36, 12, ox, oy, u),
            p(40, 20, ox, oy, u),
            p(36, 28, ox, oy, u),
            green,
        ),
        tri(
            p(36, 12, ox, oy, u),
            p(40, 20, ox, oy, u),
            p(32, 20, ox, oy, u),
            cyan,
        ),
        tri(
            p(36, 28, ox, oy, u),
            p(40, 20, ox, oy, u),
            p(32, 20, ox, oy, u),
            blue,
        ),
        // O: bottom ring
        tri(
            p(36, 28, ox, oy, u),
            p(36, 40, ox, oy, u),
            p(24, 40, ox, oy, u),
            magenta,
        ),
        tri(
            p(24, 40, ox, oy, u),
            p(12, 40, ox, oy, u),
            p(16, 32, ox, oy, u),
            red,
        ),
        tri(
            p(12, 40, ox, oy, u),
            p(0, 40, ox, oy, u),
            p(0, 28, ox, oy, u),
            orange,
        ),
        // O: left side
        tri(
            p(0, 28, ox, oy, u),
            p(0, 12, ox, oy, u),
            p(8, 20, ox, oy, u),
            yellow,
        ),
        tri(
            p(0, 12, ox, oy, u),
            p(12, 12, ox, oy, u),
            p(8, 20, ox, oy, u),
            green,
        ),
        tri(
            p(0, 28, ox, oy, u),
            p(12, 28, ox, oy, u),
            p(8, 20, ox, oy, u),
            cyan,
        ),
        // S: top band
        tri(
            p(0, 0, sx, sy, u),
            p(14, 0, sx, sy, u),
            p(8, 8, sx, sy, u),
            blue,
        ),
        tri(
            p(14, 0, sx, sy, u),
            p(28, 0, sx, sy, u),
            p(22, 8, sx, sy, u),
            magenta,
        ),
        tri(
            p(28, 0, sx, sy, u),
            p(40, 0, sx, sy, u),
            p(32, 10, sx, sy, u),
            red,
        ),
        // S: upper curve to middle
        tri(
            p(8, 8, sx, sy, u),
            p(0, 16, sx, sy, u),
            p(10, 20, sx, sy, u),
            orange,
        ),
        tri(
            p(10, 20, sx, sy, u),
            p(22, 20, sx, sy, u),
            p(16, 28, sx, sy, u),
            yellow,
        ),
        tri(
            p(22, 20, sx, sy, u),
            p(34, 20, sx, sy, u),
            p(28, 28, sx, sy, u),
            green,
        ),
        // S: middle to lower curve
        tri(
            p(34, 20, sx, sy, u),
            p(40, 30, sx, sy, u),
            p(30, 30, sx, sy, u),
            cyan,
        ),
        tri(
            p(30, 30, sx, sy, u),
            p(18, 30, sx, sy, u),
            p(24, 36, sx, sy, u),
            blue,
        ),
        tri(
            p(18, 30, sx, sy, u),
            p(6, 30, sx, sy, u),
            p(12, 38, sx, sy, u),
            magenta,
        ),
        // S: bottom band
        tri(
            p(0, 40, sx, sy, u),
            p(12, 40, sx, sy, u),
            p(6, 32, sx, sy, u),
            red,
        ),
        tri(
            p(12, 40, sx, sy, u),
            p(26, 40, sx, sy, u),
            p(20, 32, sx, sy, u),
            orange,
        ),
        tri(
            p(26, 40, sx, sy, u),
            p(40, 40, sx, sy, u),
            p(34, 32, sx, sy, u),
            yellow,
        ),
    ]
}

fn p(x: i32, y: i32, ox: i32, oy: i32, u: i32) -> Pt {
    Pt {
        x: ox + x * u,
        y: oy + y * u,
    }
}

fn tri(p0: Pt, p1: Pt, p2: Pt, color: (u8, u8, u8)) -> Tri {
    Tri { p0, p1, p2, color }
}

fn clear(fb: &mut FrameBuffer<'_>, color: (u8, u8, u8)) {
    let width = fb.width;
    let height = fb.height;
    let stride = fb.stride;
    let bpp = fb.bytes_per_pixel;
    let px = encode(color, fb.format);
    let data = fb.as_bytes_mut();
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

fn fill_triangle(fb: &mut FrameBuffer<'_>, tri: &Tri) {
    let width = fb.width;
    let height = fb.height;
    let stride = fb.stride;
    let bpp = fb.bytes_per_pixel;
    let px = encode(tri.color, fb.format);

    let min_x = tri.p0.x.min(tri.p1.x).min(tri.p2.x).max(0) as usize;
    let min_y = tri.p0.y.min(tri.p1.y).min(tri.p2.y).max(0) as usize;
    let max_x = (tri.p0.x.max(tri.p1.x).max(tri.p2.x) as usize).min(width.saturating_sub(1));
    let max_y = (tri.p0.y.max(tri.p1.y).max(tri.p2.y) as usize).min(height.saturating_sub(1));

    let data = fb.as_bytes_mut();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if inside_tri(x as i32, y as i32, tri.p0, tri.p1, tri.p2) {
                let off = y * stride + x * bpp;
                if off + 4 <= data.len() {
                    data[off..off + 4].copy_from_slice(&px);
                }
            }
        }
    }
}

fn inside_tri(px: i32, py: i32, a: Pt, b: Pt, c: Pt) -> bool {
    let s1 = edge(a, b, px, py);
    let s2 = edge(b, c, px, py);
    let s3 = edge(c, a, px, py);
    (s1 >= 0 && s2 >= 0 && s3 >= 0) || (s1 <= 0 && s2 <= 0 && s3 <= 0)
}

fn edge(a: Pt, b: Pt, px: i32, py: i32) -> i32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
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
