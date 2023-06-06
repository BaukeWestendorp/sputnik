use parser::Parser;
use render_tree::RenderObject;
use std::time::Instant;
use typed_arena::Arena;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sputnik <path-to-html-file>");
        std::process::exit(1);
    }
    let path = args[1].clone();

    let arena = Arena::new();

    let html = std::fs::read_to_string(path).unwrap();
    let parser = Parser::new(arena, html.as_str());

    let before = Instant::now();
    let document = parser.parse();
    let after = Instant::now();
    let time = after.duration_since(before);

    document.dump(Default::default());

    let render_tree = RenderObject::from(document.clone());
    render_tree.dump();

    eprintln!("Took {:?} to parse document!", time);
}
