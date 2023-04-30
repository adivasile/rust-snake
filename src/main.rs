use crossterm::{cursor, event, execute, queue, terminal, style};
use crossterm::terminal::{ClearType, EnterAlternateScreen, LeaveAlternateScreen };
use crossterm::event::*;
use std::io::{stdout, self};
use std::io::Write;
use std::time::Duration;
use rand::Rng;

struct Rectangle {
    top_left: (u16, u16),
    bottom_right: (u16, u16),
}

impl Rectangle {
    fn new(top_left: (u16, u16), bottom_right: (u16, u16)) -> Self {
       Self {
           top_left,
           bottom_right,
       } 
    }

    fn render(&self, output: &mut Output) -> crossterm::Result<()> {
        queue!(
            output,
            style::SetAttribute(style::Attribute::Reverse),
            cursor::MoveTo(self.top_left.0, self.top_left.1),
            style::Print(" ".repeat((self.bottom_right.0 - self.top_left.0) as usize)),
            cursor::MoveTo(self.top_left.0, self.bottom_right.1),
            style::Print(" ".repeat((self.bottom_right.0 - self.top_left.0) as usize)),
        )?;

        for i in self.top_left.1 .. self.bottom_right.1 + 1 {
            queue!(
                output,
                cursor::MoveTo(self.top_left.0 as u16, i as u16),
                style::Print(" "),
                cursor::MoveTo(self.bottom_right.0 as u16, i as u16),
                style::Print(" "),
            )?;
        }

        Ok(())
    }
}

struct Output {
    content: String,
}

impl Output {
    fn new() -> Self {
        Self { content: String::new() }
    }
}

impl io::Write for Output {
     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            },
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

enum Action {
    Tick,
    Quit,
    StartGame,
    Restart,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

struct Reader;

impl Reader {
   pub fn read_key(&self)  -> crossterm::Result<Action> {
       if event::poll(Duration::from_millis(300))? {
           if let Event::Key(event) = event::read().unwrap() {
               return match event {
                   KeyEvent {
                       code: KeyCode::Char(' '),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::StartGame),
                   KeyEvent {
                       code: KeyCode::Char('q'),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::Quit),
                   KeyEvent {
                       code: KeyCode::Char('y'),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::Restart),
                   KeyEvent {
                       code: KeyCode::Char('k'),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveUp),
                   KeyEvent {
                       code: KeyCode::Up,
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveUp),
                   KeyEvent {
                       code: KeyCode::Char('j'),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveDown),
                   KeyEvent {
                       code: KeyCode::Down,
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveDown),
                   KeyEvent {
                       code: KeyCode::Char('h'),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveLeft),
                   KeyEvent {
                       code: KeyCode::Left,
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveLeft),
                   KeyEvent {
                       code: KeyCode::Char('l'),
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveRight),
                   KeyEvent {
                       code: KeyCode::Right,
                       modifiers: event::KeyModifiers::NONE,
                   } => Ok(Action::MoveRight),
                   _ => Ok(Action::Tick)
               }
           } 
       }

       return Ok(Action::Tick);
   }
}

#[derive(PartialEq)]
enum GameState {
    Menu,
    Play,
    GameOver,
}

struct Game {
    output: Output,
    frame: Rectangle,
    reader: Reader,
    snake: Snake,
    score: usize,
    state: GameState,
    food: Point,
}

impl Game {
    fn new() -> Self {
        Self { 
            output: Output::new(),
            frame: Rectangle::new((30, 30), (100, 60)),
            reader: Reader,
            snake: Snake::new(),
            score: 0,
            state: GameState::Menu,
            food: Point::new(40, 45),
        }
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.snake.slither();
        self.check_collisions();
        self.feed_snake();
        match self.state {
            GameState::Menu => self.menu()?,
            GameState::Play => self.tick()?,
            GameState::GameOver => self.game_over()?,
        };

        self.output.flush()?;

        self.process_keypress()
    }

    fn feed_snake(&mut self) { 
        if self.snake.body[0] == self.food {
            self.food = self.place_new_food();
            self.snake.grow();
            self.score += 1;
        }
    }

    fn place_new_food(&mut self) -> Point {
        let new_food = Point::new(
            rand::thread_rng().gen_range(self.frame.top_left.0 + 1..self.frame.bottom_right.0 - 1),
            rand::thread_rng().gen_range(self.frame.top_left.1 + 1..self.frame.bottom_right.1 - 1),
        );

        loop {
            let mut ok = true;
            for point in self.snake.body.iter() {
                if point.x == new_food.x && point.y == new_food.y {
                    ok = false;
                    break;
                }
            }

            if ok {
                break;
            }
        }
    
        return new_food;
    }

    fn menu(&mut self) -> crossterm::Result<()> {
        let score_pos = (self.frame.top_left.0, self.frame.top_left.1 - 1);
        let score = self.score;

        queue!(
            self.output,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(score_pos.0, score_pos.1),
            style::Print(format!("Score: {}", score))
        )?;

        let menu_pos = (self.frame.top_left.0 + 10, self.frame.top_left.0 + 10);
        queue!(
            self.output,
            cursor::MoveTo(menu_pos.0, menu_pos.1),
            style::Print(format!("Welcome to snake in RUST in the terminal")),
            cursor::MoveTo(menu_pos.0, menu_pos.1 + 1),
            style::Print(format!("h/j/k/l or arrow keys to move")),
            cursor::MoveTo(menu_pos.0, menu_pos.1 + 2),
            style::Print(format!("SPACE to start game")),
            cursor::MoveTo(menu_pos.0, menu_pos.1 + 3),
            style::Print(format!("q to quit")),
        )?;

        self.frame.render(&mut self.output)?;

        Ok(())
    }

    fn game_over(&mut self) -> crossterm::Result<()> {
        let score_pos = (self.frame.top_left.0, self.frame.top_left.1 - 1);
        let score = self.score;

        queue!(
            self.output,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(score_pos.0, score_pos.1),
            style::Print(format!("Score: {}", score))
        )?;

        self.frame.render(&mut self.output)?;

        let game_over_pos = (self.frame.top_left.0 + 10, self.frame.top_left.0 + 10);
        queue!(
            self.output,
            cursor::MoveTo(game_over_pos.0, game_over_pos.1),
            style::Print(format!("Game over! Score: {}", score)),
            cursor::MoveTo(game_over_pos.0, game_over_pos.1 + 1),
            style::Print(format!("Play again? Y/N")),
        )?;
        Ok(())
    }

    fn tick(&mut self) -> crossterm::Result<()> {
        let score_pos = (self.frame.top_left.0, self.frame.top_left.1 - 1);
        let score = self.score;

        queue!(
            self.output,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(score_pos.0, score_pos.1),
            style::Print(format!("Score: {}", score))
        )?;

        self.frame.render(&mut self.output)?;
        self.snake.render(&mut self.output)?;
        
        let food_pos = (self.food.x, self.food.y);

        queue!(
            self.output,
            cursor::MoveTo(food_pos.0, food_pos.1),
            style::Print("@")
        )?;

        Ok(())
    }

    fn check_collisions(&mut self) {
        let head = &self.snake.body[0];

        if head.x <= self.frame.top_left.0 {
            self.state = GameState::GameOver;
        }

        if head.x >= self.frame.bottom_right.0 {
            self.state = GameState::GameOver;
        }

        if head.y <= self.frame.top_left.1 {
            self.state = GameState::GameOver;
        }

        if head.y >= self.frame.bottom_right.1 {
            self.state = GameState::GameOver;
        }

        for point_idx in 1..self.snake.body.len() - 1 {
            let body_part = &self.snake.body[point_idx];

            if head.x == body_part.x && head.y == body_part.y {
                self.state = GameState::GameOver;
            }
        }
    }


    fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            Action::Quit => return Ok(false),
            Action::StartGame => {
                if self.state == GameState::Menu {
                    self.state = GameState::Play;
                }
                return Ok(true)
            },
            Action::Restart => {
                if self.state == GameState::GameOver {
                    self.snake = Snake::new();
                    self.state = GameState::Play;
                }
                return Ok(true)
            },
            Action::MoveUp => {
                if self.snake.direction == Direction::Right || self.snake.direction == Direction::Left {
                    self.snake.direction = Direction::Up;
                }
                return Ok(true)
            },
            Action::MoveDown => {
                if self.snake.direction == Direction::Right || self.snake.direction == Direction::Left {
                    self.snake.direction = Direction::Down;
                }
                return Ok(true)
            },
            Action::MoveLeft => {
                if self.snake.direction == Direction::Up || self.snake.direction == Direction::Down {
                    self.snake.direction = Direction::Left;
                }
                return Ok(true)
            },
            Action::MoveRight => {
                if self.snake.direction == Direction::Up || self.snake.direction == Direction::Down {
                    self.snake.direction = Direction::Right;
                }
                return Ok(true)
            },
            _ => return Ok(true)
        }
    }
}

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
        execute!(stdout(), terminal::Clear(ClearType::All)).expect("error");
        execute!(stdout(), cursor::Show).expect("Error");
        execute!(stdout(), cursor::MoveTo(0, 0)).expect("Error");
    }
}

#[derive(PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

impl Point {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}

struct Snake {
    body: Vec<Point>,
    direction: Direction,
    grow: bool,
}

impl Snake {
    fn new() -> Self {
        Self {
            direction: Direction::Left,
            grow: false,
            body: vec![
                Point::new(60, 50),
                Point::new(61, 50),
                Point::new(62, 50),
                Point::new(63, 50),
                Point::new(64, 50),
                Point::new(65, 50),
                Point::new(66, 50),
                Point::new(67, 50),
                Point::new(68, 50),
                Point::new(69, 50),
                Point::new(70, 50),
                Point::new(71, 50),
                Point::new(72, 50),
            ]
        }
    }

    fn grow(&mut self) {
        self.grow = true
    }

    fn render(&self, output: &mut Output) -> crossterm::Result<()> {
        queue!(output, style::SetAttribute(style::Attribute::Reset))?;
        for point in self.body.iter() {
            queue!(
                output,
                cursor::MoveTo(point.x, point.y),
                style::Print("#"),
            )?
        }
        Ok(())
    }

    fn slither(&mut self) {
        match self.direction {
            Direction::Up => self.move_up(),
            Direction::Down => self.move_down(),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
        }
    }

    fn move_left(&mut self) {
        let head = self.body.first().unwrap();
        self.body.insert(0, Point::new(head.x - 1, head.y));
        if !self.grow {
            self.body.pop();
        } else {
            self.grow = false;
        }
    }

    fn move_right(&mut self) {
        let head = self.body.first().unwrap();
        self.body.insert(0, Point::new(head.x + 1, head.y));
        if !self.grow {
            self.body.pop();
        } else {
            self.grow = false;
        }
    }

    fn move_up(&mut self) {
        let head = self.body.first().unwrap();
        self.body.insert(0, Point::new(head.x, head.y - 1));
        if !self.grow {
            self.body.pop();
        } else {
            self.grow = false;
        }
    }

    fn move_down(&mut self) {
        let head = self.body.first().unwrap();
        self.body.insert(0, Point::new(head.x, head.y + 1));
        if !self.grow {
            self.body.pop();
        } else {
            self.grow = false;
        }
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;

    terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let mut game = Game::new();
    while game.run()? {}

    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
