use riscv::register::time;
use tg_display::{Display, FrameBuffer, PixelFormat, DEFAULT_VIRTIO_MMIO_BASE};

const BOARD_W: usize = 10;
const BOARD_H: usize = 20;
const STEP_CYCLES: u64 = 2_000_000;

const COLORS: [(u8, u8, u8); 8] = [
    (0, 0, 0),
    (90, 220, 255),
    (70, 90, 255),
    (255, 170, 70),
    (255, 230, 90),
    (110, 235, 110),
    (195, 110, 255),
    (255, 110, 120),
];

#[derive(Clone, Copy)]
struct Piece {
    kind: usize,
    rot: usize,
    x: isize,
    y: isize,
}

const SHAPES: [[[(isize, isize); 4]; 4]; 7] = [
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
    ],
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(0, 0), (1, 0), (1, 1), (1, 2)],
    ],
    [
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (2, 1)],
    ],
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
    ],
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
    ],
];

pub(crate) fn run() -> ! {
    let mut display = match Display::new_virtio_gpu(DEFAULT_VIRTIO_MMIO_BASE) {
        Ok(d) => d,
        Err(_) => loop {
            core::hint::spin_loop();
        },
    };

    let mut board = [[0u8; BOARD_W]; BOARD_H];
    let mut seed = time::read64() as u32;
    let mut piece = spawn_piece(&mut seed);
    let mut score = 0usize;
    let mut game_over = false;
    let mut next_step = time::read64().wrapping_add(STEP_CYCLES);

    loop {
        if let Some(c) = try_read_key() {
            match c {
                b'a' | b'A' => {
                    let mut p = piece;
                    p.x -= 1;
                    if !collide(&board, p) {
                        piece = p;
                    }
                }
                b'd' | b'D' => {
                    let mut p = piece;
                    p.x += 1;
                    if !collide(&board, p) {
                        piece = p;
                    }
                }
                b's' | b'S' => {
                    step_down(
                        &mut board,
                        &mut piece,
                        &mut seed,
                        &mut score,
                        &mut game_over,
                    );
                }
                b'w' | b'W' => {
                    let mut p = piece;
                    p.rot = (p.rot + 1) & 3;
                    if !collide(&board, p) {
                        piece = p;
                    }
                }
                b'q' | b'Q' => break,
                _ => {}
            }
        }

        let now = time::read64();
        if !game_over && now >= next_step {
            next_step = now.wrapping_add(STEP_CYCLES);
            step_down(
                &mut board,
                &mut piece,
                &mut seed,
                &mut score,
                &mut game_over,
            );
        }

        if let Ok(mut fb) = display.framebuffer() {
            draw_scene(&mut fb, &board, piece, score, game_over);
            let _ = display.flush();
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

fn step_down(
    board: &mut [[u8; BOARD_W]; BOARD_H],
    piece: &mut Piece,
    seed: &mut u32,
    score: &mut usize,
    game_over: &mut bool,
) {
    if *game_over {
        return;
    }
    let mut p = *piece;
    p.y += 1;
    if !collide(board, p) {
        *piece = p;
        return;
    }

    lock_piece(board, *piece);
    *score += clear_lines(board) * 100;
    let np = spawn_piece(seed);
    if collide(board, np) {
        *game_over = true;
    } else {
        *piece = np;
    }
}

fn spawn_piece(seed: &mut u32) -> Piece {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    let kind = (*seed as usize) % 7;
    Piece {
        kind,
        rot: 0,
        x: 3,
        y: 0,
    }
}

fn collide(board: &[[u8; BOARD_W]; BOARD_H], p: Piece) -> bool {
    for &(dx, dy) in &SHAPES[p.kind][p.rot] {
        let x = p.x + dx;
        let y = p.y + dy;
        if x < 0 || x >= BOARD_W as isize || y < 0 || y >= BOARD_H as isize {
            return true;
        }
        if board[y as usize][x as usize] != 0 {
            return true;
        }
    }
    false
}

fn lock_piece(board: &mut [[u8; BOARD_W]; BOARD_H], p: Piece) {
    let color = (p.kind as u8) + 1;
    for &(dx, dy) in &SHAPES[p.kind][p.rot] {
        let x = (p.x + dx) as usize;
        let y = (p.y + dy) as usize;
        if x < BOARD_W && y < BOARD_H {
            board[y][x] = color;
        }
    }
}

fn clear_lines(board: &mut [[u8; BOARD_W]; BOARD_H]) -> usize {
    let mut cleared = 0usize;
    let mut dst = BOARD_H as isize - 1;
    for src in (0..BOARD_H).rev() {
        let full = board[src].iter().all(|&v| v != 0);
        if full {
            cleared += 1;
            continue;
        }
        if dst as usize != src {
            board[dst as usize] = board[src];
        }
        dst -= 1;
    }
    while dst >= 0 {
        board[dst as usize] = [0u8; BOARD_W];
        dst -= 1;
    }
    cleared
}

fn draw_scene(
    fb: &mut FrameBuffer<'_>,
    board: &[[u8; BOARD_W]; BOARD_H],
    piece: Piece,
    score: usize,
    game_over: bool,
) {
    let w = fb.width;
    let h = fb.height;
    clear(fb, (18, 22, 28));

    let cell = core::cmp::max(14, core::cmp::min(h / (BOARD_H + 4), w / (BOARD_W + 12)));
    let board_w = cell * BOARD_W;
    let board_h = cell * BOARD_H;
    let ox = w / 6;
    let oy = (h.saturating_sub(board_h)) / 2;

    fill_rect(
        fb,
        ox.saturating_sub(6),
        oy.saturating_sub(6),
        board_w + 12,
        board_h + 12,
        (65, 78, 95),
    );
    fill_rect(fb, ox, oy, board_w, board_h, (28, 34, 44));

    for y in 0..BOARD_H {
        for x in 0..BOARD_W {
            let px = ox + x * cell;
            let py = oy + y * cell;
            if ((x + y) & 1) == 0 {
                fill_rect(fb, px, py, cell, cell, (32, 39, 52));
            }
            let v = board[y][x] as usize;
            if v != 0 {
                draw_block(fb, px, py, cell, COLORS[v]);
            }
        }
    }

    for &(dx, dy) in &SHAPES[piece.kind][piece.rot] {
        let x = piece.x + dx;
        let y = piece.y + dy;
        if x >= 0 && y >= 0 && x < BOARD_W as isize && y < BOARD_H as isize {
            let px = ox + (x as usize) * cell;
            let py = oy + (y as usize) * cell;
            draw_block(fb, px, py, cell, COLORS[piece.kind + 1]);
        }
    }

    let panel_x = ox + board_w + cell;
    let panel_w = core::cmp::min(w.saturating_sub(panel_x + cell), cell * 7);
    fill_rect(fb, panel_x, oy, panel_w, board_h, (25, 31, 40));

    let bar_w = core::cmp::min(
        panel_w.saturating_sub(cell),
        (score % 1000) * panel_w / 1000,
    );
    fill_rect(
        fb,
        panel_x + cell / 2,
        oy + cell,
        bar_w,
        cell / 2,
        (110, 220, 160),
    );

    if game_over {
        let gw = board_w / 2;
        let gh = cell * 3;
        let gx = ox + (board_w - gw) / 2;
        let gy = oy + (board_h - gh) / 2;
        fill_rect(fb, gx, gy, gw, gh, (170, 50, 60));
    }
}

fn draw_block(fb: &mut FrameBuffer<'_>, x: usize, y: usize, cell: usize, color: (u8, u8, u8)) {
    fill_rect(
        fb,
        x + 1,
        y + 1,
        cell.saturating_sub(2),
        cell.saturating_sub(2),
        color,
    );
    let hi = brighten(color, 28);
    let lo = darken(color, 35);
    fill_rect(fb, x + 1, y + 1, cell.saturating_sub(2), 2, hi);
    fill_rect(fb, x + 1, y + 1, 2, cell.saturating_sub(2), hi);
    fill_rect(
        fb,
        x + cell.saturating_sub(3),
        y + 1,
        2,
        cell.saturating_sub(2),
        lo,
    );
    fill_rect(
        fb,
        x + 1,
        y + cell.saturating_sub(3),
        cell.saturating_sub(2),
        2,
        lo,
    );
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

fn brighten((r, g, b): (u8, u8, u8), d: u8) -> (u8, u8, u8) {
    (
        r.saturating_add(d),
        g.saturating_add(d),
        b.saturating_add(d),
    )
}

fn darken((r, g, b): (u8, u8, u8), d: u8) -> (u8, u8, u8) {
    (
        r.saturating_sub(d),
        g.saturating_sub(d),
        b.saturating_sub(d),
    )
}

fn try_read_key() -> Option<u8> {
    const UART_BASE: usize = 0x1000_0000;
    const LSR: usize = UART_BASE + 5;
    unsafe {
        let lsr = (LSR as *const u8).read_volatile();
        if (lsr & 0x01) != 0 {
            Some((UART_BASE as *const u8).read_volatile())
        } else {
            None
        }
    }
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
