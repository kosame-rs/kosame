use std::io::Read;

use clap::Args;

#[derive(Args)]
#[command(version, about = "Format the content of Kosame macro invocations in Rust source files", long_about = None)]
pub struct Fmt {
    #[arg(short, long)]
    file: Option<std::path::PathBuf>,
}

impl Fmt {
    pub fn run(&self) {
        let input = match &self.file {
            Some(file) => std::fs::read_to_string(file).unwrap(),
            None => {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf).unwrap();
                buf
            }
        };

        let macro_patterns: Vec<String> = [
            "table",
            "pg_table",
            "statement",
            "pg_statement",
            "query",
            "pg_query",
        ]
        .iter()
        .map(|k| format!(r"({}!\s*[\[({{])", regex::escape(k))) // Wrap each in a capture group
        .collect();

        let pattern = format!("{}|([(){{}}\\[\\]])", macro_patterns.join("|"));
        let regex = regex::Regex::new(&pattern).unwrap();

        let mut output = String::new();

        struct Start {
            index: usize,
            indent: usize,
        }

        let mut indent = 0;
        let mut macro_start: Option<Start> = None;
        let mut last_end = 0;
        for r#match in regex.find_iter(&input) {
            match r#match.as_str() {
                "(" | "[" | "{" => indent += 1,
                ")" | "]" | "}" => {
                    indent -= 1;
                    if let Some(start) = macro_start.as_ref()
                        && start.indent >= indent
                    {
                        output.push_str(&input[last_end..start.index]);
                        let format_input = &input[start.index..r#match.start()];
                        println!("{}:{format_input}", start.indent);
                        output.push_str(format_input);
                        last_end = r#match.start();
                        macro_start = None;
                    }
                }
                _ => {
                    if macro_start.is_none() {
                        macro_start = Some(Start {
                            index: r#match.end(),
                            indent,
                        });
                    }
                    indent += 1;
                }
            }
        }
        output.push_str(&input[last_end..(input.len())]);

        match &self.file {
            Some(file) => std::fs::write(file, output).unwrap(),
            None => print!("{}", output),
        };
    }
}
