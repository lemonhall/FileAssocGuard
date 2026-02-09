mod captures;
mod rules;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!(
            "usage: fag <command> [args]\n\ncommands:\n  read --ext <.ext>\n  progids --ext <.ext>\n  latest --ext <.ext>\n  capture-latest --ext <.ext> --name <label>\n  apply-latest --ext <.ext> --name <label>\n  apply-latest --ext <.ext> --progid <ProgId> --hash <Hash>\n  captures --ext <.ext>\n  rules <list|add|remove> ...\n  check\n  watch-rules [--interval <seconds>]\n  watch --ext <.ext> --name <label> [--interval <seconds>]\n  restore --ext <.ext> (--progid <ProgId> | --to <vlc|potplayer>)"
        );
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
        "latest" => {
            let mut ext: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    _ => {}
                }
            }

            let Some(ext) = ext else {
                eprintln!("usage: fag latest --ext <.ext>");
                std::process::exit(2);
            };

            let effective = match fag_core::registry::effective_progid_for_ext(&ext) {
                Ok(Some(s)) => json_string(&s),
                Ok(None) => "null".into(),
                Err(err) => {
                    eprintln!("warning: effective progid query failed: {}", err);
                    "null".into()
                }
            };

            match fag_core::registry::read_user_choice_latest(&ext) {
                Ok(None) => {
                    println!(
                        "{{\"ext\":{},\"status\":\"NOT_SET\",\"prog_id\":null,\"hash\":null,\"last_write_time_filetime\":null,\"prog_id_last_write_time_filetime\":null,\"effective_progid\":{}}}",
                        json_string(&ext),
                        effective
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
                    let progid_last_write = uc
                        .prog_id_last_write_time
                        .map(|ft| ft.as_u64().to_string())
                        .map(|s| json_string(&s))
                        .unwrap_or("null".into());

                    println!(
                        "{{\"ext\":{},\"status\":\"OK\",\"prog_id\":{},\"hash\":{},\"last_write_time_filetime\":{},\"prog_id_last_write_time_filetime\":{},\"effective_progid\":{}}}",
                        json_string(&ext),
                        prog_id,
                        hash,
                        last_write,
                        progid_last_write,
                        effective
                    );
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("latest failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "capture-latest" => {
            let mut ext: Option<String> = None;
            let mut name: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    "--name" => name = args.next(),
                    _ => {}
                }
            }

            let Some(ext_raw) = ext else {
                eprintln!("usage: fag capture-latest --ext <.ext> --name <label>");
                std::process::exit(2);
            };
            let Some(name_raw) = name else {
                eprintln!("usage: fag capture-latest --ext <.ext> --name <label>");
                std::process::exit(2);
            };

            let ext = match normalize_ext_for_store(&ext_raw) {
                Ok(e) => e,
                Err(msg) => {
                    eprintln!("capture-latest failed: {}", msg);
                    std::process::exit(2);
                }
            };
            let name = name_raw.trim().to_ascii_lowercase();
            if name.is_empty() {
                eprintln!("capture-latest failed: --name is empty");
                std::process::exit(2);
            }

            match fag_core::registry::read_user_choice_latest(&ext) {
                Ok(Some(uc)) => {
                    let Some(prog_id) = uc.prog_id else {
                        eprintln!("capture-latest failed: ProgId missing in UserChoiceLatest");
                        std::process::exit(1);
                    };
                    let Some(hash) = uc.hash else {
                        eprintln!("capture-latest failed: Hash missing in UserChoiceLatest");
                        std::process::exit(1);
                    };

                    let cap = captures::LatestCapture {
                        prog_id: prog_id.clone(),
                        hash: hash.clone(),
                        last_write_time_filetime: uc.last_write_time.map(|ft| ft.as_u64()),
                        prog_id_last_write_time_filetime: uc
                            .prog_id_last_write_time
                            .map(|ft| ft.as_u64()),
                    };

                    let path = captures::default_store_path();
                    if let Err(err) = captures::upsert_latest_capture(&path, &ext, &name, cap) {
                        eprintln!("capture-latest failed: store write error: {}", err);
                        std::process::exit(1);
                    }

                    println!(
                        "{{\"ext\":{},\"name\":{},\"prog_id\":{},\"hash\":{},\"store_path\":{}}}",
                        json_string(&ext),
                        json_string(&name),
                        json_string(&prog_id),
                        json_string(&hash),
                        json_string(path.to_string_lossy().as_ref())
                    );
                    eprintln!("next: fag apply-latest --ext {} --name {}", ext, name);
                    std::process::exit(0);
                }
                Ok(None) => {
                    eprintln!(
                        "capture-latest failed: UserChoiceLatest not set for {}",
                        ext
                    );
                    std::process::exit(1);
                }
                Err(err) => {
                    eprintln!("capture-latest failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "apply-latest" => {
            let mut ext: Option<String> = None;
            let mut name: Option<String> = None;
            let mut progid: Option<String> = None;
            let mut hash: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    "--name" => name = args.next(),
                    "--progid" => progid = args.next(),
                    "--hash" => hash = args.next(),
                    _ => {}
                }
            }

            let Some(ext_raw) = ext else {
                eprintln!("usage: fag apply-latest --ext <.ext> --name <label>");
                eprintln!("   or: fag apply-latest --ext <.ext> --progid <ProgId> --hash <Hash>");
                std::process::exit(2);
            };
            let ext = match normalize_ext_for_store(&ext_raw) {
                Ok(e) => e,
                Err(msg) => {
                    eprintln!("apply-latest failed: {}", msg);
                    std::process::exit(2);
                }
            };

            let (progid, hash, source) = match (name, progid, hash) {
                (Some(n), None, None) => {
                    let label = n.trim().to_ascii_lowercase();
                    if label.is_empty() {
                        eprintln!("apply-latest failed: --name is empty");
                        std::process::exit(2);
                    }
                    let path = captures::default_store_path();
                    let cap = match captures::get_latest_capture(&path, &ext, &label) {
                        Ok(Some(c)) => c,
                        Ok(None) => {
                            eprintln!(
                                "apply-latest failed: no capture found for ext={} name={}. Run capture first: fag capture-latest --ext {} --name {}",
                                ext, label, ext, label
                            );
                            std::process::exit(1);
                        }
                        Err(err) => {
                            eprintln!("apply-latest failed: store read error: {}", err);
                            std::process::exit(1);
                        }
                    };
                    (cap.prog_id, cap.hash, format!("store:{}", label))
                }
                (None, Some(p), Some(h)) => (p, h, "inline".to_string()),
                _ => {
                    eprintln!("usage: fag apply-latest --ext <.ext> --name <label>");
                    eprintln!(
                        "   or: fag apply-latest --ext <.ext> --progid <ProgId> --hash <Hash>"
                    );
                    std::process::exit(2);
                }
            };

            match fag_core::registry::set_user_choice_latest_replay(&ext, &progid, &hash) {
                Ok(()) => {
                    let effective = match fag_core::registry::effective_progid_for_ext(&ext) {
                        Ok(Some(s)) => json_string(&s),
                        Ok(None) => "null".into(),
                        Err(err) => {
                            eprintln!("warning: effective progid query failed: {}", err);
                            "null".into()
                        }
                    };
                    println!(
                        "{{\"ext\":{},\"status\":\"APPLIED\",\"prog_id\":{},\"effective_progid\":{},\"source\":{}}}",
                        json_string(&ext),
                        json_string(&progid),
                        effective,
                        json_string(&source)
                    );
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("apply-latest failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "captures" => {
            let mut ext: Option<String> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    _ => {}
                }
            }

            let Some(ext_raw) = ext else {
                eprintln!("usage: fag captures --ext <.ext>");
                std::process::exit(2);
            };
            let ext = match normalize_ext_for_store(&ext_raw) {
                Ok(e) => e,
                Err(msg) => {
                    eprintln!("captures failed: {}", msg);
                    std::process::exit(2);
                }
            };

            let path = captures::default_store_path();
            let names = captures::list_capture_names(&path, &ext).unwrap_or_default();
            let joined = names
                .into_iter()
                .map(|s| json_string(&s))
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "{{\"ext\":{},\"names\":[{}],\"store_path\":{}}}",
                json_string(&ext),
                joined,
                json_string(path.to_string_lossy().as_ref())
            );
            std::process::exit(0);
        }
        "rules" => {
            let Some(action) = args.next() else {
                eprintln!("usage: fag rules <list|add|remove> ...");
                std::process::exit(2);
            };

            match action.as_str() {
                "list" => {
                    let path = rules::default_rules_path();
                    let items = rules::list_rules(&path).unwrap_or_default();
                    let joined = items
                        .into_iter()
                        .map(|(ext, name)| {
                            format!(
                                "{{\"ext\":{},\"name\":{}}}",
                                json_string(&ext),
                                json_string(&name)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    println!(
                        "{{\"rules\":[{}],\"rules_path\":{}}}",
                        joined,
                        json_string(path.to_string_lossy().as_ref())
                    );
                    std::process::exit(0);
                }
                "add" => {
                    let mut ext: Option<String> = None;
                    let mut name: Option<String> = None;
                    while let Some(arg) = args.next() {
                        match arg.as_str() {
                            "--ext" => ext = args.next(),
                            "--name" => name = args.next(),
                            _ => {}
                        }
                    }

                    let (Some(ext_raw), Some(name_raw)) = (ext, name) else {
                        eprintln!("usage: fag rules add --ext <.ext> --name <label>");
                        std::process::exit(2);
                    };
                    let ext = match normalize_ext_for_store(&ext_raw) {
                        Ok(e) => e,
                        Err(msg) => {
                            eprintln!("rules add failed: {}", msg);
                            std::process::exit(2);
                        }
                    };
                    let label = name_raw.trim().to_ascii_lowercase();
                    if label.is_empty() {
                        eprintln!("rules add failed: --name is empty");
                        std::process::exit(2);
                    }

                    let cap_path = captures::default_store_path();
                    let cap_ok = matches!(
                        captures::get_latest_capture(&cap_path, &ext, &label),
                        Ok(Some(_))
                    );
                    if !cap_ok {
                        eprintln!(
                            "rules add failed: capture missing for ext={} name={}. Run: fag capture-latest --ext {} --name {}",
                            ext, label, ext, label
                        );
                        std::process::exit(1);
                    }

                    let path = rules::default_rules_path();
                    if let Err(err) = rules::upsert_rule(&path, &ext, &label) {
                        eprintln!("rules add failed: store write error: {}", err);
                        std::process::exit(1);
                    }
                    println!(
                        "{{\"status\":\"ADDED\",\"ext\":{},\"name\":{},\"rules_path\":{}}}",
                        json_string(&ext),
                        json_string(&label),
                        json_string(path.to_string_lossy().as_ref())
                    );
                    std::process::exit(0);
                }
                "remove" => {
                    let mut ext: Option<String> = None;
                    while let Some(arg) = args.next() {
                        match arg.as_str() {
                            "--ext" => ext = args.next(),
                            _ => {}
                        }
                    }
                    let Some(ext_raw) = ext else {
                        eprintln!("usage: fag rules remove --ext <.ext>");
                        std::process::exit(2);
                    };
                    let ext = match normalize_ext_for_store(&ext_raw) {
                        Ok(e) => e,
                        Err(msg) => {
                            eprintln!("rules remove failed: {}", msg);
                            std::process::exit(2);
                        }
                    };
                    let path = rules::default_rules_path();
                    match rules::remove_rule(&path, &ext) {
                        Ok(true) => {
                            println!(
                                "{{\"status\":\"REMOVED\",\"ext\":{},\"rules_path\":{}}}",
                                json_string(&ext),
                                json_string(path.to_string_lossy().as_ref())
                            );
                            std::process::exit(0);
                        }
                        Ok(false) => {
                            eprintln!("rules remove: not found for {}", ext);
                            std::process::exit(2);
                        }
                        Err(err) => {
                            eprintln!("rules remove failed: store write error: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("usage: fag rules <list|add|remove> ...");
                    std::process::exit(2);
                }
            }
        }
        "check" => {
            let rules_path = rules::default_rules_path();
            let rules_items = match rules::list_rules(&rules_path) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("check failed: rules read error: {}", err);
                    std::process::exit(1);
                }
            };

            if rules_items.is_empty() {
                eprintln!(
                    "check: no rules found (add one with: fag rules add --ext .mp4 --name <label>)"
                );
                std::process::exit(2);
            }

            let cap_path = captures::default_store_path();
            let mut has_tampered = false;
            for (ext, label) in rules_items {
                let cap = match captures::get_latest_capture(&cap_path, &ext, &label) {
                    Ok(Some(c)) => c,
                    Ok(None) => {
                        eprintln!(
                            "check failed: capture missing for ext={} name={}. Re-capture: fag capture-latest --ext {} --name {}",
                            ext, label, ext, label
                        );
                        std::process::exit(1);
                    }
                    Err(err) => {
                        eprintln!("check failed: capture store read error: {}", err);
                        std::process::exit(1);
                    }
                };

                let effective = match fag_core::registry::effective_progid_for_ext(&ext) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("check failed: effective progid query failed: {}", err);
                        std::process::exit(1);
                    }
                };
                let ok = effective.as_deref() == Some(cap.prog_id.as_str());
                if !ok {
                    has_tampered = true;
                }
                println!(
                    "{{\"ext\":{},\"name\":{},\"status\":{},\"effective_progid\":{},\"target_progid\":{}}}",
                    json_string(&ext),
                    json_string(&label),
                    json_string(if ok { "OK" } else { "TAMPERED" }),
                    effective.map(|s| json_string(&s)).unwrap_or("null".into()),
                    json_string(&cap.prog_id)
                );
            }

            std::process::exit(if has_tampered { 2 } else { 0 });
        }
        "watch-rules" => {
            let mut interval_secs: u64 = 5;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--interval" => {
                        let Some(v) = args.next() else {
                            eprintln!("usage: fag watch-rules [--interval <seconds>]");
                            std::process::exit(2);
                        };
                        interval_secs = match v.parse::<u64>() {
                            Ok(n) if n > 0 => n,
                            _ => {
                                eprintln!("watch-rules failed: --interval must be a positive integer (seconds)");
                                std::process::exit(2);
                            }
                        };
                    }
                    _ => {}
                }
            }

            let rules_path = rules::default_rules_path();
            let cap_path = captures::default_store_path();
            eprintln!(
                "watch-rules interval={}s rules={} captures={} (Ctrl+C to stop)",
                interval_secs,
                rules_path.to_string_lossy(),
                cap_path.to_string_lossy()
            );

            let interval = std::time::Duration::from_secs(interval_secs);
            loop {
                let rules_items = rules::list_rules(&rules_path).unwrap_or_default();
                if rules_items.is_empty() {
                    eprintln!("watch-rules: no rules found");
                    std::thread::sleep(interval);
                    continue;
                }

                for (ext, label) in rules_items.iter() {
                    let cap = match captures::get_latest_capture(&cap_path, ext, label) {
                        Ok(Some(c)) => c,
                        _ => {
                            eprintln!(
                                "watch-rules: capture missing ext={} name={} (skip)",
                                ext, label
                            );
                            continue;
                        }
                    };

                    let effective = fag_core::registry::effective_progid_for_ext(ext)
                        .ok()
                        .flatten();
                    let ok = effective.as_deref() == Some(cap.prog_id.as_str());
                    if ok {
                        println!(
                            "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"OK\",\"effective_progid\":{},\"target_progid\":{}}}",
                            unix_time_ms(),
                            json_string(ext),
                            json_string(label),
                            effective.map(|s| json_string(&s)).unwrap_or("null".into()),
                            json_string(&cap.prog_id)
                        );
                        continue;
                    }

                    println!(
                        "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"TAMPERED\",\"effective_progid\":{},\"target_progid\":{}}}",
                        unix_time_ms(),
                        json_string(ext),
                        json_string(label),
                        effective.map(|s| json_string(&s)).unwrap_or("null".into()),
                        json_string(&cap.prog_id)
                    );

                    if let Err(err) = fag_core::registry::set_user_choice_latest_replay(
                        ext,
                        &cap.prog_id,
                        &cap.hash,
                    ) {
                        eprintln!(
                            "watch-rules apply failed ext={} name={}: {}",
                            ext, label, err
                        );
                        continue;
                    }

                    let after = fag_core::registry::effective_progid_for_ext(ext)
                        .ok()
                        .flatten();
                    println!(
                        "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"APPLIED\",\"effective_progid\":{},\"target_progid\":{}}}",
                        unix_time_ms(),
                        json_string(ext),
                        json_string(label),
                        after.map(|s| json_string(&s)).unwrap_or("null".into()),
                        json_string(&cap.prog_id)
                    );
                }

                std::thread::sleep(interval);
            }
        }
        "watch" => {
            let mut ext: Option<String> = None;
            let mut name: Option<String> = None;
            let mut interval_secs: u64 = 5;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    "--name" => name = args.next(),
                    "--interval" => {
                        let Some(v) = args.next() else {
                            eprintln!(
                                "usage: fag watch --ext <.ext> --name <label> [--interval <seconds>]"
                            );
                            std::process::exit(2);
                        };
                        interval_secs = match v.parse::<u64>() {
                            Ok(n) if n > 0 => n,
                            _ => {
                                eprintln!(
                                    "watch failed: --interval must be a positive integer (seconds)"
                                );
                                std::process::exit(2);
                            }
                        };
                    }
                    _ => {}
                }
            }

            let (Some(ext_raw), Some(name_raw)) = (ext, name) else {
                eprintln!("usage: fag watch --ext <.ext> --name <label> [--interval <seconds>]");
                std::process::exit(2);
            };

            let ext = match normalize_ext_for_store(&ext_raw) {
                Ok(e) => e,
                Err(msg) => {
                    eprintln!("watch failed: {}", msg);
                    std::process::exit(2);
                }
            };
            let label = name_raw.trim().to_ascii_lowercase();
            if label.is_empty() {
                eprintln!("watch failed: --name is empty");
                std::process::exit(2);
            }

            let path = captures::default_store_path();
            let cap = match captures::get_latest_capture(&path, &ext, &label) {
                Ok(Some(c)) => c,
                Ok(None) => {
                    eprintln!(
                        "watch failed: no capture found for ext={} name={}. Run: fag capture-latest --ext {} --name {}",
                        ext, label, ext, label
                    );
                    std::process::exit(1);
                }
                Err(err) => {
                    eprintln!("watch failed: store read error: {}", err);
                    std::process::exit(1);
                }
            };

            let target = cap.prog_id.clone();
            eprintln!(
                "watching ext={} target={} label={} interval={}s store={} (Ctrl+C to stop)",
                ext,
                target,
                label,
                interval_secs,
                path.to_string_lossy()
            );

            let interval = std::time::Duration::from_secs(interval_secs);
            loop {
                let now_ms = unix_time_ms();
                let effective = match fag_core::registry::effective_progid_for_ext(&ext) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("warning: effective progid query failed: {}", err);
                        None
                    }
                };

                let needs_fix = effective.as_deref() != Some(target.as_str());
                if !needs_fix {
                    println!(
                        "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"OK\",\"effective_progid\":{},\"target_progid\":{}}}",
                        now_ms,
                        json_string(&ext),
                        effective
                            .map(|s| json_string(&s))
                            .unwrap_or("null".into()),
                        json_string(&target)
                    );
                } else {
                    println!(
                        "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"TAMPERED\",\"effective_progid\":{},\"target_progid\":{}}}",
                        now_ms,
                        json_string(&ext),
                        effective
                            .map(|s| json_string(&s))
                            .unwrap_or("null".into()),
                        json_string(&target)
                    );

                    match fag_core::registry::set_user_choice_latest_replay(
                        &ext,
                        &cap.prog_id,
                        &cap.hash,
                    ) {
                        Ok(()) => {
                            let after = fag_core::registry::effective_progid_for_ext(&ext)
                                .ok()
                                .flatten();
                            println!(
                                "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"APPLIED\",\"effective_progid\":{},\"target_progid\":{}}}",
                                unix_time_ms(),
                                json_string(&ext),
                                after.map(|s| json_string(&s)).unwrap_or("null".into()),
                                json_string(&target)
                            );
                        }
                        Err(err) => {
                            eprintln!("watch apply failed: {}", err);
                        }
                    }
                }

                std::thread::sleep(interval);
            }
        }
        "restore" => {
            let mut ext: Option<String> = None;
            let mut progid: Option<String> = None;
            let mut to: Option<String> = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    "--progid" => progid = args.next(),
                    "--to" => to = args.next(),
                    _ => {}
                }
            }

            let Some(ext) = ext else {
                eprintln!(
                    "usage: fag restore --ext <.ext> (--progid <ProgId> | --to <vlc|potplayer>)"
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
                    eprintln!("usage: fag restore --ext <.ext> (--progid <ProgId> | --to <vlc|potplayer>)");
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
                    hash_version,
                }) => {
                    eprintln!(
                        "restore failed: UserChoiceLatest is enabled (HashVersion={}). Use the capture/replay workflow instead.",
                        hash_version
                    );
                    eprintln!("Steps:");
                    eprintln!(
                        "  1) Use Windows Settings to set the default app for {} once.",
                        ext
                    );
                    eprintln!(
                        "  2) Run: fag capture-latest --ext {} --name <vlc|potplayer>",
                        ext
                    );
                    eprintln!("  3) Later, restore with: fag apply-latest --ext {} --name <vlc|potplayer>", ext);
                    std::process::exit(1);
                }
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

fn normalize_ext_for_store(ext: &str) -> Result<String, String> {
    let ext = ext.trim();
    if ext.is_empty() || ext == "." {
        return Err("invalid extension".to_string());
    }
    let ext = ext.strip_prefix('.').unwrap_or(ext);
    if ext.is_empty() || ext.contains(['\\', '/', '\0']) {
        return Err("invalid extension".to_string());
    }
    Ok(format!(".{}", ext))
}

fn unix_time_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
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
