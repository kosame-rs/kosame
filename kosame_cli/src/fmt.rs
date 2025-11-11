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

        let regex = regex::Regex::new(concat!(
            "(",
            "table",
            "|",
            "statement",
            "|",
            "query",
            "|",
            "pg_table",
            "|",
            "pg_statement",
            "|",
            "pg_query",
            r")!\s*[\{\(\[]"
        ))
        .unwrap();

        let mut buffer = &input[..];
        let mut output = String::new();
        while let Some(r#match) = regex.find(buffer) {
            let start = r#match.end();
            output.push_str(&buffer[0..start]);
            let mut stack = 0;
            let mut end = buffer.len();
            for (i, c) in buffer[start..buffer.len()].char_indices() {
                match c {
                    '(' | '[' | '{' => stack += 1,
                    ')' | ']' | '}' => {
                        stack -= 1;
                        if stack < 0 {
                            end = i + start;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let formatting_input = &buffer[start..end];
            println!("{formatting_input}");
            output.push_str(formatting_input);

            buffer = &buffer[end..];
        }

        output.push_str(buffer);

        match &self.file {
            Some(file) => std::fs::write(file, output).unwrap(),
            None => print!("{}", output),
        };
    }
}
