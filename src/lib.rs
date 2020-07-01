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

impl Direction {
    fn opposite(&self) -> Direction {
        use Direction::*;
        match self {
            Left => Right,
            Right => Left,
            Up => Down,
            Down => Up,
        }
    }
}

impl PartialEq for Direction {
    fn eq(&self, &other: &Direction) -> bool {
        use Direction::*;
        match (*self, other) {
            (Left, Left) => true,
            (Right, Right) => true,
            (Up, Up) => true,
            (Down, Down) => true,
            _ => false,
        }
    }
}

pub enum Input {
    Quit,
    D(Direction),
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentType {
    Def, // default: []
    End(Direction), /*
         Left:   [=
         Right:  =]
         Up:     TT
         Down:   L/
              */
    Corner(Direction), /* the different kinds of tailsegments:
                Left:   ./
                Up:     \.
                Right:  /'
                Down:   '\
                */
    Straight(Direction), /*
                Up/Down:    ||
                Right/Left: ==
    */
}

struct Segment {
    position: (u32, u32),
    kind: SegmentType,
}

impl Segment {
    fn new(pos: (u32, u32)) -> Segment {
        Segment { position: pos, kind: SegmentType::Def}
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameItems {
    Apple,
    SnakeTailSegment(SegmentType), // todo: implement different types of segments (began implementing that above)
    SnakeHead(Direction),
    Empty,
}

impl fmt::Display for GameItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        use GameItems::*;
        use SegmentType::*;
        use Direction::*;
        write!(
            f,
            "{}",
            match self {
                Empty => "  ",
                Apple => "()",
                SnakeTailSegment(s) => match s {
                    Def => "[]",
                    End(d) => match d {
                        Left => "[=",
                        Right => "=]",
                        Up => "TT",
                        Down => "L/",
                    },
                    Corner(d) => match d {
                        Left => "./",
                        Up => "\\.",
                        Right => "/'",
                        Down => "'\\",
                    },
                    Straight(d) => match d {
                        Down => "||",
                        Up => "||",
                        Right => "==",
                        Left => "==",
                    }
                },
                GameItems::SnakeHead(d) => match d {
                    Right => "0{",
                    Down => "/\\",
                    Left => "}0",
                    Up => "\\/",
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

fn _update_snake(snake: &Vec<Segment>) -> &Vec<Segment> {
    // todo: implement update function to generate the shape of the snake
    snake
}

fn render(screen: &[Vec<GameItems>], score: u32) -> io::Result<()> {
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
    result.push_str(&format!("score: {}\n", score));
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
    use rand::Rng;
    init();
    
    // -- Configuration --
    let width = 32;
    let height = 16;
    let fps = 10_f64;
    let nanos = (1_f64 / fps * 1_000_000_000_f64) as u128;

    // -- Game Variables --
    let mut snake_tail: Vec<Segment> = vec![];
    let mut snake_len = 10_u32;
    let mut score = 0_u32;
    let mut collided: bool;
    let mut rng = rand::thread_rng();
    let mut apple = (rng.gen_range(0, height), rng.gen_range(0, width));
    let mut snake_head = (height / 2 as u32, width / 2 as u32);
    let mut direction = Direction::Up;
    let mut screen = vec![vec![Empty; width as usize]; height as usize];

    let mut _frame_count = 0_u32;
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

    'main: loop {
        frame_start = time::Instant::now();
        _frame_count += 1;

        // --- input ---
        direction = if let Ok(input) = rx.try_recv() {
            if let Input::D(d) = input {
                if d == direction.opposite() { // prevents you to eat yourself by going in the opposite direction your facing
                    direction
                } else {
                    d
                }
            } else if let Input::Quit = input {
                break 'main;
            } else {
                direction
            }
        } else {
            direction
        };

        // -- game logic --
        snake_tail.push(Segment::new(snake_head));
        if snake_tail.len() > snake_len as usize {
            snake_tail.remove(0);
        }

        snake_head = match direction {
            Direction::Right => if snake_head.1 < width - 1 {(snake_head.0, snake_head.1 + 1)} else {break 'main;},
            Direction::Left  => if snake_head.1 > 0 {(snake_head.0, snake_head.1 - 1)} else {break 'main;},
            Direction::Down  => if snake_head.0 < height - 1 {(snake_head.0 + 1, snake_head.1)} else {break 'main;},
            Direction::Up    => if snake_head.0 > 0 {(snake_head.0 - 1, snake_head.1)} else {break 'main;},
        };

        collided = snake_head == apple;
        snake_len = if collided {snake_len + 2} else {snake_len};
        score = if collided {score + 1} else {score};
        apple = if collided {(rng.gen_range(0, height), rng.gen_range(0, width))} else {apple};

        // -- putting everything onto the screen --
        screen[apple.0 as usize][apple.1 as usize] = Apple;
        for i in &snake_tail {
            if i.position == snake_head {break 'main;}
            screen[i.position.0 as usize][i.position.1 as usize] = GameItems::SnakeTailSegment(SegmentType::Def);
        }
        screen[snake_head.0 as usize][snake_head.1 as usize] = SnakeHead(direction);

        // rendering the screen and wait the rest of the time to get the desired framerate
        render(&screen, score)?;
        screen = reset_screen(&screen);
        elapsed_time = frame_start.elapsed();
        if elapsed_time.as_nanos() < nanos {
            thread::sleep(time::Duration::from_nanos(nanos as u64) - elapsed_time);
        }
    }
    end();
    Ok(())
}
