fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: fag <command> [args]\n\ncommands:\n  read --ext <.ext>");
        std::process::exit(2);
    };

    match command.as_str() {
        "read" => {
            let mut ext: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    _ => {}
                }
            }

            let Some(ext) = ext else {
                eprintln!("usage: fag read --ext <.ext>");
                std::process::exit(2);
            };

            match fag_core::registry::read_user_choice(&ext) {
                Ok(None) => {
                    println!(
                        "{{\"ext\":{},\"status\":\"NOT_SET\",\"prog_id\":null,\"hash\":null,\"last_write_time_filetime\":null}}",
                        json_string(&ext)
                    );
                    std::process::exit(0);
                }
                Ok(Some(uc)) => {
                    let prog_id = uc.prog_id.map(|s| json_string(&s)).unwrap_or("null".into());
                    let hash = uc.hash.map(|s| json_string(&s)).unwrap_or("null".into());
                    let last_write = uc
                        .last_write_time
                        .map(|ft| ft.as_u64().to_string())
                        .map(|s| json_string(&s))
                        .unwrap_or("null".into());

                    println!(
                        "{{\"ext\":{},\"status\":\"OK\",\"prog_id\":{},\"hash\":{},\"last_write_time_filetime\":{}}}",
                        json_string(&ext),
                        prog_id,
                        hash,
                        last_write
                    );
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("read failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("unknown command: {}", command);
            std::process::exit(2);
        }
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

