#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::sleep;
use user_lib::{read, STDIN};

const W: usize = 10;
const H: usize = 20;

#[derive(Clone, Copy)]
struct Piece {
    kind: usize,
    rot: usize,
    x: i32,
    y: i32,
}

const SHAPES: [[[(i32, i32); 4]; 4]; 7] = [
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

#[unsafe(no_mangle)]
extern "C" fn main() -> i32 {
    let mut board = [[0u8; W]; H];
    let mut seed: u32 = 1234567;
    let mut score: usize = 0;
    let mut current = spawn(&mut seed);
    let mut frame: usize = 0;

    loop {
        frame += 1;
        if let Some(c) = poll_key() {
            match c {
                b'a' | b'A' => {
                    let _ = try_move(&mut current, -1, 0, &board);
                }
                b'd' | b'D' => {
                    let _ = try_move(&mut current, 1, 0, &board);
                }
                b's' | b'S' => {
                    if !try_move(&mut current, 0, 1, &board) {
                        lock_piece(&current, &mut board);
                        score += clear_lines(&mut board) * 100;
                        current = spawn(&mut seed);
                        if collides(&current, &board) {
                            break;
                        }
                    }
                }
                b'w' | b'W' => try_rotate(&mut current, &board),
                b'q' | b'Q' => break,
                _ => {}
            }
        }

        if frame.is_multiple_of(8) {
            if !try_move(&mut current, 0, 1, &board) {
                lock_piece(&current, &mut board);
                score += clear_lines(&mut board) * 100;
                current = spawn(&mut seed);
                if collides(&current, &board) {
                    break;
                }
            }
        }

        render(&board, &current, score);
        sleep(35);
    }

    println!("\nGame Over! score={score}");
    0
}

fn spawn(seed: &mut u32) -> Piece {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    Piece {
        kind: ((*seed >> 16) % 7) as usize,
        rot: 0,
        x: 3,
        y: 0,
    }
}

fn collides(p: &Piece, board: &[[u8; W]; H]) -> bool {
    for &(dx, dy) in &SHAPES[p.kind][p.rot] {
        let x = p.x + dx;
        let y = p.y + dy;
        if x < 0 || x >= W as i32 || y >= H as i32 {
            return true;
        }
        if y >= 0 && board[y as usize][x as usize] != 0 {
            return true;
        }
    }
    false
}

fn try_move(p: &mut Piece, dx: i32, dy: i32, board: &[[u8; W]; H]) -> bool {
    let old = *p;
    p.x += dx;
    p.y += dy;
    if collides(p, board) {
        *p = old;
        false
    } else {
        true
    }
}

fn try_rotate(p: &mut Piece, board: &[[u8; W]; H]) {
    let old = *p;
    p.rot = (p.rot + 1) & 3;
    if collides(p, board) {
        p.x -= 1;
        if collides(p, board) {
            p.x += 2;
            if collides(p, board) {
                *p = old;
            }
        }
    }
}

fn lock_piece(p: &Piece, board: &mut [[u8; W]; H]) {
    for &(dx, dy) in &SHAPES[p.kind][p.rot] {
        let x = p.x + dx;
        let y = p.y + dy;
        if y >= 0 && x >= 0 && x < W as i32 && y < H as i32 {
            board[y as usize][x as usize] = (p.kind + 1) as u8;
        }
    }
}

fn clear_lines(board: &mut [[u8; W]; H]) -> usize {
    let mut cleared = 0usize;
    let mut y = H as i32 - 1;
    while y >= 0 {
        let full = board[y as usize].iter().all(|&c| c != 0);
        if full {
            for yy in (1..=y as usize).rev() {
                board[yy] = board[yy - 1];
            }
            board[0] = [0u8; W];
            cleared += 1;
        } else {
            y -= 1;
        }
    }
    cleared
}

fn render(board: &[[u8; W]; H], cur: &Piece, score: usize) {
    print!("\x1b[H");
    println!("Tetris  WASD move/rotate  q quit  score={score}");
    print!("+");
    for _ in 0..W {
        print!("--");
    }
    println!("+");
    for y in 0..H {
        print!("|");
        for x in 0..W {
            let mut v = board[y][x];
            for &(dx, dy) in &SHAPES[cur.kind][cur.rot] {
                let px = cur.x + dx;
                let py = cur.y + dy;
                if px == x as i32 && py == y as i32 {
                    v = (cur.kind + 1) as u8;
                }
            }
            if v == 0 {
                print!("  ");
            } else {
                print!("[]");
            }
        }
        println!("|");
    }
    print!("+");
    for _ in 0..W {
        print!("--");
    }
    println!("+");
}

fn poll_key() -> Option<u8> {
    let mut c = [0u8; 1];
    match read(STDIN, &mut c) {
        n if n > 0 => Some(c[0]),
        _ => None,
    }
}
