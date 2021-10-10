use rand::Rng;
use std::io::{stdin, stdout, Read, Write};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use piston_window;
use piston_window::Window;

#[derive(PartialEq)]
enum Live {
    Alive,
    Dead,
}

struct Cell {
    live: Live,
    birth_day: u64,
}

struct Board {
    board: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
}

const DIRECTIONS: [[i32; 2]; 8] = [
    [-1, -1],
    [-1, 0],
    [-1, 1],
    [0, -1],
    [0, 1],
    [1, -1],
    [1, 0],
    [1, 1],
];

impl Board {
    fn new(width: usize, height: usize) -> Board {
        let mut board: Vec<Vec<Cell>> = Vec::new();
        for _ in 0..height {
            let mut line: Vec<Cell> = Vec::new();
            for w in 0..width {
                line.push(Cell {
                    live: Live::Alive,
                    birth_day: 0,
                })
            }
            board.push(line);
        }
        Board {
            board,
            width,
            height,
        }
    }
    fn set(&mut self, w: usize, h: usize, cell: Cell) {
        self.board[h][w] = cell
    }

    fn is_alive(&self, w: usize, h: usize) -> bool {
        self.board[h][w].live == Live::Alive
    }
    fn alive_neighbors_count(&self, w: usize, h: usize) -> u8 {
        let mut count: u8 = 0;
        for dir in DIRECTIONS {
            let _w: i32 = w as i32 + dir[0];
            let _h = h as i32 + dir[1];
            if _w < 0 || _w as usize >= self.width || _h < 0 || _h as usize >= self.height {
                continue;
            }
            if self.is_alive(_w as usize, _h as usize) {
                count += 1;
            }
        }
        count
    }
}

struct Universe {
    twin: Vec<Board>,
    iboard: usize,
    now: u64,
    width: usize,
    height: usize,
}

impl Universe {
    fn new(width: usize, height: usize) -> Universe {
        let mut u: Universe = Universe {
            iboard: 0,
            twin: vec![],
            now: 0,
            width: width,
            height: height,
        };
        u.twin.push(Board::new(width, height));
        u.twin.push(Board::new(width, height));

        for h in 0..height {
            for w in 0..width {
                let mut live: Live;
                if rand::thread_rng().gen_bool(1.0 / 2.0) {
                    live = Live::Alive;
                } else {
                    live = Live::Dead;
                }
                u.twin[0].set(w, h, Cell { live, birth_day: 0 })
            }
        }
        u
    }
    fn getNowBoard(&mut self) -> &Board {
        &self.twin[self.iboard]
    }
    fn getBoard(&mut self, i: usize) -> &Board {
        &self.twin[i]
    }
    fn tick(&mut self) {
        let prev_i = self.iboard;
        let now_i = (prev_i + 1) % 2;
        self.now += 1;
        for h in 0..self.height {
            for w in 0..self.width {
                let mut live: Live;
                match self.getBoard(prev_i).alive_neighbors_count(w, h) {
                    0..=1 => {
                        live = Live::Dead;
                    }
                    3 => {
                        live = Live::Alive;
                    }
                    4..=u8::MAX => {
                        live = Live::Dead;
                    }
                    _ => {
                        if self.getBoard(prev_i).is_alive(w, h) {
                            live = Live::Alive;
                        } else {
                            live = Live::Dead;
                        }
                    }
                }
                self.twin[now_i].set(
                    w,
                    h,
                    Cell {
                        live,
                        birth_day: self.now,
                    },
                )
            }
        }
        self.iboard = now_i;
    }
}

fn console_game() {
    let mut width: usize = 0;
    let mut height: usize = 0;

    match termion::terminal_size() {
        Ok(size) => {
            width = size.0 as usize;
            height = size.1 as usize;
        }
        Err(err) => {
            panic!(err)
        }
    }
    let mut universe: Universe = Universe::new(width, height);

    let mut stdout = stdout().into_raw_mode().unwrap();

    writeln!(
        stdout,
        "{}{}{}Use the up/down arrow keys to change the blue in the rainbow.",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
        .unwrap();

    writeln!(stdout, "{}", termion::clear::All).unwrap();
    loop {
        universe.tick();

        let board = universe.getNowBoard();
        for h in 0..board.height {
            writeln!(stdout, "{}", termion::cursor::Goto(1, h as u16 + 1)).unwrap();
            for w in 0..board.width {
                if board.is_alive(w, h) {
                    write!(stdout, "◼").unwrap();
                } else {
                    write!(stdout, " ").unwrap();
                }
            }
        }
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn game_2d() {
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("game of life 2d", [1920, 1080])
            .exit_on_esc(true).fullscreen(false).build().unwrap();

    let window_size = window.size();
    let color_dead = piston_window::color::BLACK;
    let color_alive = piston_window::color::WHITE;

    const cell_length: f64 = 5.0;
    let cell_Rec = |w: f64, h: f64| -> [f64; 4]{
        let _w = w * cell_length;
        let _h = h * cell_length;
        [_w, _h, cell_length, cell_length]
    };
    let height = (window_size.height as f64 / cell_length) as usize;
    let width = (window_size.width as f64 / cell_length) as usize;

    let mut universe: Universe = Universe::new(width, height);


    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics, _device| {
            universe.tick();

            piston_window::rectangle(color_dead,
                                     [0.0, 0.0, 1920.0, 1080.0],
                                     context.transform,
                                     graphics);

            let board = universe.getNowBoard();
            for h in 0..height {
                for w in 0..width {
                    if board.is_alive(w, h) {
                        piston_window::grid::GridCells::from()
                        piston_window::rectangle(color_alive,
                                                 cell_Rec(w as f64, h as f64),
                                                 context.transform,
                                                 graphics);
                    }
                }
            }
            // std::thread::sleep(std::time::Duration::from_millis(1));
        });
    }
}

fn game_3d() {}

fn main() {
    let pattern = std::env::args().nth(1).expect("no game mode given");
    match pattern.as_str() {
        "console" => {
            console_game();
        }
        "2d" => {
            game_2d()
        }
        "3d" => {
            game_3d()
        }
        _ => {
            println!("unknown game mode");
        }
    }
}
