use html_parser::Parser;
use layout::tree::LayoutTree;
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

    let html = std::fs::read_to_string(path.clone()).unwrap();
    let parser = Parser::new(arena, html.as_str());

    eprintln!("Started parsing '{}'", path);
    let before = Instant::now();
    let document = parser.parse();
    let after = Instant::now();
    let time = after.duration_since(before);
    eprintln!("Finished parsing document! Took {:?}!", time);
    eprintln!();

    eprintln!("---- DOM Tree ----");
    document.dump(Default::default());
    eprintln!();

    eprintln!("---- Layout Tree ----");
    let layout_tree = LayoutTree::from(&document);
    layout_tree.dump(Default::default());
}
