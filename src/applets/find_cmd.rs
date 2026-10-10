use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;
use std::process::Command;

pub struct FindApplet;

#[derive(Debug)]
struct ExecPlusBatch {
    cmd_template: Vec<String>,
    files: Vec<String>,
}

#[derive(Debug)]
enum Action {
    Name { pattern: String, case_fold: bool },
    Type { file_type: char },
    Print,
    Print0,
    Prune,
    Empty,
    Path { pattern: String, case_fold: bool },
    ExecSingle { cmd_args: Vec<String> },
    ExecBatch { batch_idx: usize },
    Ok { cmd_args: Vec<String> },
}

#[derive(Debug)]
struct ActionItem {
    action: Action,
    invert: bool,
}

struct FindContext<'a> {
    groups: &'a [Vec<ActionItem>],
    batches: &'a mut [ExecPlusBatch],
    need_print: bool,
    maxdepth: usize,
    mindepth: usize,
    depth_first: bool,
    follow_symlinks: bool,
    had_error: &'a mut bool,
}

fn fnmatch_match(pattern: &str, string: &str, case_fold: bool) -> bool {
    extern "C" {
        fn fnmatch(
            pattern: *const libc::c_char,
            string: *const libc::c_char,
            flags: libc::c_int,
        ) -> libc::c_int;
    }
    let Ok(c_pat) = CString::new(pattern.as_bytes()) else {
        return false;
    };
    let Ok(c_str) = CString::new(string.as_bytes()) else {
        return false;
    };
    let flags = if case_fold { 16 } else { 0 };
    unsafe { fnmatch(c_pat.as_ptr(), c_str.as_ptr(), flags) == 0 }
}

fn get_basename_for_name_match(path: &str) -> &str {
    let bytes = path.as_bytes();
    if bytes.is_empty() {
        return "";
    }
    if let Some(last_slash_pos) = path.rfind('/') {
        if last_slash_pos == bytes.len() - 1 {
            let mut end = bytes.len();
            while end > 0 && bytes[end - 1] == b'/' {
                end -= 1;
            }
            if end == 0 {
                "/"
            } else {
                let mut start = end;
                while start > 0 && bytes[start - 1] != b'/' {
                    start -= 1;
                }
                &path[start..end]
            }
        } else {
            &path[last_slash_pos + 1..]
        }
    } else {
        path
    }
}

fn evaluate_action(
    action: &Action,
    path: &str,
    metadata: &fs::Metadata,
    batches: &mut [ExecPlusBatch],
) -> (bool, bool) {
    match action {
        Action::Name { pattern, case_fold } => {
            let basename = get_basename_for_name_match(path);
            let matched = fnmatch_match(pattern, basename, *case_fold);
            (matched, false)
        }
        Action::Type { file_type } => {
            let ft = metadata.file_type();
            let matched = match *file_type {
                'f' => ft.is_file(),
                'd' => ft.is_dir(),
                'l' => ft.is_symlink(),
                'c' => ft.is_char_device(),
                'b' => ft.is_block_device(),
                'p' => ft.is_fifo(),
                's' => ft.is_socket(),
                _ => false,
            };
            (matched, false)
        }
        Action::Print => {
            println!("{}", path);
            (true, false)
        }
        Action::Print0 => {
            print!("{}\0", path);
            let _ = io::stdout().flush();
            (true, false)
        }
        Action::Prune => {
            let is_dir = metadata.is_dir() && !metadata.is_symlink();
            (true, is_dir)
        }
        Action::Empty => {
            let matched = if metadata.is_dir() && !metadata.is_symlink() {
                match fs::read_dir(path) {
                    Ok(mut it) => it.next().is_none(),
                    Err(_) => false,
                }
            } else if metadata.is_file() {
                metadata.len() == 0
            } else {
                false
            };
            (matched, false)
        }
        Action::Path { pattern, case_fold } => {
            let matched = fnmatch_match(pattern, path, *case_fold);
            (matched, false)
        }
        Action::ExecSingle { cmd_args } => {
            let substituted: Vec<String> =
                cmd_args.iter().map(|arg| arg.replace("{}", path)).collect();
            if substituted.is_empty() {
                (false, false)
            } else {
                let prog = &substituted[0];
                let args = &substituted[1..];
                let status = Command::new(prog).args(args).status();
                match status {
                    Ok(s) => (s.success(), false),
                    Err(e) => {
                        eprintln!("find: {}: {}", prog, e);
                        (false, false)
                    }
                }
            }
        }
        Action::ExecBatch { batch_idx } => {
            batches[*batch_idx].files.push(path.to_string());
            (true, false)
        }
        Action::Ok { cmd_args } => {
            let substituted: Vec<String> =
                cmd_args.iter().map(|arg| arg.replace("{}", path)).collect();
            if substituted.is_empty() {
                (false, false)
            } else {
                for arg in &substituted {
                    eprint!("{} ", arg);
                }
                eprint!("?");
                let _ = io::stderr().flush();

                let mut line = String::new();
                if io::stdin().read_line(&mut line).is_err() {
                    (false, false)
                } else {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with('y') || trimmed.starts_with('Y') {
                        let prog = &substituted[0];
                        let args = &substituted[1..];
                        let status = Command::new(prog).args(args).status();
                        match status {
                            Ok(s) => (s.success(), false),
                            Err(e) => {
                                eprintln!("find: {}: {}", prog, e);
                                (false, false)
                            }
                        }
                    } else {
                        (false, false)
                    }
                }
            }
        }
    }
}

fn recurse_dir(path: &str, depth: usize, root_dev: Option<u64>, ctx: &mut FindContext<'_>) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("find: '{path}': {err}");
            *ctx.had_error = true;
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("find: '{path}': {err}");
                *ctx.had_error = true;
                continue;
            }
        };

        let file_name = entry.file_name();
        let child_path = if path.ends_with('/') {
            format!("{}{}", path, file_name.to_string_lossy())
        } else {
            format!("{}/{}", path, file_name.to_string_lossy())
        };

        let child_meta = match if ctx.follow_symlinks {
            fs::metadata(&child_path)
        } else {
            fs::symlink_metadata(&child_path)
        } {
            Ok(m) => m,
            Err(err) => {
                eprintln!("find: '{child_path}': {err}");
                *ctx.had_error = true;
                continue;
            }
        };

        if let Some(rdev) = root_dev {
            if child_meta.dev() != rdev {
                // If on different filesystem, don't descend into it,
                // but evaluate the mount point itself if depth allows.
                if !ctx.depth_first && depth < ctx.maxdepth && depth + 1 >= ctx.mindepth {
                    let mut any_group_matched = false;
                    for group in ctx.groups {
                        let mut group_matched = true;
                        for item in group {
                            let (res, _prune) = evaluate_action(
                                &item.action,
                                &child_path,
                                &child_meta,
                                ctx.batches,
                            );
                            let effective_res = if item.invert { !res } else { res };
                            if !effective_res {
                                group_matched = false;
                                break;
                            }
                        }
                        if group_matched {
                            any_group_matched = true;
                            break;
                        }
                    }
                    if any_group_matched && ctx.need_print {
                        println!("{}", child_path);
                    }
                }
                continue;
            }
        }

        traverse(&child_path, depth + 1, &child_meta, root_dev, ctx);
    }
}

fn traverse(
    path: &str,
    depth: usize,
    metadata: &fs::Metadata,
    root_dev: Option<u64>,
    ctx: &mut FindContext<'_>,
) {
    if depth > ctx.maxdepth {
        return;
    }

    let is_dir = metadata.is_dir() && (!metadata.is_symlink() || ctx.follow_symlinks);

    let eval_actions = |ctx: &mut FindContext<'_>| -> (bool, bool) {
        if depth < ctx.mindepth {
            return (false, false);
        }
        let mut overall_prune = false;
        let mut any_group_matched = false;

        for group in ctx.groups {
            let mut group_matched = true;
            for item in group {
                let (res, prune) = evaluate_action(&item.action, path, metadata, ctx.batches);
                if prune {
                    overall_prune = true;
                }
                let effective_res = if item.invert { !res } else { res };
                if !effective_res {
                    group_matched = false;
                    break;
                }
            }
            if group_matched {
                any_group_matched = true;
                break;
            }
        }

        if any_group_matched && ctx.need_print {
            println!("{}", path);
        }

        (any_group_matched, overall_prune)
    };

    if !ctx.depth_first {
        let (_matched, prune) = eval_actions(ctx);

        if is_dir && depth < ctx.maxdepth && !prune {
            recurse_dir(path, depth, root_dev, ctx);
        }
    } else {
        if is_dir && depth < ctx.maxdepth {
            recurse_dir(path, depth, root_dev, ctx);
        }
        let _ = eval_actions(ctx);
    }
}

impl Applet for FindApplet {
    fn name(&self) -> &'static str {
        "find"
    }

    fn description(&self) -> &'static str {
        "Search for files in a directory hierarchy"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut root_paths: Vec<String> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s.starts_with('-') || s == "!" || s == "(" {
                break;
            }
            root_paths.push(s.into_owned());
            i += 1;
        }

        if root_paths.is_empty() {
            root_paths.push(".".to_string());
        }

        let mut groups: Vec<Vec<ActionItem>> = vec![Vec::new()];
        let mut batches: Vec<ExecPlusBatch> = Vec::new();
        let mut need_print = true;
        let mut maxdepth = usize::MAX;
        let mut mindepth = 0;
        let mut xdev = false;
        let mut depth_first = false;
        let mut follow_symlinks = false;
        let mut invert = false;

        while i < args.len() {
            let arg = args[i].to_string_lossy().into_owned();
            match arg.as_str() {
                "-follow" | "-L" => {
                    follow_symlinks = true;
                }
                "-H" => {}
                "-xdev" => {
                    xdev = true;
                }
                "-depth" => {
                    depth_first = true;
                }
                "-maxdepth" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -maxdepth");
                        return Ok(1);
                    }
                    let val = args[i].to_string_lossy();
                    match val.parse::<usize>() {
                        Ok(n) => maxdepth = n,
                        Err(_) => {
                            eprintln!("find: invalid argument '{}' to '-maxdepth'", val);
                            return Ok(1);
                        }
                    }
                }
                "-mindepth" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -mindepth");
                        return Ok(1);
                    }
                    let val = args[i].to_string_lossy();
                    match val.parse::<usize>() {
                        Ok(n) => mindepth = n,
                        Err(_) => {
                            eprintln!("find: invalid argument '{}' to '-mindepth'", val);
                            return Ok(1);
                        }
                    }
                }
                "-name" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -name");
                        return Ok(1);
                    }
                    let pattern = args[i].to_string_lossy().into_owned();
                    let action = Action::Name {
                        pattern,
                        case_fold: false,
                    };
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-iname" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -iname");
                        return Ok(1);
                    }
                    let pattern = args[i].to_string_lossy().into_owned();
                    let action = Action::Name {
                        pattern,
                        case_fold: true,
                    };
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-type" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -type");
                        return Ok(1);
                    }
                    let t_str = args[i].to_string_lossy();
                    let t_char = if t_str.len() == 1 {
                        t_str.chars().next().unwrap()
                    } else {
                        '?'
                    };
                    if !"fdlcbps".contains(t_char) {
                        eprintln!("find: unknown file type '{}'", t_str);
                        return Ok(1);
                    }
                    let action = Action::Type { file_type: t_char };
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-print" => {
                    need_print = false;
                    let action = Action::Print;
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-print0" => {
                    need_print = false;
                    let action = Action::Print0;
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-prune" => {
                    let action = Action::Prune;
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-empty" => {
                    let action = Action::Empty;
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-path" | "-wholename" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -path");
                        return Ok(1);
                    }
                    let pattern = args[i].to_string_lossy().into_owned();
                    let action = Action::Path {
                        pattern,
                        case_fold: false,
                    };
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-ipath" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("find: option requires an argument: -ipath");
                        return Ok(1);
                    }
                    let pattern = args[i].to_string_lossy().into_owned();
                    let action = Action::Path {
                        pattern,
                        case_fold: true,
                    };
                    groups
                        .last_mut()
                        .unwrap()
                        .push(ActionItem { action, invert });
                    invert = false;
                }
                "-exec" | "-ok" => {
                    need_print = false;
                    let is_ok = arg == "-ok";
                    i += 1;
                    let start = i;
                    let mut terminator: Option<&str> = None;
                    while i < args.len() {
                        let a = args[i].to_string_lossy();
                        if a == ";" {
                            terminator = Some(";");
                            break;
                        }
                        if !is_ok && a == "+" {
                            terminator = Some("+");
                            break;
                        }
                        i += 1;
                    }
                    if terminator.is_none() || start == i {
                        eprintln!("find: option requires an argument: {}", arg);
                        return Ok(1);
                    }
                    let term = terminator.unwrap();
                    let cmd_args: Vec<String> = (start..i)
                        .map(|idx| args[idx].to_string_lossy().into_owned())
                        .collect();
                    if term == "+" {
                        let batch_idx = batches.len();
                        batches.push(ExecPlusBatch {
                            cmd_template: cmd_args,
                            files: Vec::new(),
                        });
                        let action = Action::ExecBatch { batch_idx };
                        groups
                            .last_mut()
                            .unwrap()
                            .push(ActionItem { action, invert });
                    } else if is_ok {
                        let action = Action::Ok { cmd_args };
                        groups
                            .last_mut()
                            .unwrap()
                            .push(ActionItem { action, invert });
                    } else {
                        let action = Action::ExecSingle { cmd_args };
                        groups
                            .last_mut()
                            .unwrap()
                            .push(ActionItem { action, invert });
                    }
                    invert = false;
                }
                "-o" | "-or" => {
                    groups.push(Vec::new());
                    invert = false;
                }
                "-a" | "-and" => {}
                "!" | "-not" => {
                    invert = !invert;
                }
                _ => {
                    eprintln!("find: unrecognized: {}", arg);
                    return Ok(1);
                }
            }
            i += 1;
        }

        let mut had_error = false;

        for root_path in &root_paths {
            let p = Path::new(root_path);
            let root_meta = match if follow_symlinks {
                fs::metadata(p)
            } else {
                fs::symlink_metadata(p)
            } {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("find: '{root_path}': {e}");
                    had_error = true;
                    continue;
                }
            };

            let root_dev = if xdev { Some(root_meta.dev()) } else { None };

            let mut ctx = FindContext {
                groups: &groups,
                batches: &mut batches,
                need_print,
                maxdepth,
                mindepth,
                depth_first,
                follow_symlinks,
                had_error: &mut had_error,
            };

            traverse(root_path, 0, &root_meta, root_dev, &mut ctx);
        }

        // Flush batches for -exec ... +
        for batch in &batches {
            if batch.files.is_empty() {
                continue;
            }
            let prog = &batch.cmd_template[0];
            let mut final_args = Vec::new();
            let has_placeholder = batch.cmd_template[1..].iter().any(|a| a.contains("{}"));
            for arg in &batch.cmd_template[1..] {
                if arg.contains("{}") {
                    for file in &batch.files {
                        final_args.push(arg.replace("{}", file));
                    }
                } else {
                    final_args.push(arg.clone());
                }
            }
            if !has_placeholder {
                for file in &batch.files {
                    final_args.push(file.clone());
                }
            }

            let status = Command::new(prog).args(&final_args).status();
            match status {
                Ok(s) => {
                    if !s.success() {
                        had_error = true;
                    }
                }
                Err(e) => {
                    eprintln!("find: {}: {}", prog, e);
                    had_error = true;
                }
            }
        }

        if had_error {
            Ok(1)
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basename_matching() {
        assert_eq!(get_basename_for_name_match("/"), "/");
        assert_eq!(get_basename_for_name_match("//"), "/");
        assert_eq!(get_basename_for_name_match("///"), "/");
        assert_eq!(get_basename_for_name_match(".///"), ".");
        assert_eq!(get_basename_for_name_match("a/b"), "b");
        assert_eq!(get_basename_for_name_match("a/b/"), "b");
        assert_eq!(get_basename_for_name_match("a/b///"), "b");
        assert_eq!(get_basename_for_name_match("foo"), "foo");
        assert_eq!(get_basename_for_name_match(""), "");
    }

    #[test]
    fn test_fnmatch() {
        assert!(fnmatch_match("*", "anything", false));
        assert!(fnmatch_match("test*", "testfile", false));
        assert!(fnmatch_match("/", "/", false));
        assert!(!fnmatch_match("//", "/", false));
        assert!(fnmatch_match(".", ".", false));
        assert!(!fnmatch_match(".///", ".", false));
        assert!(fnmatch_match("[a-z]*", "foo", false));
        assert!(!fnmatch_match("[0-9]*", "foo", false));
    }
}
