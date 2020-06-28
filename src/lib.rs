use std::fmt;
use std::io::{self, Write};
use std::{thread, time};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

pub enum Input {
    Quit,
    D(Direction),
}

#[derive(Debug)]
pub enum GameItems {
    Apple,
    SnakeTail,
    SnakeHead(Direction),
    Empty,
}

impl fmt::Display for GameItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameItems::Empty => "..",
                GameItems::Apple => "()",
                GameItems::SnakeTail => "[]",
                GameItems::SnakeHead(d) => match d {
                    Direction::Right => "O<",
                    Direction::Down => "/\\",
                    Direction::Left => ">O",
                    Direction::Up => "\\/",
                },
            }
        )
    }
}

fn init() {
    print!("\x1B[?25l \x1B[2J"); // hide cursor and clear console
}

fn end() {
    print!("\x1B[?25h"); // show cursor
}

fn render(screen: &[Vec<GameItems>], fc: u32) -> io::Result<()> {
    let mut result = String::from("");
    for i in screen {
        for j in i {
            result.push_str(&format!("{}", j));
        }

        result.push_str("\n");
    }
    result.push_str(&format!("{}\n", fc));
    print!("\x1B[0;0H{}", result);
    io::stdout().flush()?;

    Ok(())
}

fn handle_keyevent(event: crossterm::event::Event) -> Option<Input> {
    use Direction::*;
    use Input::*;
    // parcially copied from: https://docs.rs/crossterm/0.17.5/crossterm/event/index.html
    match event {
        crossterm::event::Event::Key(key_event) => {
            // println!("{:?}", key_event.code);
            match key_event.code {
                crossterm::event::KeyCode::Char(c) => match c.to_ascii_lowercase() {
                    'w' => Some(D(Up)),
                    'a' => Some(D(Left)),
                    's' => Some(D(Down)),
                    'd' => Some(D(Right)),
                    'q' => Some(Quit),
                    _ => None,
                },
                crossterm::event::KeyCode::Up => Some(D(Up)),
                crossterm::event::KeyCode::Left => Some(D(Left)),
                crossterm::event::KeyCode::Down => Some(D(Down)),
                crossterm::event::KeyCode::Right => Some(D(Right)),
                crossterm::event::KeyCode::Esc => Some(Quit),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn game() -> std::io::Result<()> {
    use crossterm::event::{read, Event};
    use std::sync::mpsc;
    use std::thread;
    use GameItems::*;
    init();

    let width = 17;
    let height = 32;
    let fps = 10_f32;
    let nanos = (1_f32 / fps * 1_000_000_000_f32) as u128;
    let mut snake_tail: Vec<(usize, usize)> = vec![];
    let mut snake_head = (height / 2, width / 2);
    let mut direction = Direction::Up;
    let mut screen = vec![];

    for _ in 0..height {
        let mut line = vec![];
        for _ in 0..width {
            line.push(Empty);
        }
        screen.push(line);
    }

    let mut frame_count = 0_u32;
    let mut frame_start: time::Instant;
    let mut elapsed_time: time::Duration;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let input = handle_keyevent(read().unwrap());
        if let Some(d) = input {
            tx.send(d).unwrap();
        }
    });

    loop {
        frame_start = time::Instant::now();
        frame_count += 1;

        // --- input ---
        ///// todo: implement this in multithreading because it is waiting for a keypress, which interrupts the gameloop
        direction = if let Ok(input) = rx.try_recv() {
            if let Input::D(d) = input {
                d
            } else if let Input::Quit = input {
                break;
            } else {
                direction
            }
        } else {
            direction
        };

        // Todo: game logic
        snake_head = match direction {
            Direction::Right => (snake_head.0, snake_head.1 + 1),
            Direction::Left => (snake_head.0, snake_head.1 - 1),
            Direction::Down => (snake_head.0 + 1, snake_head.1),
            Direction::Up => (snake_head.0 - 1, snake_head.1),
        };
        screen[snake_head.0][snake_head.1] = SnakeHead(direction);

        render(&screen[..], frame_count)?;
        // println!("{:?}", direction);
        screen[snake_head.0][snake_head.1] = Empty;
        // if frame_count == 100 {
        //     break;
        // }
        elapsed_time = frame_start.elapsed();
        if elapsed_time.as_nanos() < nanos {
            thread::sleep(time::Duration::from_nanos(nanos as u64) - elapsed_time);
        }
    }
    end();
    Ok(())
}
