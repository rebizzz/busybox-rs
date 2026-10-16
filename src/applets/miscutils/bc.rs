use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStrExt;

pub struct BcApplet;
impl Applet for BcApplet {
    fn name(&self) -> &'static str {
        "bc"
    }
    fn description(&self) -> &'static str {
        "Arbitrary precision calculator language"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut exprs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-e" && i + 1 < args.len() {
                exprs.push(args[i + 1].to_string_lossy().to_string());
                i += 2;
                continue;
            }
            i += 1;
        }

        let eval_infix = |expr: &str| {
            if let Some(res) = eval_simple_math(expr) {
                println!("{}", res);
            }
        };

        if !exprs.is_empty() {
            for e in exprs {
                eval_infix(&e);
            }
        } else {
            let stdin = io::stdin();
            for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                let line = line.trim();
                if line == "quit" {
                    break;
                }
                if !line.is_empty() {
                    eval_infix(line);
                }
            }
        }

        Ok(0)
    }
}

fn eval_simple_math(expr: &str) -> Option<i64> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_ascii_digit() {
            let mut num = 0i64;
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num = num * 10 + (d as i64 - '0' as i64);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Num(num));
        } else {
            match c {
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '*' => tokens.push(Token::Star),
                '/' => tokens.push(Token::Slash),
                '%' => tokens.push(Token::Percent),
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                _ => {}
            }
            chars.next();
        }
    }

    #[derive(Debug, PartialEq)]
    enum Token {
        Num(i64),
        Plus,
        Minus,
        Star,
        Slash,
        Percent,
        LParen,
        RParen,
    }

    let mut output = Vec::new();
    let mut ops = Vec::new();

    let precedence = |t: &Token| match t {
        Token::Star | Token::Slash | Token::Percent => 2,
        Token::Plus | Token::Minus => 1,
        _ => 0,
    };

    for tok in tokens {
        match tok {
            Token::Num(n) => output.push(Token::Num(n)),
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent => {
                while let Some(top) = ops.last() {
                    if *top != Token::LParen && precedence(top) >= precedence(&tok) {
                        output.push(ops.pop().unwrap());
                    } else {
                        break;
                    }
                }
                ops.push(tok);
            }
            Token::LParen => ops.push(tok),
            Token::RParen => {
                while let Some(top) = ops.pop() {
                    if top == Token::LParen {
                        break;
                    }
                    output.push(top);
                }
            }
        }
    }
    while let Some(top) = ops.pop() {
        output.push(top);
    }

    let mut stack = Vec::new();
    for tok in output {
        match tok {
            Token::Num(n) => stack.push(n),
            Token::Plus => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(a + b);
            }
            Token::Minus => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(a - b);
            }
            Token::Star => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push(a * b);
            }
            Token::Slash => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                if b == 0 {
                    return None;
                }
                stack.push(a / b);
            }
            Token::Percent => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                if b == 0 {
                    return None;
                }
                stack.push(a % b);
            }
            _ => {}
        }
    }

    stack.pop()
}
