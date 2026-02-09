mod captures;
mod logging;
mod rules;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!(
            "usage: fag <command> [args]\n\ncommands:\n  read --ext <.ext>\n  progids --ext <.ext>\n  latest --ext <.ext>\n  capture-latest --ext <.ext> --name <label>\n  apply-latest --ext <.ext> --name <label>\n  apply-latest --ext <.ext> --progid <ProgId> --hash <Hash>\n  captures --ext <.ext>\n  rules <list|add|remove> ...\n  check\n  watch-rules [--interval <seconds>] [--monitor-only]\n  watch --ext <.ext> --name <label> [--interval <seconds>] [--monitor-only]\n  sysinfo\n  debug-legacy-hash --ext <.ext> --sid <SID> --progid <ProgId> --regdate-hex <16hex> [--experience <str>]\n  features <status|set> ...\n  win11 disable-userchoicelatest\n  restore --ext <.ext> (--progid <ProgId> | --to <vlc|potplayer>)"
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
                    let effective_raw = match fag_core::registry::effective_progid_for_ext(&ext) {
                        Ok(v) => v,
                        Err(err) => {
                            eprintln!("warning: effective progid query failed: {}", err);
                            None
                        }
                    };
                    let ok = effective_raw.as_deref() == Some(progid.as_str());
                    let effective = effective_raw
                        .as_deref()
                        .map(json_string)
                        .unwrap_or("null".into());
                    println!(
                        "{{\"ext\":{},\"status\":{},\"prog_id\":{},\"effective_progid\":{},\"source\":{},\"hint\":{}}}",
                        json_string(&ext),
                        json_string(if ok { "APPLIED" } else { "REJECTED" }),
                        json_string(&progid),
                        effective,
                        json_string(&source),
                        json_string(if ok {
                            ""
                        } else {
                            "系统可能拒绝/回滚了这次写入：请去 Windows 设置里手动改回默认程序；本工具会记录/提醒篡改事件。"
                        })
                    );
                    std::process::exit(if ok { 0 } else { 1 });
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
            let log_path = logging::default_log_path();
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
                let line = format!(
                    "{{\"ext\":{},\"name\":{},\"status\":{},\"effective_progid\":{},\"target_progid\":{}}}",
                    json_string(&ext),
                    json_string(&label),
                    json_string(if ok { "OK" } else { "TAMPERED" }),
                    effective.map(|s| json_string(&s)).unwrap_or("null".into()),
                    json_string(&cap.prog_id)
                );
                println!("{}", line);
                if !ok {
                    let _ = logging::append_line(&log_path, &line);
                }
            }

            std::process::exit(if has_tampered { 2 } else { 0 });
        }
        "watch-rules" => {
            let mut interval_secs: u64 = 5;
            let mut monitor_only: bool = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--interval" => {
                        let Some(v) = args.next() else {
                            eprintln!("usage: fag watch-rules [--interval <seconds>] [--monitor-only]");
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
                    "--monitor-only" => monitor_only = true,
                    _ => {}
                }
            }

            let rules_path = rules::default_rules_path();
            let cap_path = captures::default_store_path();
            let log_path = logging::default_log_path();
            eprintln!(
                "watch-rules interval={}s rules={} captures={} log={} (Ctrl+C to stop)",
                interval_secs,
                rules_path.to_string_lossy(),
                cap_path.to_string_lossy(),
                log_path.to_string_lossy()
            );

            #[derive(Debug, Copy, Clone)]
            struct BackoffState {
                failures: u32,
                next_allowed_ms: u128,
                manual_only: bool,
            }

            fn backoff_seconds(failures: u32) -> u64 {
                if failures == 0 {
                    return 0;
                }
                let shift = failures.saturating_sub(1).min(4);
                let secs = 30u64.saturating_mul(1u64 << shift);
                secs.min(600)
            }

            let mut backoff: std::collections::BTreeMap<String, BackoffState> =
                std::collections::BTreeMap::new();
            let mut last_emitted: std::collections::BTreeMap<String, (String, Option<String>)> =
                std::collections::BTreeMap::new();

            fn should_emit(
                last: &mut std::collections::BTreeMap<String, (String, Option<String>)>,
                key: &str,
                status: &str,
                effective: &Option<String>,
            ) -> bool {
                match last.get(key) {
                    Some((prev_status, prev_effective))
                        if prev_status == status && prev_effective == effective =>
                    {
                        false
                    }
                    _ => {
                        last.insert(key.to_string(), (status.to_string(), effective.clone()));
                        true
                    }
                }
            }

            let interval = std::time::Duration::from_secs(interval_secs);
            loop {
                let now_ms = unix_time_ms();
                let rules_items = rules::list_rules(&rules_path).unwrap_or_default();
                if rules_items.is_empty() {
                    eprintln!("watch-rules: no rules found");
                    std::thread::sleep(interval);
                    continue;
                }

                for (ext, label) in rules_items.iter() {
                    let key = format!("{}|{}", ext, label);
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

                    if let Some(st) = backoff.get(&key) {
                        if st.manual_only {
                            // still allow "ok" check below to clear state, but don't attempt apply.
                        } else if now_ms < st.next_allowed_ms {
                            continue;
                        }
                    }

                    let effective = fag_core::registry::effective_progid_for_ext(ext)
                        .ok()
                        .flatten();
                    let ok = effective.as_deref() == Some(cap.prog_id.as_str());
                    if ok {
                        backoff.remove(&key);
                        if should_emit(&mut last_emitted, &key, "OK", &effective) {
                            let line = format!(
                                "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"OK\",\"effective_progid\":{},\"target_progid\":{}}}",
                                unix_time_ms(),
                                json_string(ext),
                                json_string(label),
                                effective.map(|s| json_string(&s)).unwrap_or("null".into()),
                                json_string(&cap.prog_id)
                            );
                            println!("{}", line);
                        }
                        continue;
                    }

                    let st = backoff.get(&key).copied();
                    let manual_only_for_key = monitor_only || st.map(|s| s.manual_only).unwrap_or(false);

                    if should_emit(&mut last_emitted, &key, "TAMPERED", &effective) {
                        let line = format!(
                            "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"TAMPERED\",\"effective_progid\":{},\"target_progid\":{},\"mode\":{}}}",
                            unix_time_ms(),
                            json_string(ext),
                            json_string(label),
                            effective.map(|s| json_string(&s)).unwrap_or("null".into()),
                            json_string(&cap.prog_id),
                            json_string(if manual_only_for_key { "MONITOR_ONLY" } else { "AUTO_RESTORE" })
                        );
                        println!("{}", line);
                        let _ = logging::append_line(&log_path, &line);
                    }

                    if manual_only_for_key {
                        continue;
                    }

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
                    if after.as_deref() == Some(cap.prog_id.as_str()) {
                        backoff.remove(&key);
                        if should_emit(&mut last_emitted, &key, "APPLIED", &after) {
                            let line = format!(
                                "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"APPLIED\",\"effective_progid\":{},\"target_progid\":{}}}",
                                unix_time_ms(),
                                json_string(ext),
                                json_string(label),
                                after.map(|s| json_string(&s)).unwrap_or("null".into()),
                                json_string(&cap.prog_id)
                            );
                            println!("{}", line);
                            let _ = logging::append_line(&log_path, &line);
                        }
                    } else {
                        let failures = backoff.get(&key).map(|s| s.failures).unwrap_or(0) + 1;
                        let secs = backoff_seconds(failures);
                        backoff.insert(
                            key.clone(),
                            BackoffState {
                                failures,
                                next_allowed_ms: now_ms.saturating_add(u128::from(secs) * 1000),
                                manual_only: true,
                            },
                        );
                        if should_emit(&mut last_emitted, &key, "REJECTED", &after) {
                            let line = format!(
                                "{{\"time_unix_ms\":{},\"ext\":{},\"name\":{},\"status\":\"REJECTED\",\"effective_progid\":{},\"target_progid\":{},\"backoff_seconds\":{},\"next_mode\":\"MONITOR_ONLY\",\"hint\":\"系统拒绝/回滚了写入：后续改为只提示不自动改。建议去 Windows 设置里手动改回默认程序，然后再运行 fag capture-latest（可更新抓取）\"}}",
                                unix_time_ms(),
                                json_string(ext),
                                json_string(label),
                                after.map(|s| json_string(&s)).unwrap_or("null".into()),
                                json_string(&cap.prog_id),
                                secs
                            );
                            println!("{}", line);
                            let _ = logging::append_line(&log_path, &line);
                        }
                    }
                }

                std::thread::sleep(interval);
            }
        }
        "sysinfo" => match fag_core::sysinfo::read_sysinfo() {
            Ok(si) => {
                let sid = si.sid.as_deref().map(json_string).unwrap_or("null".into());
                let hash_version = si
                    .hash_version
                    .map(|v| v.to_string())
                    .unwrap_or("null".into());
                let ucpd_enabled = si
                    .ucpd_enabled
                    .map(|v| if v { "true" } else { "false" }.to_string())
                    .unwrap_or("null".into());
                let ucpd_driver_present = si
                    .ucpd_driver_present
                    .map(|v| if v { "true" } else { "false" }.to_string())
                    .unwrap_or("null".into());
                let guidance = si
                    .guidance
                    .into_iter()
                    .map(|s| json_string(&s))
                    .collect::<Vec<_>>()
                    .join(",");

                println!(
                    "{{\"sid\":{},\"hash_version\":{},\"user_choice_latest_enabled\":{},\"ucpd_enabled\":{},\"ucpd_driver_present\":{},\"guidance\":[{}]}}",
                    sid,
                    hash_version,
                    if si.user_choice_latest_enabled {
                        "true"
                    } else {
                        "false"
                    },
                    ucpd_enabled,
                    ucpd_driver_present,
                    guidance
                );
                std::process::exit(0);
            }
            Err(err) => {
                eprintln!("sysinfo failed: {}", err);
                std::process::exit(1);
            }
        },
        "debug-legacy-hash" => {
            let mut ext: Option<String> = None;
            let mut sid: Option<String> = None;
            let mut progid: Option<String> = None;
            let mut regdate_hex: Option<String> = None;
            let mut experience: Option<String> = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ext" => ext = args.next(),
                    "--sid" => sid = args.next(),
                    "--progid" => progid = args.next(),
                    "--regdate-hex" => regdate_hex = args.next(),
                    "--experience" => experience = args.next(),
                    _ => {}
                }
            }

            let (Some(ext), Some(sid), Some(prog_id), Some(regdate_hex)) =
                (ext, sid, progid, regdate_hex)
            else {
                eprintln!(
                    "usage: fag debug-legacy-hash --ext <.ext> --sid <SID> --progid <ProgId> --regdate-hex <16hex> [--experience <str>]"
                );
                std::process::exit(2);
            };

            let exp = experience.unwrap_or_else(|| fag_core::hash::USER_EXPERIENCE.to_string());
            let hash = fag_core::hash::compute_user_choice_hash_with_experience(
                &ext,
                &sid,
                &prog_id,
                &regdate_hex,
                &exp,
            );

            println!(
                "{{\"ext\":{},\"sid\":{},\"prog_id\":{},\"regdate_hex\":{},\"experience\":{},\"hash\":{}}}",
                json_string(&ext),
                json_string(&sid),
                json_string(&prog_id),
                json_string(&regdate_hex),
                json_string(&exp),
                json_string(&hash)
            );
            std::process::exit(0);
        },
        "features" => {
            let Some(sub) = args.next() else {
                eprintln!("usage: fag features <status|set> ...");
                std::process::exit(2);
            };

            match sub.as_str() {
                "status" => {
                    let mut id: Option<u32> = None;
                    let mut ty = fag_core::features::FeatureConfigurationType::Runtime;
                    while let Some(arg) = args.next() {
                        match arg.as_str() {
                            "--id" => {
                                let Some(v) = args.next() else {
                                    eprintln!("usage: fag features status --id <number> [--type <boot|runtime>]");
                                    std::process::exit(2);
                                };
                                id = v.parse::<u32>().ok();
                            }
                            "--type" => {
                                let Some(v) = args.next() else {
                                    eprintln!("usage: fag features status --id <number> [--type <boot|runtime>]");
                                    std::process::exit(2);
                                };
                                ty = match v.as_str() {
                                    "boot" => fag_core::features::FeatureConfigurationType::Boot,
                                    "runtime" => fag_core::features::FeatureConfigurationType::Runtime,
                                    _ => {
                                        eprintln!("features status failed: --type must be boot or runtime");
                                        std::process::exit(2);
                                    }
                                };
                            }
                            _ => {}
                        }
                    }

                    let Some(id) = id else {
                        eprintln!("usage: fag features status --id <number> [--type <boot|runtime>]");
                        std::process::exit(2);
                    };

                    match fag_core::features::query_feature_configuration(id, ty) {
                        Ok(cfg) => {
                            let state = match cfg.enabled_state {
                                fag_core::features::FeatureEnabledState::Default => "default",
                                fag_core::features::FeatureEnabledState::Disabled => "disabled",
                                fag_core::features::FeatureEnabledState::Enabled => "enabled",
                            };
                            let ty_str = match ty {
                                fag_core::features::FeatureConfigurationType::Boot => "boot",
                                fag_core::features::FeatureConfigurationType::Runtime => "runtime",
                            };
                            println!(
                                "{{\"id\":{},\"type\":{},\"enabled_state\":{},\"priority\":{},\"variant\":{},\"variant_payload_kind\":{},\"variant_payload\":{}}}",
                                id,
                                json_string(ty_str),
                                json_string(state),
                                cfg.priority,
                                cfg.variant,
                                cfg.variant_payload_kind,
                                cfg.variant_payload
                            );
                            std::process::exit(0);
                        }
                        Err(err) => {
                            eprintln!("features status failed: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
                "set" => {
                    let mut id: Option<u32> = None;
                    let mut ty = fag_core::features::FeatureConfigurationType::Boot;
                    let mut state: Option<fag_core::features::FeatureEnabledState> = None;
                    while let Some(arg) = args.next() {
                        match arg.as_str() {
                            "--id" => {
                                let Some(v) = args.next() else {
                                    eprintln!("usage: fag features set --id <number> --state <default|disabled|enabled> [--type <boot|runtime>]");
                                    std::process::exit(2);
                                };
                                id = v.parse::<u32>().ok();
                            }
                            "--type" => {
                                let Some(v) = args.next() else {
                                    eprintln!("usage: fag features set --id <number> --state <default|disabled|enabled> [--type <boot|runtime>]");
                                    std::process::exit(2);
                                };
                                ty = match v.as_str() {
                                    "boot" => fag_core::features::FeatureConfigurationType::Boot,
                                    "runtime" => fag_core::features::FeatureConfigurationType::Runtime,
                                    _ => {
                                        eprintln!("features set failed: --type must be boot or runtime");
                                        std::process::exit(2);
                                    }
                                };
                            }
                            "--state" => {
                                let Some(v) = args.next() else {
                                    eprintln!("usage: fag features set --id <number> --state <default|disabled|enabled> [--type <boot|runtime>]");
                                    std::process::exit(2);
                                };
                                state = Some(match v.as_str() {
                                    "default" => fag_core::features::FeatureEnabledState::Default,
                                    "disabled" => fag_core::features::FeatureEnabledState::Disabled,
                                    "enabled" => fag_core::features::FeatureEnabledState::Enabled,
                                    _ => {
                                        eprintln!("features set failed: --state must be default, disabled, or enabled");
                                        std::process::exit(2);
                                    }
                                });
                            }
                            _ => {}
                        }
                    }

                    let (Some(id), Some(state)) = (id, state) else {
                        eprintln!("usage: fag features set --id <number> --state <default|disabled|enabled> [--type <boot|runtime>]");
                        std::process::exit(2);
                    };

                    match fag_core::features::set_feature_state(id, ty, state) {
                        Ok(()) => {
                            let ty_str = match ty {
                                fag_core::features::FeatureConfigurationType::Boot => "boot",
                                fag_core::features::FeatureConfigurationType::Runtime => "runtime",
                            };
                            let state_str = match state {
                                fag_core::features::FeatureEnabledState::Default => "default",
                                fag_core::features::FeatureEnabledState::Disabled => "disabled",
                                fag_core::features::FeatureEnabledState::Enabled => "enabled",
                            };
                            println!(
                                "{{\"id\":{},\"type\":{},\"status\":\"OK\",\"state\":{},\"hint\":\"boot changes usually need a reboot\"}}",
                                id,
                                json_string(ty_str),
                                json_string(state_str)
                            );
                            std::process::exit(0);
                        }
                        Err(err) => {
                            eprintln!("features set failed: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("usage: fag features <status|set> ...");
                    std::process::exit(2);
                }
            }
        }
        "win11" => {
            let Some(sub) = args.next() else {
                eprintln!("usage: fag win11 disable-userchoicelatest");
                std::process::exit(2);
            };
            if sub.as_str() != "disable-userchoicelatest" {
                eprintln!("usage: fag win11 disable-userchoicelatest");
                std::process::exit(2);
            }

            let ids = [43229420u32, 27623730u32];
            let mut updates = Vec::new();
            let mut errors = Vec::new();
            for id in ids {
                let runtime_res = fag_core::features::set_feature_state(
                    id,
                    fag_core::features::FeatureConfigurationType::Runtime,
                    fag_core::features::FeatureEnabledState::Disabled,
                );
                let boot_res = fag_core::features::set_feature_state(
                    id,
                    fag_core::features::FeatureConfigurationType::Boot,
                    fag_core::features::FeatureEnabledState::Disabled,
                );

                let runtime_ok = runtime_res.is_ok();
                let boot_ok = boot_res.is_ok();
                if let Err(err) = runtime_res {
                    errors.push(format!(
                        "{{\"id\":{},\"type\":\"runtime\",\"error\":{}}}",
                        id,
                        json_string(&err.to_string())
                    ));
                }
                if let Err(err) = boot_res {
                    errors.push(format!(
                        "{{\"id\":{},\"type\":\"boot\",\"error\":{}}}",
                        id,
                        json_string(&err.to_string())
                    ));
                }

                updates.push(format!(
                    "{{\"id\":{},\"runtime_ok\":{},\"boot_ok\":{}}}",
                    id,
                    if runtime_ok { "true" } else { "false" },
                    if boot_ok { "true" } else { "false" }
                ));
            }

            println!(
                "{{\"status\":{},\"updates\":[{}],\"errors\":[{}],\"reboot_required\":true,\"hint\":\"after reboot, re-run sysinfo; if HashVersion became 0, legacy restore can work\"}}",
                json_string(if errors.is_empty() { "OK" } else { "ERROR" }),
                updates.join(","),
                errors.join(",")
            );
            if errors.is_empty() {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        },
        "watch" => {
            let mut ext: Option<String> = None;
            let mut name: Option<String> = None;
            let mut interval_secs: u64 = 5;
            let mut monitor_only: bool = false;

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
                    "--monitor-only" => monitor_only = true,
                    _ => {}
                }
            }

            let (Some(ext_raw), Some(name_raw)) = (ext, name) else {
                eprintln!("usage: fag watch --ext <.ext> --name <label> [--interval <seconds>] [--monitor-only]");
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
            let log_path = logging::default_log_path();
            eprintln!(
                "watching ext={} target={} label={} interval={}s store={} log={} (Ctrl+C to stop)",
                ext,
                target,
                label,
                interval_secs,
                path.to_string_lossy(),
                log_path.to_string_lossy()
            );

            let interval = std::time::Duration::from_secs(interval_secs);
            let mut failures: u32 = 0;
            let mut next_allowed_ms: u128 = 0;
            let mut manual_only = monitor_only;
            let mut last_emitted: Option<(String, Option<String>)> = None;
            loop {
                let now_ms = unix_time_ms();
                if !manual_only && next_allowed_ms != 0 && now_ms < next_allowed_ms {
                    std::thread::sleep(interval);
                    continue;
                }
                let effective = match fag_core::registry::effective_progid_for_ext(&ext) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("warning: effective progid query failed: {}", err);
                        None
                    }
                };

                let needs_fix = effective.as_deref() != Some(target.as_str());
                if !needs_fix {
                    failures = 0;
                    next_allowed_ms = 0;
                    manual_only = monitor_only;
                    let status = "OK".to_string();
                    if last_emitted.as_ref().map(|(s, e)| (s, e)) != Some((&status, &effective))
                    {
                        let line = format!(
                            "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"OK\",\"effective_progid\":{},\"target_progid\":{}}}",
                            now_ms,
                            json_string(&ext),
                            effective
                                .as_deref()
                                .map(json_string)
                                .unwrap_or("null".into()),
                            json_string(&target)
                        );
                        println!("{}", line);
                        last_emitted = Some((status, effective.clone()));
                    }
                } else {
                    let status = "TAMPERED".to_string();
                    if last_emitted.as_ref().map(|(s, e)| (s, e)) != Some((&status, &effective))
                    {
                        let line = format!(
                            "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"TAMPERED\",\"effective_progid\":{},\"target_progid\":{},\"mode\":{}}}",
                            now_ms,
                            json_string(&ext),
                            effective
                                .as_deref()
                                .map(json_string)
                                .unwrap_or("null".into()),
                            json_string(&target),
                            json_string(if manual_only { "MONITOR_ONLY" } else { "AUTO_RESTORE" })
                        );
                        println!("{}", line);
                        let _ = logging::append_line(&log_path, &line);
                        last_emitted = Some((status, effective.clone()));
                    }

                    if manual_only {
                        std::thread::sleep(interval);
                        continue;
                    }

                    match fag_core::registry::set_user_choice_latest_replay(
                        &ext,
                        &cap.prog_id,
                        &cap.hash,
                    ) {
                        Ok(()) => {
                            let after = fag_core::registry::effective_progid_for_ext(&ext)
                                .ok()
                                .flatten();
                            if after.as_deref() == Some(target.as_str()) {
                                failures = 0;
                                next_allowed_ms = 0;
                                let line = format!(
                                    "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"APPLIED\",\"effective_progid\":{},\"target_progid\":{}}}",
                                    unix_time_ms(),
                                    json_string(&ext),
                                    after.as_deref().map(json_string).unwrap_or("null".into()),
                                    json_string(&target)
                                );
                                println!("{}", line);
                                let _ = logging::append_line(&log_path, &line);
                                last_emitted = Some(("APPLIED".to_string(), after));
                            } else {
                                failures += 1;
                                let shift = failures.saturating_sub(1).min(4);
                                let secs = (30u64.saturating_mul(1u64 << shift)).min(600);
                                next_allowed_ms = now_ms.saturating_add(u128::from(secs) * 1000);
                                let line = format!(
                                    "{{\"time_unix_ms\":{},\"ext\":{},\"status\":\"REJECTED\",\"effective_progid\":{},\"target_progid\":{},\"backoff_seconds\":{},\"next_mode\":\"MONITOR_ONLY\",\"hint\":\"系统拒绝/回滚了写入：后续改为只提示不自动改。建议去 Windows 设置里手动改回默认程序，然后再运行 fag capture-latest（可更新抓取）\"}}",
                                    unix_time_ms(),
                                    json_string(&ext),
                                    after.as_deref().map(json_string).unwrap_or("null".into()),
                                    json_string(&target),
                                    secs
                                );
                                println!("{}", line);
                                let _ = logging::append_line(&log_path, &line);
                                manual_only = true;
                                last_emitted = Some(("REJECTED".to_string(), after));
                            }
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
