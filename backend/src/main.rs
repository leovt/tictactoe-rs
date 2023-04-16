use std::collections::HashMap;
use std::sync::Mutex;
#[macro_use]
extern crate rocket;
use common::{Game, Message};
use rocket_dyn_templates::{context, Template};

mod prd_interface;

pub struct GameRunner {
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Games::new(HashMap::new()))
        .mount("/", routes![index])
        .mount("/", routes![prd_interface::game])
        .mount("/", routes![prd_interface::answer])
        .mount("/", routes![prd_interface::list_games])
        .attach(Template::fairing())
}
