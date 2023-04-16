use std::collections::HashMap;
use std::sync::Mutex;
#[macro_use]
extern crate rocket;
use common::{Game, Message, Player, Position};
use rocket::form::{Form, FromForm};
use rocket::response::status::BadRequest;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::serde::Serialize;
use rocket_dyn_templates::{context, Template};

struct GameRunner {
    id: String,
    game: Game,
    messages: Vec<Message>,
}

impl GameRunner {
    fn new(id: String) -> GameRunner {
        GameRunner {
            id,
            game: Game::new(),
            messages: Vec::new(),
        }
    }

    fn messages_since(&mut self, first: usize) {
        for msg in self.game.next_actions() {
            self.game.handle(&msg);
            self.messages.push(msg.clone());

            if let Message::Question(question) = msg {}
        }
    }
}

type Games = Mutex<HashMap<String, GameRunner>>;

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {name: "world"})
}

#[get("/games")]
fn list_games(games: &State<Games>) -> Template {
    let games = games.lock().expect("lock shared data");
    #[derive(Serialize)]
    struct GameInfo {
        id: String,
        uri: String,
        status: String,
    }
    let games: Vec<GameInfo> = games
        .values()
        .map(|game| GameInfo {
            id: game.id.clone(),
            uri: uri!(game(&game.id)).to_string(),
            status: game.game.status_message().to_string(),
        })
        .collect();

    Template::render("list_games", context! {games: games})
}

fn css_class_for_position(game: &Game, position: Position) -> &str {
    match game.board[position.index()] {
        Some(Player::Cross) => "cross",
        Some(Player::Nought) => "nought",
        None => "",
    }
}

#[get("/games/<id>")]
fn game(id: String, games: &State<Games>) -> Template {
    let mut games = games.lock().expect("lock shared data");
    if games.get(&id).is_none() {
        games.insert(id.clone(), GameRunner::new(id.clone()));
    }
    let game = &mut games
        .get_mut(&id)
        .expect("the game must exist at theis point")
        .game;

    'until_question: while game.is_running() {
        for msg in game.next_actions() {
            game.handle(&msg);

            println!("Message {:?}", msg);

            if let common::Message::Question(_) = msg {
                break 'until_question;
            }
        }
    }

    #[allow(non_snake_case)]
    #[derive(Serialize)]
    struct Board<'a> {
        TL: &'a str,
        TC: &'a str,
        TR: &'a str,
        CL: &'a str,
        CC: &'a str,
        CR: &'a str,
        BL: &'a str,
        BC: &'a str,
        BR: &'a str,
    }
    use Position::*;
    let board = Board {
        TL: css_class_for_position(game, TopLeft),
        TC: css_class_for_position(game, TopCenter),
        TR: css_class_for_position(game, TopRight),
        CL: css_class_for_position(game, CenterLeft),
        CC: css_class_for_position(game, Center),
        CR: css_class_for_position(game, CenterRight),
        BL: css_class_for_position(game, BottomLeft),
        BC: css_class_for_position(game, BottomCenter),
        BR: css_class_for_position(game, BottomRight),
    };
    Template::render("game", context! {board: board, status: game.status_message()})
}

#[allow(non_snake_case)]
#[derive(Debug, FromForm)]
struct Answer {
    TL: Option<String>,
    TC: Option<String>,
    TR: Option<String>,
    CL: Option<String>,
    CC: Option<String>,
    CR: Option<String>,
    BL: Option<String>,
    BC: Option<String>,
    BR: Option<String>,
}

#[post("/games/<id>", data = "<answer>")]
fn answer(
    id: String,
    answer: Form<Answer>,
    games: &State<Games>,
) -> Result<Redirect, BadRequest<()>> {
    use Position::*;
    if let Some(position) = match *answer {
        Answer {
            TL: Some(_),
            TC: None,
            TR: None,
            CL: None,
            CC: None,
            CR: None,
            BL: None,
            BC: None,
            BR: None,
        } => Some(TopLeft),
        Answer {
            TL: None,
            TC: Some(_),
            TR: None,
            CL: None,
            CC: None,
            CR: None,
            BL: None,
            BC: None,
            BR: None,
        } => Some(TopCenter),
        Answer {
            TL: None,
            TC: None,
            TR: Some(_),
            CL: None,
            CC: None,
            CR: None,
            BL: None,
            BC: None,
            BR: None,
        } => Some(TopRight),
        Answer {
            TL: None,
            TC: None,
            TR: None,
            CL: Some(_),
            CC: None,
            CR: None,
            BL: None,
            BC: None,
            BR: None,
        } => Some(CenterLeft),
        Answer {
            TL: None,
            TC: None,
            TR: None,
            CL: None,
            CC: Some(_),
            CR: None,
            BL: None,
            BC: None,
            BR: None,
        } => Some(Center),
        Answer {
            TL: None,
            TC: None,
            TR: None,
            CL: None,
            CC: None,
            CR: Some(_),
            BL: None,
            BC: None,
            BR: None,
        } => Some(CenterRight),
        Answer {
            TL: None,
            TC: None,
            TR: None,
            CL: None,
            CC: None,
            CR: None,
            BL: Some(_),
            BC: None,
            BR: None,
        } => Some(BottomLeft),
        Answer {
            TL: None,
            TC: None,
            TR: None,
            CL: None,
            CC: None,
            CR: None,
            BL: None,
            BC: Some(_),
            BR: None,
        } => Some(BottomCenter),
        Answer {
            TL: None,
            TC: None,
            TR: None,
            CL: None,
            CC: None,
            CR: None,
            BL: None,
            BC: None,
            BR: Some(_),
        } => Some(BottomRight),
        _ => None,
    } {
        let mut games = games.lock().expect("lock shared data");
        if games.get(&id).is_none() {
            return Err(BadRequest::<()>(None));
        }
        let gamerunner = games.get_mut(&id).unwrap();

        gamerunner.game.set_answer(common::Answer(position));

        Ok(Redirect::to(uri!(game(id))))
    } else {
        Err(BadRequest::<()>(None))
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Games::new(HashMap::new()))
        .mount("/", routes![index])
        .mount("/", routes![game])
        .mount("/", routes![answer])
        .mount("/", routes![list_games])
        .attach(Template::fairing())
}
