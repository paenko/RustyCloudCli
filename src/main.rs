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
//use rustc_serialize::json;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix;
use chrono::*;

use rustc_serialize::Encodable;
use bincode::rustc_serialize::{encode_into, encode, decode, decode_from};
use bincode::SizeLimit;
use rustc_serialize::json::{self,ToJson,Json};

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
    fileId: Uuid,
    payload: String,
    timeEdited: DateTime<Local>,
}

impl DocFile {
    fn open(path: &Path) -> Self {
        let res: DocFile = decode_from(&mut File::open(&path).unwrap(), SizeLimit::Infinite)
            .unwrap();
        res
    }

    fn create(path : &Path) -> Self
    {
      let mut fl : File = File::open(path).unwrap();
      let mut buf = Vec::new();
      fl.read_to_end(&mut buf);
      DocFile
      {
        filename: path.to_str().unwrap().to_string(),
        fileId: Uuid::new_v4(),
        payload: base64::encode(&buf),
        timeEdited: Local::now(),
      }
    }

    fn newSent(fln : String, fid: Uuid, py: Vec<u8>) -> Self
    {
      DocFile
      {
        filename: fln,
        fileId: fid,
        payload: base64::encode(&py),
        timeEdited: Local::now(),
      }

    }

    fn writeFile(&self) {
        let mut f = OpenOptions::new().write(true).create(true).open("Test").unwrap();
        encode_into(self, &mut f, SizeLimit::Infinite);
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    let mut SyncedFiles = Vec::new();
    SyncedFiles = decode_from(&mut File::open("Synced.syn").unwrap(), SizeLimit::Infinite).unwrap();
    if(args.cmd_sync)
    {
      sync(args);
    }
    else if(args.cmd_post)
    {
      post(args, &mut SyncedFiles);
    }
    else if(args.cmd_get)
    {
      get(args);
    }
    else if(args.cmd_delete)
    {
      delete(args);
    }
    
    let mut f =
            OpenOptions::new().write(true).create(true).open("Synced.syn").unwrap();

    encode_into(&SyncedFiles, &mut f, SizeLimit::Infinite);
    //Encode Synced
}

fn sync(args : Args)
{

}

fn post(args: Args, vs :&mut Vec<String>)
{
  let rp = args.arg_path.unwrap().clone();
  vs.push(rp.clone());
  let p = Path::new(&rp);
  let object = DocFile::create(&p);
  println!("{}", object.payload.len());
  let res = RestClient::post("http://127.0.0.1:8080/file",
                                    &json::encode(&object).unwrap(), 
                                    "application/json").unwrap();
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
