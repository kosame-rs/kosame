mod fmt;

use clap::{Args, Parser};
use fmt::Fmt;

#[derive(Parser)]
#[command(
    name = "kosame",
    bin_name = "kosame",
    about = "Kosame: Macro-based Rust ORM focused on developer ergonomics"
)]
enum Root {
    Fmt(Fmt),
    // Introspect(Introspect),
}

#[derive(Args)]
#[command(version, about = "Introspects a database and generates a matching Kosame schema", long_about = None)]
struct Introspect {}

fn main() {
    let root = Root::parse_from(std::env::args().skip(1));
    let result = match root {
        Root::Fmt(inner) => inner.run(),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
