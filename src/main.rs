#![allow(dead_code)]
mod lib;
use lib::game_items::GameItems::*;
use std::{
    io::{stdin, stdout, Write},
    thread, time,
};
extern crate crossterm;
use crossterm::event::{read, Event};

fn main() -> std::io::Result<()> {
    lib::draw::init();

    let width = 32;
    let height = 32;
    let mut snake_tail: Vec<(i32, i32)> = vec![];
    let mut snake_head = (width / 2, height / 2);
    let mut direction = 0_u8;
    let mut screen = vec![];

    for _ in 0..height {
        let mut line = vec![];
        for _ in 0..width {
            line.push(Empty);
        }
        screen.push(line);
    }

    let mut frame_count = 0;
    let mut frame_start: time::Instant;
    let mut elapsed_time: time::Duration;

    loop {
        frame_start = time::Instant::now();
        frame_count += 1;

        // --- input ---
        // parcially copied from: https://docs.rs/crossterm/0.17.5/crossterm/event/index.html
        direction = match read() /*from crate crossterm*/ { // todo: implement this in multithreading because it is waiting for a keypress, which interrupts the gameloop
            Ok(Event::Key(event)) => {
                println!("{:?}", event.code); 
                match event.code {
                    crossterm::event::KeyCode::Char(c) => {
                        match c.to_ascii_lowercase() {
                            'w' => 3,
                            'a' => 2,
                            's' => 1,
                            'd' => 0,
                            _ => direction
                        }
                    }
                    _ => direction
                }
            },
            Err(e) => panic!("An error occured: {}", e),
            _ => direction
        };

        // Todo: game logic

        screen[snake_head.0][snake_head.1] = SnakeHead(direction);
        direction = frame_count % 4;
        if frame_count == 100 {
            break;
        }
        // lib::draw::render(&screen[..])?;
        println!("{}", direction);
        screen[snake_head.0][snake_head.1] = Empty;
        elapsed_time = frame_start.elapsed();
        // println!("{:?}", elapsed_time);
        if elapsed_time.as_nanos() < 33_333_333 {
            thread::sleep(time::Duration::from_nanos(33_333_333) - elapsed_time);
        }
        lib::draw::
    }
    lib::draw::end();
    Ok(())
}
