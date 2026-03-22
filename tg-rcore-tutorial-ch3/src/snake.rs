use riscv::register::time;
use tg_display::{Display, FrameBuffer, PixelFormat, DEFAULT_VIRTIO_MMIO_BASE};
use tg_sbi::console_putchar;

const GRID_W: usize = 32;
const GRID_H: usize = 20;
const MAX_LEN: usize = GRID_W * GRID_H;
const DEBUG_INPUT: bool = true;
const STEP_CYCLES: u64 = 1_200_000;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub(super) fn run() -> ! {
    if DEBUG_INPUT {
        dbg_str("[snake] enter run()\n");
        dbg_str("[snake] input source: UART MMIO polling\n");
        dbg_str("[snake] tip: serial terminal input is expected, GTK window keys are ignored in this mode\n");
    }

    tg_sbi::set_timer(u64::MAX);

    let mut display = match Display::new_virtio_gpu(DEFAULT_VIRTIO_MMIO_BASE) {
        Ok(d) => d,
        Err(_) => loop {
            core::hint::spin_loop();
        },
    };

    let mut body = [Pos { x: 0, y: 0 }; MAX_LEN];
    let mut len = 4usize;
    body[0] = Pos {
        x: GRID_W / 2,
        y: GRID_H / 2,
    };
    body[1] = Pos {
        x: GRID_W / 2 - 1,
        y: GRID_H / 2,
    };
    body[2] = Pos {
        x: GRID_W / 2 - 2,
        y: GRID_H / 2,
    };
    body[3] = Pos {
        x: GRID_W / 2 - 3,
        y: GRID_H / 2,
    };

    let mut dir = Dir::Right;
    let mut food = spawn_food(&body, len, 13);
    let mut game_over = false;
    let mut tick = 0usize;
    let mut next_step = time::read64().wrapping_add(STEP_CYCLES);

    if let Ok(mut fb) = display.framebuffer() {
        draw_scene(&mut fb, &body, len, food, game_over);
        let _ = display.flush();
        if DEBUG_INPUT {
            dbg_str("[snake] initial frame flushed\n");
        }
    } else if DEBUG_INPUT {
        dbg_str("[snake] framebuffer acquire failed\n");
    }

    loop {
        tick = tick.wrapping_add(1);

        if DEBUG_INPUT && tick.is_multiple_of(2_000_000) {
            dbg_str("[snake] heartbeat tick=");
            dbg_usize(tick);
            dbg_str("\n");
        }

        if let Some(c) = try_read_key() {
            if DEBUG_INPUT {
                dbg_str("[snake] key=0x");
                dbg_hex2(c);
                dbg_str(" '");
                if c.is_ascii_graphic() || c == b' ' {
                    console_putchar(c);
                } else {
                    console_putchar(b'.');
                }
                dbg_str("'\n");
            }
            if c == b'q' {
                break;
            }
            if let Some(nd) = map_dir(c) {
                if !is_opposite(dir, nd) {
                    dir = nd;
                }
            }
        }

        let now = time::read64();
        if !game_over && now >= next_step {
            next_step = now.wrapping_add(STEP_CYCLES);
            let next = step(body[0], dir);
            if hit_wall(next) || hit_body(next, &body, len) {
                game_over = true;
                if DEBUG_INPUT {
                    dbg_str("[snake] game over\n");
                }
            } else {
                let ate = next == food;
                for i in (1..len).rev() {
                    body[i] = body[i - 1];
                }
                body[0] = next;
                if DEBUG_INPUT {
                    dbg_str("[snake] move head=(");
                    dbg_usize(body[0].x);
                    dbg_str(",");
                    dbg_usize(body[0].y);
                    dbg_str(")\n");
                }
                if ate && len + 1 < MAX_LEN {
                    body[len] = body[len - 1];
                    len += 1;
                    food = spawn_food(&body, len, tick);
                    if DEBUG_INPUT {
                        dbg_str("[snake] eat food, len=");
                        dbg_usize(len);
                        dbg_str("\n");
                    }
                }
            }
        }

        if let Ok(mut fb) = display.framebuffer() {
            draw_scene(&mut fb, &body, len, food, game_over);
            let _ = display.flush();
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

pub(super) fn activate_display() {
    if let Ok(mut display) = Display::new_virtio_gpu(DEFAULT_VIRTIO_MMIO_BASE) {
        if let Ok(mut fb) = display.framebuffer() {
            clear(&mut fb, (20, 24, 30));
            let w = fb.width;
            let h = fb.height;
            let panel_w = w / 2;
            let panel_h = h / 6;
            let x = (w.saturating_sub(panel_w)) / 2;
            let y = (h.saturating_sub(panel_h)) / 2;
            fill_rect(&mut fb, x, y, panel_w, panel_h, (50, 65, 90));
            fill_rect(
                &mut fb,
                x + 8,
                y + 8,
                panel_w.saturating_sub(16),
                panel_h.saturating_sub(16),
                (30, 36, 44),
            );
            let _ = display.flush();
        }
    }
}

fn draw_scene(
    fb: &mut FrameBuffer<'_>,
    body: &[Pos; MAX_LEN],
    len: usize,
    food: Pos,
    game_over: bool,
) {
    let w = fb.width;
    let h = fb.height;
    let cell = core::cmp::max(8, core::cmp::min(w / (GRID_W + 4), h / (GRID_H + 6)));
    let board_w = cell * GRID_W;
    let board_h = cell * GRID_H;
    let ox = (w.saturating_sub(board_w)) / 2;
    let oy = (h.saturating_sub(board_h)) / 2;

    clear(fb, (20, 24, 30));
    fill_rect(
        fb,
        ox.saturating_sub(4),
        oy.saturating_sub(4),
        board_w + 8,
        board_h + 8,
        (75, 80, 92),
    );
    fill_rect(fb, ox, oy, board_w, board_h, (30, 36, 44));

    for y in 0..GRID_H {
        for x in 0..GRID_W {
            if ((x + y) & 1) == 0 {
                fill_rect(fb, ox + x * cell, oy + y * cell, cell, cell, (34, 40, 48));
            }
        }
    }

    fill_rect(
        fb,
        ox + food.x * cell + 2,
        oy + food.y * cell + 2,
        cell.saturating_sub(4),
        cell.saturating_sub(4),
        (255, 90, 90),
    );

    for i in (1..len).rev() {
        let p = body[i];
        fill_rect(
            fb,
            ox + p.x * cell + 2,
            oy + p.y * cell + 2,
            cell.saturating_sub(4),
            cell.saturating_sub(4),
            (70, 220, 120),
        );
    }
    let head = body[0];
    fill_rect(
        fb,
        ox + head.x * cell + 1,
        oy + head.y * cell + 1,
        cell.saturating_sub(2),
        cell.saturating_sub(2),
        (120, 255, 170),
    );

    if game_over {
        let gw = board_w / 2;
        let gh = cell * 2;
        let gx = ox + (board_w - gw) / 2;
        let gy = oy + (board_h - gh) / 2;
        fill_rect(fb, gx, gy, gw, gh, (180, 35, 35));
    }
}

fn clear(fb: &mut FrameBuffer<'_>, color: (u8, u8, u8)) {
    let (w, h, stride, bpp, fmt) = (
        fb.width,
        fb.height,
        fb.stride,
        fb.bytes_per_pixel,
        fb.format,
    );
    let px = encode(color, fmt);
    let data = fb.as_bytes_mut();
    for y in 0..h {
        let row = y * stride;
        for x in 0..w {
            let off = row + x * bpp;
            if off + 4 <= data.len() {
                data[off..off + 4].copy_from_slice(&px);
            }
        }
    }
}

fn fill_rect(
    fb: &mut FrameBuffer<'_>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: (u8, u8, u8),
) {
    if w == 0 || h == 0 {
        return;
    }
    let max_x = core::cmp::min(fb.width, x.saturating_add(w));
    let max_y = core::cmp::min(fb.height, y.saturating_add(h));
    if x >= max_x || y >= max_y {
        return;
    }
    let (stride, bpp, fmt) = (fb.stride, fb.bytes_per_pixel, fb.format);
    let px = encode(color, fmt);
    let data = fb.as_bytes_mut();
    for yy in y..max_y {
        let row = yy * stride;
        for xx in x..max_x {
            let off = row + xx * bpp;
            if off + 4 <= data.len() {
                data[off..off + 4].copy_from_slice(&px);
            }
        }
    }
}

fn map_dir(c: u8) -> Option<Dir> {
    match c {
        b'w' | b'W' => Some(Dir::Up),
        b's' | b'S' => Some(Dir::Down),
        b'a' | b'A' => Some(Dir::Left),
        b'd' | b'D' => Some(Dir::Right),
        _ => None,
    }
}

fn is_opposite(a: Dir, b: Dir) -> bool {
    matches!(
        (a, b),
        (Dir::Up, Dir::Down)
            | (Dir::Down, Dir::Up)
            | (Dir::Left, Dir::Right)
            | (Dir::Right, Dir::Left)
    )
}

fn step(p: Pos, d: Dir) -> Pos {
    match d {
        Dir::Up => Pos {
            x: p.x,
            y: p.y.saturating_sub(1),
        },
        Dir::Down => Pos { x: p.x, y: p.y + 1 },
        Dir::Left => Pos {
            x: p.x.saturating_sub(1),
            y: p.y,
        },
        Dir::Right => Pos { x: p.x + 1, y: p.y },
    }
}

fn hit_wall(p: Pos) -> bool {
    p.x >= GRID_W || p.y >= GRID_H
}

fn hit_body(p: Pos, body: &[Pos; MAX_LEN], len: usize) -> bool {
    for seg in body.iter().take(len) {
        if *seg == p {
            return true;
        }
    }
    false
}

fn spawn_food(body: &[Pos; MAX_LEN], len: usize, seed: usize) -> Pos {
    let mut x = (seed.wrapping_mul(1103515245).wrapping_add(12345) >> 8) % GRID_W;
    let mut y = (seed.wrapping_mul(69069).wrapping_add(1) >> 8) % GRID_H;
    for _ in 0..(GRID_W * GRID_H) {
        let p = Pos { x, y };
        if !hit_body(p, body, len) {
            return p;
        }
        x = (x + 7) % GRID_W;
        y = (y + 5) % GRID_H;
    }
    Pos { x: 0, y: 0 }
}

fn try_read_key() -> Option<u8> {
    const UART_BASE: usize = 0x1000_0000;
    const LSR: usize = UART_BASE + 5;
    // SAFETY: Accessing QEMU virt UART MMIO registers.
    unsafe {
        let lsr = (LSR as *const u8).read_volatile();
        if (lsr & 0x01) != 0 {
            Some((UART_BASE as *const u8).read_volatile())
        } else {
            None
        }
    }
}

fn dbg_str(s: &str) {
    for &b in s.as_bytes() {
        console_putchar(b);
    }
}

fn dbg_usize(mut n: usize) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    if n == 0 {
        console_putchar(b'0');
        return;
    }
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    for &b in &buf[i..] {
        console_putchar(b);
    }
}

fn dbg_hex2(v: u8) {
    fn nibble(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            _ => b'a' + (n - 10),
        }
    }
    console_putchar(nibble((v >> 4) & 0x0f));
    console_putchar(nibble(v & 0x0f));
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
