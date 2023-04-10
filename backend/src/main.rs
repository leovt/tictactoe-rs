use std::collections::HashMap;
use std::sync::Mutex;
#[macro_use]
extern crate rocket;
use common::{Game, Message, Player};
use rocket::State;
use rocket_dyn_templates::{Template, context};

struct GameRunner {
    game: Game,
    messages: Vec<Message>,
}

impl GameRunner {
    fn new() -> GameRunner {
        GameRunner {
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

use rocket::response::content::RawHtml;
#[get("/games")]
fn list_games(games: &State<Games>) -> RawHtml<String> {
    let mut lines:Vec<String> = vec!["<html>".to_string(),"<body>".to_string(),"<ul>".to_string()];
    let games = games.lock().expect("lock shared data");
    for id in games.keys() {
        let game_uri = uri!(game(id));
        lines.push(format!("<li><a href=\"{}\">{}</a></li>", &game_uri, &id));
    }
    lines.extend(vec!["</ul>".to_string(), "</body>".to_string(),"</html>".to_string()]);
    RawHtml(lines.join("\r\n"))
}

#[get("/games/<id>")]
fn game(id: String, games: &State<Games>) -> String {
    let mut games = games.lock().expect("lock shared data");
    if games.get(&id).is_none() {
        games.insert(id.clone(), GameRunner::new());
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
