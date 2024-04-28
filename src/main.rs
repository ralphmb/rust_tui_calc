// TODO
// - Enforce a depth limit in parser()?
// - Remove all the clone()s
// - Make a struct to store the history?
// - - Will allow a better system for passing historic queries to input - it can return references to avoid cloning.
// - - and will be able to enforce maximum length
//
// - PARSER: try the function f(x) = (x-3)^4
// - - Parentheses aren't displayed
// - Resizable panes?
//
use num_parser;
use std::env;
use std::io;
mod app;
mod tui;
use app::App;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let result = num_parser::eval(&*args[1]);
        match result {
            Ok(res) => println!("{}", res),
            Err(msg) => println!("Error - {}", msg),
        }
        return Ok(());
    } else if args.len() > 2 {
        println!("Error - this tool accepts exactly one argument")
    }
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);

    tui::restore()?;
    app_result
}
