use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{BufRead, Read};

struct OptSpec {
    short: Vec<(u8, u8)>,
    long: Vec<(String, u8)>,
}

fn short_argmode(short: &[(u8, u8)], c: u8) -> Option<u8> {
    short.iter().find(|(k, _)| *k == c).map(|(_, m)| *m)
}

fn parse_short_spec(s: &[u8]) -> Vec<(u8, u8)> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] == b':' {
            i += 1;
            continue;
        }
        let c = s[i];
        let mut mode = 0;
        if i + 1 < s.len() && s[i + 1] == b':' {
            mode = 1;
            if i + 2 < s.len() && s[i + 2] == b':' {
                mode = 2;
                i += 1;
            }
            i += 1;
        }
        v.push((c, mode));
        i += 1;
    }
    v
}

fn parse_long_spec(s: &[u8]) -> Vec<(String, u8)> {
    let mut v = Vec::new();
    for part in String::from_utf8_lossy(s).split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, mode) = if let Some(n) = part.strip_suffix("::") {
            (n, 2)
        } else if let Some(n) = part.strip_suffix(':') {
            (n, 1)
        } else {
            (part, 0)
        };
        v.push((name.to_owned(), mode));
    }
    v
}

fn sh_quote(s: &[u8], out: &mut String) {
    out.push('\'');
    for &c in s {
        if c == b'\'' {
            out.push_str("'\\''");
        } else {
            out.push(c as char);
        }
    }
    out.push('\'');
}

pub struct GetoptApplet;
impl Applet for GetoptApplet {
    fn name(&self) -> &'static str {
        "getopt"
    }
    fn description(&self) -> &'static str {
        "Parse short+long options, print normalized quoted argv (util-linux subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut short_spec: Vec<u8> = Vec::new();
        let mut long_spec: Vec<u8> = Vec::new();
        let mut _name = "getopt".to_owned();
        let mut quiet = false;
        let mut test = false;
        let mut params: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"--" {
                i += 1;
                while i < args.len() {
                    params.push(&args[i]);
                    i += 1;
                }
                break;
            }
            if (b == b"-o" || b == b"--options") && i + 1 < args.len() {
                short_spec = ab(&args[i + 1]).to_vec();
                i += 2;
                continue;
            }
            if (b == b"-l" || b == b"--longoptions") && i + 1 < args.len() {
                long_spec = ab(&args[i + 1]).to_vec();
                i += 2;
                continue;
            }
            if (b == b"-n" || b == b"--name") && i + 1 < args.len() {
                _name = args[i + 1].to_string_lossy().into_owned();
                i += 2;
                continue;
            }
            if b == b"-q" || b == b"--quiet" {
                quiet = true;
                i += 1;
                continue;
            }
            if b == b"-T" || b == b"--test" {
                test = true;
                i += 1;
                continue;
            }
            if b == b"--options" || b.starts_with(b"--options=") {
                let v = if let Some(eq) = b.iter().position(|&c| c == b'=') {
                    b[eq + 1..].to_vec()
                } else {
                    i += 1;
                    if i < args.len() {
                        ab(&args[i]).to_vec()
                    } else {
                        Vec::new()
                    }
                };
                short_spec = v;
                i += 1;
                continue;
            }
            if b.starts_with(b"-")
                && b.len() > 1
                && !b.starts_with(b"--")
                && short_spec.is_empty()
                && long_spec.is_empty()
                && params.is_empty()
            {
                short_spec = b[1..].to_vec();
                i += 1;
                continue;
            }
            params.push(&args[i]);
            i += 1;
        }
        if test {
            return Ok(4);
        }

        let mut argv: Vec<&[u8]> = Vec::new();
        let mut operands: Vec<&[u8]> = Vec::new();
        let mut seen_dd = false;
        for p in &params {
            if !seen_dd && ab(p) == b"--" {
                seen_dd = true;
                continue;
            }
            if seen_dd {
                operands.push(ab(p));
            } else {
                argv.push(ab(p));
            }
        }
        let spec = OptSpec {
            short: parse_short_spec(&short_spec),
            long: parse_long_spec(&long_spec),
        };
        let mut out = String::new();
        let mut rest: Vec<&[u8]> = Vec::new();
        let mut j = 0;
        let mut bad = false;
        while j < argv.len() {
            let tok = argv[j];
            if tok == b"--" {
                rest.extend_from_slice(&argv[j + 1..]);
                break;
            }
            if tok.starts_with(b"--") && tok.len() > 2 {
                let body = &tok[2..];
                let (lname, val) = match body.iter().position(|&c| c == b'=') {
                    Some(eq) => (&body[..eq], Some(&body[eq + 1..])),
                    None => (body, None),
                };
                let lstr = String::from_utf8_lossy(lname).into_owned();
                let matches: Vec<usize> = spec
                    .long
                    .iter()
                    .enumerate()
                    .filter(|(_, (n, _))| *n == lstr || n.starts_with(lstr.as_str()))
                    .map(|(k, _)| k)
                    .collect();
                let exact = spec.long.iter().position(|(n, _)| *n == lstr);
                let idx = if let Some(e) = exact {
                    Some(e)
                } else if matches.len() == 1 {
                    Some(matches[0])
                } else {
                    None
                };
                match idx {
                    None => {
                        if !quiet {
                            eprintln!("getopt: unrecognized option '--{lstr}'");
                        }
                        bad = true;
                        j += 1;
                    }
                    Some(k) => {
                        let (lname2, mode) = spec.long[k].clone();
                        match (mode, val) {
                            (0, Some(_)) => {
                                if !quiet {
                                    eprintln!(
                                        "getopt: option '--{lname2}' doesn't allow an argument"
                                    );
                                }
                                bad = true;
                            }
                            (0, None) => {
                                out.push_str(" --");
                                out.push_str(&lname2);
                            }
                            (_, Some(v)) => {
                                out.push_str(" --");
                                out.push_str(&lname2);
                                out.push(' ');
                                sh_quote(v, &mut out);
                            }
                            (_, None) => {
                                if j + 1 < argv.len() {
                                    j += 1;
                                    out.push_str(" --");
                                    out.push_str(&lname2);
                                    out.push(' ');
                                    sh_quote(argv[j], &mut out);
                                } else if mode == 1 {
                                    if !quiet {
                                        eprintln!(
                                            "getopt: option '--{lname2}' requires an argument"
                                        );
                                    }
                                    bad = true;
                                } else {
                                    out.push_str(" --");
                                    out.push_str(&lname2);
                                }
                            }
                        }
                        j += 1;
                    }
                }
                continue;
            }
            if tok.len() > 1 && tok[0] == b'-' {
                let mut k = 1;
                while k < tok.len() {
                    let c = tok[k];
                    match short_argmode(&spec.short, c) {
                        None => {
                            if !quiet {
                                eprintln!("getopt: invalid option -- '{}'", c as char);
                            }
                            bad = true;
                            k += 1;
                        }
                        Some(0) => {
                            out.push_str(" -");
                            out.push(c as char);
                            k += 1;
                        }
                        Some(m) => {
                            let rest_tok = &tok[k + 1..];
                            if !rest_tok.is_empty() {
                                out.push_str(" -");
                                out.push(c as char);
                                out.push(' ');
                                sh_quote(rest_tok, &mut out);
                                k = tok.len();
                            } else if j + 1 < argv.len() {
                                j += 1;
                                out.push_str(" -");
                                out.push(c as char);
                                out.push(' ');
                                sh_quote(argv[j], &mut out);
                                k = tok.len();
                            } else if m == 1 {
                                if !quiet {
                                    eprintln!(
                                        "getopt: option requires an argument -- '{}'",
                                        c as char
                                    );
                                }
                                bad = true;
                                k = tok.len();
                            } else {
                                out.push_str(" -");
                                out.push(c as char);
                                k = tok.len();
                            }
                        }
                    }
                }
                j += 1;
                continue;
            }

            rest.extend_from_slice(&argv[j..]);
            break;
        }
        if bad {
            return Ok(1);
        }
        out.push_str(" --");
        for r in rest.iter().chain(operands.iter()) {
            out.push(' ');
            sh_quote(r, &mut out);
        }
        println!("{out}");
        Ok(0)
    }
}
