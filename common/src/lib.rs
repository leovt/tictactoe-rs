use std::{collections::HashMap, iter::Map};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Nought,
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    PlayersTurn(Player),
    CheckWin(Player),
    PlayerWon(Player),
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Position {
    fn index(&self) -> usize {
        use Position::*;
        match *self {
            TopLeft => 0,
            TopCenter => 1,
            TopRight => 2,
            CenterLeft => 3,
            Center => 4,
            CenterRight => 5,
            BottomLeft => 6,
            BottomCenter => 7,
            BottomRight => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question(Player, Vec<Position>);

impl Question {
    fn validate(&self, answer: &Answer) -> bool {
        let Question(_, choices) = self;
        let Answer(choice) = *answer;
        choices.contains(&choice)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer(Position);

#[derive(Debug, Clone)]
pub enum Message {
    Move(Player, Position),
    Turn(Player),
    Win(Player),
    Draw,
    Question(Question),
    AcceptAnswer,
    RejectAnswer(Question, Answer),
}

#[derive(Debug)]
pub struct Game {
    phase: Phase,
    board: [Option<Player>; 9],
    answer: Option<Answer>,
}

impl Game {
    fn new() -> Game {
        Game {
            phase: Phase::PlayersTurn(Player::Nought),
            board: [None; 9],
            answer: None,
        }
    }

    fn ask(&self, question: Question, msg: &mut Vec<Message>) -> Option<Answer> {
        if let Some(answer) = &self.answer {
            if question.validate(&answer) {
                msg.push(Message::AcceptAnswer);
                Some(answer.clone())
            } else {
                msg.push(Message::RejectAnswer(question, answer.clone()));
                None
            }
        } else {
            msg.push(Message::Question(question));
            None
        }
    }

    fn next_actions(&self) -> Vec<Message> {
        let mut msg = Vec::new();
        match self.phase {
            Phase::PlayersTurn(player) => {
                use Position::*;
                static POSITIONS: [Position; 9] = [
                    TopLeft,
                    TopCenter,
                    TopRight,
                    CenterLeft,
                    Center,
                    CenterRight,
                    BottomLeft,
                    BottomCenter,
                    BottomRight,
                ];

                let question = Question(
                    player,
                    POSITIONS
                        .iter()
                        .cloned()
                        .filter(|p| self.board[p.index()] == None)
                        .collect(),
                );

                if let Some(Answer(position)) = self.ask(question, &mut msg) {
                    msg.push(Message::Move(player, position))
                }
            }
            Phase::PlayerWon(_) => {}
            Phase::Draw => {}
            Phase::CheckWin(player) => {
                use Position::*;
                static CHECKS: [[Position; 3]; 8] = [
                    [TopLeft, TopCenter, TopRight],
                    [CenterLeft, Center, CenterRight],
                    [BottomLeft, BottomCenter, BottomRight],
                    [TopLeft, CenterLeft, BottomLeft],
                    [TopCenter, Center, BottomCenter],
                    [TopRight, CenterRight, BottomRight],
                    [TopLeft, Center, BottomRight],
                    [TopRight, Center, BottomLeft],
                ];
                println!("{} empty spaces", self.board.iter().filter(|x| x.is_none()).count());
                if self.board.iter().filter(|x| x.is_none()).count() == 0 {
                    msg.push(Message::Draw)
                } else {
                    use Player::*;
                    msg.push(Message::Turn(match player {
                        Nought => Cross,
                        Cross => Nought,
                    }))
                }
            }
        };
        msg
    }
}

impl Listener for Game {
    fn handle(&mut self, msg: &Message) {
        use Message::*;
        match msg {
            Move(player, position) => {
                assert!(self.board[position.index()] == None);
                self.board[position.index()] = Some(*player);
                self.phase = Phase::CheckWin(*player);
            }
            Win(player) => {
                self.phase = Phase::PlayerWon(*player);
            }
            Draw => {
                self.phase = Phase::Draw;
            }
            Turn(player) => {
                self.phase = Phase::PlayersTurn(*player);
            }
            Question(_) => {}
            AcceptAnswer => {self.answer=None}
            RejectAnswer(_, _) => {self.answer=None}
        }
    }
}

trait Listener {
    fn handle(&mut self, msg: &Message);
}

struct MessagePrinter {}

impl Listener for MessagePrinter {
    fn handle(&mut self, msg: &Message) {
        println!("{:?}", msg);
    }
}

trait Agent {
    fn answer(&mut self, question: &Question) -> Answer;
}

struct RandomAgent {
    random: u32,
}

impl Agent for RandomAgent {
    fn answer(&mut self, question: &Question) -> Answer {
        self.random = self.random.wrapping_mul(22695477).wrapping_add(1);
        let index = ((self.random >> 16) as usize) % (question.1.len());
        Answer(question.1[index])
    }
}

struct GameLoop {
    game: Game,
    listeners: Vec<Box<dyn Listener>>,
    agents: HashMap<Player, Box<dyn Agent>>,
}

impl GameLoop {
    fn run(&mut self) {
        while match self.game.phase {
            Phase::PlayerWon(_) => false,
            Phase::Draw => false,
            _ => true,
        } {
            for msg in self.game.next_actions() {
                self.game.handle(&msg);

                for listener in self.listeners.iter_mut() {
                    listener.handle(&msg);
                }

                if let Message::Question(question) = msg {
                    let answer = self.agents.get_mut(&question.0).unwrap().answer(&question);
                    self.game.answer = Some(answer);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let game = Game::new();
        let mut gl = GameLoop {
            game: game,
            listeners: vec![Box::new(MessagePrinter {})],
            agents: HashMap::new(),
        };
        gl.agents
            .insert(Player::Cross, Box::new(RandomAgent { random: 67 }));
        gl.agents
            .insert(Player::Nought, Box::new(RandomAgent { random: 97 }));

        gl.run();
    }
}
