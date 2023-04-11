use std::collections::HashMap;
use std::sync::Mutex;
#[macro_use]
extern crate rocket;
use common::{Game, Message, Player};
use rocket::State;
use rocket_dyn_templates::{Template, context};
use rocket_dyn_templates::serde::Serialize;

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
    Template::render("index", context!{name: "world"})
}

#[get("/games")]
fn list_games(games: &State<Games>) -> Template {
    let games = games.lock().expect("lock shared data");
    #[derive(Serialize)]
    struct GameInfo {
        id: String,
        uri: String,
    }
    let games:Vec<GameInfo> = games.keys().map(|id| GameInfo{id:id.clone(), uri:uri!(game(id)).to_string()}).collect();

    Template::render("list_games", context!{games: games})
}

#[get("/games/<id>")]
fn game(id: String, games: &State<Games>) -> String {
    let mut games = games.lock().expect("lock shared data");
    if games.get(&id).is_none() {
        games.insert(id.clone(), GameRunner::new(id.clone()));
    }
    id.clone()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Games::new(HashMap::new()))
        .mount("/", routes![index])
        .mount("/", routes![game])
        .mount("/", routes![list_games])
        .attach(Template::fairing())
    }
