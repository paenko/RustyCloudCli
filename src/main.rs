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
  rustyc list
";

#[derive(Debug,RustcDecodable)]
struct Args {
    cmd_sync: bool,
    cmd_post: bool,
    cmd_get: bool,
    cmd_list: bool,
    cmd_delete: bool,
    arg_path: Option<String>,
    arg_id: Option<String>,
}

#[derive(Clone,RustcEncodable,RustcDecodable)]
struct DocFile {
    filename: String,
    file_id: Uuid,
    payload: String,
    lastEdited: DateTime<UTC>,
}

#[derive(Clone,RustcEncodable,RustcDecodable, Hash, Eq, PartialEq)]
struct TrackingFile {
    filename: String,
    file_id: Uuid,
    path: String,
    lastEdited: DateTime<UTC>,
}

impl TrackingFile {
    pub fn new(file_id: Uuid, filename: String, path: String, le: DateTime<UTC>) -> Self {
        TrackingFile {
            file_id: file_id,
            filename: filename,
            path: path,
            lastEdited: le,
        }
    }
}

impl DocFile {
    fn open(path: &Path) -> Self {
        let res: DocFile = decode_from(&mut File::open(&path).unwrap(), SizeLimit::Infinite)
            .unwrap();
        res
    }

    fn new(file_id: Uuid, path: &Path) -> Self
    {
        let mut fl: File = File::open(path).unwrap();
        let mut buf = Vec::new();
        fl.read_to_end(&mut buf);
        DocFile {
            filename: path.to_str().unwrap().to_string(),
            file_id: file_id,
            payload: base64::encode(&buf),
            lastEdited: UTC::now(),
        }
    }

    fn create(path: &Path) -> Self {
        let mut fl: File = File::open(path).unwrap();
        let mut buf = Vec::new();
        fl.read_to_end(&mut buf);
        DocFile {
            filename: path.to_str().unwrap().to_string(),
            file_id: Uuid::new_v4(),
            payload: base64::encode(&buf),
            lastEdited: UTC::now(),
        }
    }

    fn newSent(fln: String, fid: Uuid, py: Vec<u8>) -> Self {
        DocFile {
            filename: fln,
            file_id: fid,
            payload: base64::encode(&py),
            lastEdited: UTC::now(),
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

    let mut SyncedFiles: Vec<TrackingFile> =
        match decode_from(&mut File::open("Synced.syn").unwrap(), SizeLimit::Infinite) {
            Ok(e) => e,
            Err(err) => Vec::new(),
        };

    if (args.cmd_sync) {
        sync(args, &mut SyncedFiles);
    } else if (args.cmd_post) {
        post(args, &mut SyncedFiles);
    } else if (args.cmd_get) {
        getSave(&args);
    } else if (args.cmd_delete) {
        delete(args);
    } else if (args.cmd_list) {
        list(args, &mut SyncedFiles);
    }

    // Encode Synced
    let mut f = OpenOptions::new().write(true).create(true).open("Synced.syn").unwrap();
    encode_into(&SyncedFiles, &mut f, SizeLimit::Infinite);
}

fn list(mut args: Args, vs: &mut Vec<TrackingFile>)
{
    let mut unique : Vec<Uuid> = Vec::new();
    for x in vs.clone().into_iter()
    {
        let mut dup = false;
        for u in unique.clone().into_iter()
        {
            if(x.file_id == u) {dup = true;}
        }
        
        if(dup == false)
        {
            println!("name: {}  id: {}", x.filename, x.file_id);
            unique.push(x.file_id);
        }
    }
}

fn sync(mut args: Args, vs: &mut Vec<TrackingFile>) {

    let url = format!("http://127.0.0.1:8080/files");
    let st = RestClient::get(&url).unwrap().body;
    let Doc: Vec<DocFile> = json::decode(&st).unwrap();

    for d in Doc.iter()
    {
        //println!("{} {}", d.clone().file_id, d.clone().filename);
        vs.push(TrackingFile::new(d.clone().file_id, d.clone().filename,d.clone().filename, d.clone().lastEdited));
    }
    let mut lastTrack : TrackingFile = TrackingFile::new(Uuid::new_v4(), "und".to_string(), "und".to_string(), UTC::now());
    for x in vs.clone().into_iter() {
        // SYNCED aktuell?
        let res = match RestClient::post("http://127.0.0.1:8080/file/sync",
                                         &json::encode(&x).unwrap(),
                                         "application/json") {
            Ok(_) => {
                // update on server
                let args = Args {
                    arg_path: Some(x.path.to_string()),
                    cmd_sync: false,
                    cmd_post: true,
                    cmd_get: false,
                    cmd_list: false,
                    cmd_delete: false,
                    arg_id: Some(x.file_id.to_string()),
                };

                // TODO UPDATE HERE7
                if(Path::new(&x.path).exists())
                {
                    let got = get(&args);
                    let remotetime = got.clone().lastEdited;
                    if(lastTrack.filename == "und") { lastTrack.lastEdited = x.lastEdited; }
                    if(remotetime<lastTrack.lastEdited)
                    {
                        //lastTrack = x;
                        //post(args,  vs);
                        //println!("post {:?} {:?} {:?}", lastTrack.filename, lastTrack.lastEdited, String::from_utf8(base64::decode(&got.payload).unwrap()));
                    }
                    else
                    {
                        lastTrack = x;
                        //println!("get {:?} {:?} {:?}", lastTrack.filename, lastTrack.lastEdited, String::from_utf8(base64::decode(&got.payload).unwrap()));
                        let DF = get(&args);
                        fs::remove_file(DF.clone().filename);
                        let mut f = OpenOptions::new().write(true).create(true).open(DF.filename).unwrap();
                        let bytes = base64::decode(&DF.payload).unwrap();
                        f.write_all(bytes.as_slice());
                    }
                }
                else {
                
                let DF = get(&args);
                let mut f = OpenOptions::new().write(true).create(true).open(DF.filename).unwrap();
                let bytes = base64::decode(&DF.payload).unwrap();
                f.write_all(bytes.as_slice());
                //vs.puhs()
                }
                
            }
            Err(_) => {
                // Override local file
            }
        };
    }
}


fn post(mut args: Args,  vs: &mut Vec<TrackingFile>) {
    let rp = args.arg_path.unwrap().clone();
    let p = Path::new(&rp);
    let object = match args.arg_id {
        Some(i) =>  {
         DocFile::new(Uuid::parse_str(&i).unwrap(), &p)
    }
    None =>  {
         DocFile::create(&p)
    }
    };
    //println!("{}", object.payload.len());
    let res = RestClient::post("http://127.0.0.1:8080/file/push",
                               &json::encode(&object).unwrap(),
                               "application/json")
        .unwrap();
    
    let TF = TrackingFile::new(Uuid::parse_str(&res.body).unwrap(), p.file_name().unwrap().to_str().unwrap().to_string(), format!("{}", p.display()), UTC::now());
    vs.push(TF);
    
}

fn get(args: &Args) -> DocFile {
    let sid = args.arg_id.clone().unwrap();
    let url = format!("http://127.0.0.1:8080/files/{}", sid);
    let st = RestClient::get(&url).unwrap().body;
    // decode
    //println!("{}", st);
    json::decode(&st).unwrap()
}

fn getSave(args: &Args) {
    let DF = get(&args);
    let mut f = OpenOptions::new().write(true).create(true).open(DF.filename).unwrap();
    let bytes = base64::decode(&DF.payload).unwrap();
    f.write_all(bytes.as_slice());
}

fn delete(args: Args) {
    let sid = &args.arg_id.unwrap();
    let url = format!("http://127.0.0.1:8080/files/{}", sid);
    //println!("{}", RestClient::delete(&url).unwrap());
}

// File Handler
