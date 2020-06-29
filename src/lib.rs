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
/*
pub enum TailSegment {
    Def, // default: []
    End(Direction), /*
         Left:   [=
         Right:  =]
         Up:     TT
         Down:   L/
              */
    Middle(u8), /* the different kinds of tailsegments:
                0:  ./
                1:  \.
                2:  /'                    }0=='\
                3:  '\                        ||
                4:  ||                      /'./
                5:  ==                      \.=]
                */
}
*/
#[derive(Debug, Clone, Copy)]
pub enum GameItems {
    Apple,
    SnakeTailSegment, // todo: implement different types of segments (began implementing that above)
    SnakeHead(Direction),
    Empty,
}

impl fmt::Display for GameItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameItems::Empty => "  ",
                GameItems::Apple => "()",
                GameItems::SnakeTailSegment => "[]",
                GameItems::SnakeHead(d) => match d {
                    Direction::Right => "0{",
                    Direction::Down => "/\\",
                    Direction::Left => "}0",
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
    result.push_str(" .");
    for _ in 0..screen[0].len() {
        result.push_str(",.")
    }
    result.push_str(",\n");
    for i in screen {
        result.push_str(" ¦");
        for j in i {
            result.push_str(&format!("{}", j));
        }

        result.push_str("¦\n");
    }
    result.push_str(" ¨");
    for _ in 0..screen[0].len() {
        result.push_str("\"¨")
    }
    result.push_str("\"\n");
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

fn reset_screen(s: &[Vec<GameItems>]) -> Vec<Vec<GameItems>> {
    let h = s.len();
    let w = s[0].len();
    vec![vec![GameItems::Empty; w]; h]
}

pub fn game() -> std::io::Result<()> {
    use crossterm::event::read;
    use std::sync::mpsc;
    use GameItems::*;
    init();

    let width = 48;
    let height = 24;
    let fps = 15_f64;
    let nanos = (1_f64 / fps * 1_000_000_000_f64) as u128;
    let mut snake_tail: Vec<(usize, usize)> = vec![];
    let mut snake_head = (height / 2, width / 2);
    let mut direction = Direction::Up;
    let mut screen = vec![vec![Empty; width]; height];

    let mut frame_count = 0_u32;
    let mut frame_start: time::Instant;
    let mut elapsed_time: time::Duration;

    // multithreaded input loop
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
        snake_tail.push(snake_head);
        if snake_tail.len() > 10 {
            snake_tail.remove(0);
        }
        snake_head = match direction {
            Direction::Right => (snake_head.0, snake_head.1 + 1),
            Direction::Left => (snake_head.0, snake_head.1 - 1),
            Direction::Down => (snake_head.0 + 1, snake_head.1),
            Direction::Up => (snake_head.0 - 1, snake_head.1),
        };
        for i in &snake_tail {
            screen[i.0][i.1] = GameItems::SnakeTailSegment;
        }
        screen[snake_head.0][snake_head.1] = SnakeHead(direction);

        render(&screen, frame_count)?;
        // println!("{:?}", direction);
        screen = reset_screen(&screen);
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
