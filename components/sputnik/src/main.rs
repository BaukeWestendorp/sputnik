use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sputnik <path-to-html-file>");
        std::process::exit(1);
    }
    let path = args[1].clone();

    if path.ends_with(".html") {
        parse_html_file(&path);
        return;
    }

    if path.ends_with(".css") {
        parse_css_file(&path);
        return;
    }
}

fn parse_html_file(path: &str) {
    let arena = typed_arena::Arena::new();

    let html = std::fs::read_to_string(path.clone()).unwrap();
    let parser = html_parser::Parser::new(arena, html.as_str());

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
    let layout_tree = layout::tree::LayoutTree::from(&document);
    layout_tree.dump(Default::default());
}

fn parse_css_file(path: &str) {
    let css = std::fs::read_to_string(path.clone()).unwrap();
    let parser = css_parser::Parser::new(css.as_str());

    eprintln!("Started parsing '{}'", path);
    let before = Instant::now();
    parser.parse();
    let after = Instant::now();
    let time = after.duration_since(before);
    eprintln!("Finished parsing document! Took {:?}!", time);
    eprintln!();
}
