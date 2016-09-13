extern crate pencil;
extern crate dotenv;

use dotenv::dotenv;
use std::env;

use std::collections::BTreeMap;

use pencil::{Pencil, Request, Response, PencilResult};

fn hello(request: &mut Request) -> PencilResult {
    let mut context = BTreeMap::new();
    context.insert("name".to_string(), "template".to_string());
    return request.app.render_template("hello.html", &context);
}

fn main() {
    dotenv().ok();
    let mut app = Pencil::new(".");
    app.register_template("hello.html");
    app.get("/", "hello", hello);
    let binding = match env::var("GANBARE_SERVER_BINDING") {
        Err(_) => { println!("Specify the ip address and port to listen (e.g. 0.0.0.0:80) in envvar GANBARE_SERVER_BINDING!"); return; },
        Ok(ok) => ok,
    };
    println!("Ready to run at {}", binding);
    app.run(binding.as_str());
}
