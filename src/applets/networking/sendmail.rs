use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::os::unix::ffi::OsStrExt;

pub struct SendmailApplet;

impl Applet for SendmailApplet {
    fn name(&self) -> &'static str {
        "sendmail"
    }
    fn description(&self) -> &'static str {
        "Send mail via SMTP or pipe"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut server = "127.0.0.1:25".to_string();
        let mut from_addr = "root@localhost".to_string();
        let mut recipients = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-S" && i + 1 < args.len() {
                i += 1;
                server = String::from_utf8_lossy(args[i].as_bytes()).to_string();
                if !server.contains(':') {
                    server.push_str(":25");
                }
            } else if b == b"-f" && i + 1 < args.len() {
                i += 1;
                from_addr = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if !b.starts_with(b"-") {
                recipients.push(String::from_utf8_lossy(b).to_string());
            }
            i += 1;
        }

        if recipients.is_empty() {
            recipients.push("root@localhost".to_string());
        }

        let stream = match TcpStream::connect(&server) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sendmail: cannot connect to {}: {}", server, e);
                return Ok(1);
            }
        };

        let mut stream_write = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sendmail: {}", e);
                return Ok(1);
            }
        };
        let mut reader = BufReader::new(stream);

        fn smtp_cmd(
            r: &mut BufReader<TcpStream>,
            w: &mut TcpStream,
            cmd: &str,
        ) -> io::Result<String> {
            if !cmd.is_empty() {
                w.write_all(cmd.as_bytes())?;
                w.write_all(b"\r\n")?;
                w.flush()?;
            }
            let mut line = String::new();
            r.read_line(&mut line)?;
            Ok(line)
        }

        if let Err(e) = (|| -> io::Result<()> {
            smtp_cmd(&mut reader, &mut stream_write, "")?;
            smtp_cmd(&mut reader, &mut stream_write, "HELO localhost")?;
            smtp_cmd(
                &mut reader,
                &mut stream_write,
                &format!("MAIL FROM:<{}>", from_addr),
            )?;
            for rcpt in &recipients {
                smtp_cmd(
                    &mut reader,
                    &mut stream_write,
                    &format!("RCPT TO:<{}>", rcpt),
                )?;
            }
            smtp_cmd(&mut reader, &mut stream_write, "DATA")?;

            let stdin = io::stdin();
            let mut in_reader = BufReader::new(stdin.lock());
            let mut line = String::new();
            while in_reader.read_line(&mut line)? > 0 {
                if line.trim_end() == "." {
                    stream_write.write_all(b"..\r\n")?;
                } else {
                    stream_write.write_all(line.as_bytes())?;
                }
                line.clear();
            }
            smtp_cmd(&mut reader, &mut stream_write, "\r\n.")?;
            smtp_cmd(&mut reader, &mut stream_write, "QUIT")?;
            Ok(())
        })() {
            eprintln!("sendmail: error: {}", e);
            return Ok(1);
        }

        Ok(0)
    }
}
