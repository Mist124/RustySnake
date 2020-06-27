pub mod game_items {
    #[derive(Debug)]
    pub enum GameItems {
        Apple,
        SnakeTail,
        SnakeHead(u8),
        Empty,
    }
}

pub mod draw {
    pub fn init() {
        print!("\x1B[?25l \x1B[2J"); // hide cursor and clear console
    }

    pub fn end() {
        print!("\x1B[?25h"); // show cursor
    }

    use super::game_items::GameItems;
    use std::io::{self, Write};

    pub fn render(screen: &[Vec<GameItems>]) -> io::Result<()> {
        let mut render = String::from("");
        for i in screen {
            for j in i {
                render.push_str(match j {
                    GameItems::Apple => "()",
                    GameItems::SnakeTail => "[]",
                    GameItems::SnakeHead(d) => match d % 4 {
                        0 => "O<",
                        1 => "/\\",
                        2 => ">O",
                        3 => "\\/",
                        _ => "",
                    },
                    _ => "--",
                });
            }

            render.push_str("\n");
        }
        print!("\x1B[0;0H{}", render);
        io::stdout().flush()?;

        Ok(())
    }
}
