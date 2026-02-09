mod setuserfta;

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
        "progids" => {
            let mut ext: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    _ => {}
                }
            }

            let Some(ext) = ext else {
                eprintln!("usage: fag progids --ext <.ext>");
                std::process::exit(2);
            };

            match fag_core::registry::list_open_with_progids(&ext) {
                Ok(progids) => {
                    let joined = progids
                        .into_iter()
                        .map(|s| json_string(&s))
                        .collect::<Vec<_>>()
                        .join(",");
                    println!("{{\"ext\":{},\"progids\":[{}]}}", json_string(&ext), joined);
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("progids failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "restore" => {
            let mut ext: Option<String> = None;
            let mut progid: Option<String> = None;
            let mut to: Option<String> = None;
            let mut setuserfta: Option<String> = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    "--progid" => progid = args.next(),
                    "--to" => to = args.next(),
                    "--setuserfta" => setuserfta = args.next(),
                    _ => {}
                }
            }

            let Some(ext) = ext else {
                eprintln!(
                    "usage: fag restore --ext <.ext> (--progid <ProgId> | --to <vlc|potplayer>) [--setuserfta <path>]"
                );
                std::process::exit(2);
            };

            let progid = match (progid, to) {
                (Some(p), None) => p,
                (None, Some(hint)) => match pick_progid_by_hint(&ext, &hint) {
                    Ok(p) => p,
                    Err(msg) => {
                        eprintln!("{}", msg);
                        std::process::exit(1);
                    }
                },
                _ => {
                    eprintln!("usage: fag restore --ext <.ext> (--progid <ProgId> | --to <vlc|potplayer>) [--setuserfta <path>]");
                    std::process::exit(2);
                }
            };

            match fag_core::registry::set_user_choice(&ext, &progid) {
                Ok(r) => {
                    println!(
                        "{{\"ext\":{},\"status\":\"RESTORED\",\"prog_id\":{},\"regdate_hex\":{},\"hash\":{},\"attempts\":{}}}",
                        json_string(&r.ext),
                        json_string(&r.prog_id),
                        json_string(&r.regdate_hex),
                        json_string(&r.hash),
                        r.attempts
                    );
                    std::process::exit(0);
                }
                Err(fag_core::registry::SetUserChoiceError::UserChoiceLatestEnabled {
                    hash_version: _,
                }) => match try_restore_via_setuserfta(&ext, &progid, setuserfta.as_deref()) {
                    Ok(()) => {
                        let verified = match fag_core::registry::read_user_choice(&ext) {
                            Ok(Some(uc)) => uc.prog_id.as_deref() == Some(progid.as_str()),
                            _ => false,
                        };
                        println!(
                            "{{\"ext\":{},\"status\":\"RESTORED_VIA_SETUSERFTA\",\"prog_id\":{},\"verified\":{}}}",
                            json_string(&ext),
                            json_string(&progid),
                            if verified { "true" } else { "false" }
                        );
                        std::process::exit(0);
                    }
                    Err(fallback_err) => {
                        eprintln!("restore failed: UserChoiceLatest is enabled");
                        eprintln!("fallback(SetUserFTA) failed: {}", fallback_err);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("restore failed: {}", err);
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

fn pick_progid_by_hint(ext: &str, hint: &str) -> Result<String, String> {
    let hint = hint.trim().to_ascii_lowercase();
    if hint.is_empty() {
        return Err("restore --to requires a non-empty hint".to_string());
    }

    let progids = fag_core::registry::list_open_with_progids(ext).map_err(|e| e.to_string())?;
    if progids.is_empty() {
        return Err(format!(
            "no ProgId candidates found for {} (try setting the default app once via UI, then rerun `fag progids --ext {}`)",
            ext, ext
        ));
    }

    if let Some(p) = progids
        .iter()
        .find(|p| p.to_ascii_lowercase().contains(&hint))
    {
        return Ok(p.clone());
    }

    let preview = progids.into_iter().take(30).collect::<Vec<_>>().join(", ");
    Err(format!(
        "no ProgId matched hint '{}'. candidates (first 30): {}. Use `fag restore --ext {} --progid <one-of-these>`",
        hint, preview, ext
    ))
}

fn try_restore_via_setuserfta(
    ext: &str,
    progid: &str,
    setuserfta_override: Option<&str>,
) -> Result<(), String> {
    let exe = setuserfta::find_setuserfta_exe(setuserfta_override).ok_or_else(|| {
        format!(
            "SetUserFTA.exe not found. Provide `--setuserfta <path>` or set env {}.",
            setuserfta::ENV_SETUSERFTA_EXE
        )
    })?;

    setuserfta::set_association(&exe, ext, progid).map_err(|e| e.to_string())
}
