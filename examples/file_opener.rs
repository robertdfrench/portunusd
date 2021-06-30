/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
use libc::chroot;
use portunusd::derive_server_procedure;
use std::str::from_utf8;
use std::fs::File;
use std::ffi::CString;
use std::io::Read;

fn content_type() -> String {
    "Content-Type: text/html; charset=UTF-8".to_owned()
}

fn content_length(msg: &str) -> String {
    let len = msg.to_owned().into_bytes().len();
    format!("Content-Length: {}", len)
}

fn response(msg: &str, status: &str) -> String {
    format!("HTTP/1.1 {}\n{}\n{}\n\n{}",
        status,
        content_type(),
        content_length(msg),
        msg
    )
}

fn four_hundred(msg: &str) -> String {
    response(msg, "400 Bad Request")
}

fn four_oh_four(msg: &str) -> String {
    response(msg, "404 Not Found")
}

fn two_hundred(msg: &str) -> String {
    response(msg, "200 OK")
}

fn try_to_read_file(uri: &str) -> std::io::Result<String> {
    let path = std::path::Path::new(uri);
    match path.is_dir() {
        true => try_to_read_file(&format!("{}/{}",path.to_str().unwrap(),"index.html")),
        false => match path.is_file() {
            false => Ok("Crab mind is blown.".to_owned()),
            true => {
                let mut f = File::open(uri)?;
                let mut contents = String::new();
                f.read_to_string(&mut contents)?;
                Ok(contents)
            }
        }
    }
}

fn open_file_from_http_request_uri(request: &str) -> String {
    match request.lines().nth(0) {
        None => four_hundred("Your http request was empty. Doin crab a concern. Go away!"),
        Some(request_line) => {
            let elements: Vec<&str> = request_line.split(' ').collect();
            if elements.len() != 3 {
                return four_hundred("Crab recommend you read RFC 2612.");
            }
            if elements[0] != "GET" {
                return four_hundred("Only can do a GET or a PINCH!!");
            }
            let path = elements[1];
            match try_to_read_file(path) {
                Err(e) => {
                    eprintln!("{}: {}", path, e);
                    four_oh_four("No such file. Go away! Will do a pinch.")
                },
                Ok(content) => two_hundred(&content)
            }
        }
    }
}

pub fn open_file(request: &[u8]) -> Vec<u8> {
    let response = match from_utf8(request) {
        Err(_) => four_hundred("You are trying to do crab a confuse. Go away!"),
        Ok(request) => open_file_from_http_request_uri(request),
    };
    response.into_bytes()
}

derive_server_procedure!(open_file as FileOpener);

fn main() {
    println!("Booting FileOpener Application");
    let file_opener = FileOpener::install("/var/run/file_opener.portunusd").unwrap();
    let docroot = CString::new("/opt/local/var/www").unwrap();
    match unsafe { chroot(docroot.as_c_str().as_ptr()) } {
        0 => {
            std::env::set_current_dir("/").unwrap();
            file_opener.park();
        },
        _ => {
            eprintln!("chroot failure");
        }
    }
}
