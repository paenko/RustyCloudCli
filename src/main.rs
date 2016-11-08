extern crate docopt;
extern crate rustc_serialize;
extern crate uuid;
extern crate rest_client;
extern crate bincode;
extern crate base64;
extern crate chrono;

use std::path::Path;
use std::io::BufReader;
use std::collections::BTreeMap;
use std::collections::{HashSet, HashMap};
use std::io::{Read, Write};
use uuid::Uuid;
use docopt::Docopt;
use rest_client::RestClient;
// use rustc_serialize::json;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix;
use chrono::*;

use rustc_serialize::Encodable;
use bincode::rustc_serialize::{encode_into, encode, decode, decode_from};
use bincode::SizeLimit;
use rustc_serialize::json::{self, ToJson, Json};

const USAGE: &'static str = "
Rusty Cloud.

Usage:
  rustyc sync
  rustyc post <path>
  rustyc get <id>
  rustyc delete <id>
";

#[derive(Debug,RustcDecodable)]
struct Args {
    cmd_sync: bool,
    cmd_post: bool,
    cmd_get: bool,
    cmd_delete: bool,
    arg_path: Option<String>,
    arg_id: Option<String>,
}

#[derive(RustcEncodable,RustcDecodable)]
struct DocFile {
    filename: String,
    file_id: Uuid,
    payload: String,
    lastEdited: DateTime<UTC>,
}

#[derive(Clone,RustcEncodable,RustcDecodable)]
struct TrackingFile {
    filename: String,
    file_id: Uuid,
    path: String,
    lastEdited: DateTime<UTC>,
}

impl TrackingFile {
    pub fn new(file_id: Uuid, filename: String, path: String) -> Self {
        TrackingFile {
            file_id: file_id,
            filename: filename,
            path: path,
            lastEdited: UTC::now(),
        }
    }
}

impl DocFile {
    fn open(path: &Path) -> Self {
        let res: DocFile = decode_from(&mut File::open(&path).unwrap(), SizeLimit::Infinite)
            .unwrap();
        res
    }

    fn create(path: &Path) -> Self {
        let mut fl: File = File::open(path).unwrap();
        let mut buf = Vec::new();
        fl.read_to_end(&mut buf);
        DocFile {
            filename: path.to_str().unwrap().to_string(),
            file_id: Uuid::new_v4(),
            payload: base64::encode(&buf),
            lastEdited: Local::now(),
        }
    }

    fn newSent(fln: String, fid: Uuid, py: Vec<u8>) -> Self {
        DocFile {
            filename: fln,
            file_id: fid,
            payload: base64::encode(&py),
            lastEdited: Local::now(),
        }

    }

    fn writeFile(&self) {
        let mut f = OpenOptions::new().write(true).create(true).open("Test").unwrap();
        encode_into(self, &mut f, SizeLimit::Infinite);
    }
}

fn main() {
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    let mut SyncedFiles: Vec<TrackingFile> =
        match decode_from(&mut File::open("Synced.syn").unwrap(), SizeLimit::Infinite) {
            Ok(e) => e,
            Err(err) => Vec::new(),
        };

    if (args.cmd_sync) {
        sync(args, SyncedFiles.clone());
    } else if (args.cmd_post) {
        post(args, SyncedFiles.clone());
    } else if (args.cmd_get) {
        get(args);
    } else if (args.cmd_delete) {
        delete(args);
    }

    let mut f = OpenOptions::new().write(true).create(true).open("Synced.syn").unwrap();

    encode_into(&SyncedFiles, &mut f, SizeLimit::Infinite);
    // Encode Synced
}

fn sync(mut args: Args, mut vs: Vec<TrackingFile>) {
    for x in vs.clone().into_iter() {
        // SYNCED aktuell?
        let res = match RestClient::post("http://127.0.0.1:8080/file/sync",
                                         &json::encode(x).unwrap(),
                                         "application/json") {
            Ok(_) => {
                // update on server
                let args = Args {
                    arg_path: Some(x.to_string()),
                    cmd_sync: false,
                    cmd_post: false,
                    cmd_get: false,
                    cmd_delete: false,
                    arg_id: x.file_id,
                };

                // TODO UPDATE HERE
            }
            Err(_) => {
                // Override local file
            }
        };
    }
}

fn post(mut args: Args) {
    let rp = args.arg_path.unwrap().clone();
    let p = Path::new(&rp);
    let object = DocFile::create(&p);
    println!("{}", object.payload.len());
    let res = RestClient::post("http://127.0.0.1:8080/file",
                               &json::encode(&object).unwrap(),
                               "application/json")
        .unwrap();
    println!("{}", res);

}

fn get(args: Args) {
    let sid = &args.arg_id.unwrap();
    let url = format!("http://127.0.0.1:8080/files/{}", sid);
    let st = RestClient::get(&url).unwrap().body;
    // decode
    let mut f = OpenOptions::new().write(true).create(true).open("Test").unwrap();
    encode_into(&st, &mut f, SizeLimit::Infinite);
}

fn delete(args: Args) {
    let sid = &args.arg_id.unwrap();
    let url = format!("http://127.0.0.1:8080/files/{}", sid);
    println!("{}", RestClient::delete(&url).unwrap());
}

// File Handler
