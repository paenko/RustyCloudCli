extern crate docopt;
extern crate rustc_serialize;
extern crate uuid;
extern crate rest_client;

use std::path::Path;
use std::io::{Read, Write};
use uuid::Uuid;
use docopt::Docopt;
use rest_client::RestClient;
use rustc_serialize::json;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix;

const USAGE: &'static str = "
Rusty Cloud.

Usage:
  rustyc sync <path> <stop>
  rustyc get <id>
  rustyc delete <id>
";

#[derive(Debug,RustcDecodable)]
struct Args {
    cmd_sync: bool,
    arg_stop: Option<String>,
    cmd_get: bool,
    cmd_delete: bool,
    arg_path: Option<String>,
    arg_id: Option<u64>,
}

#[derive(Debug,RustcEncodable)]
struct DocFile {
    filename: String,
    fileId: Uuid,
    payload: Vec<u8>,
}

impl DocFile{
    fn new(path : String) -> Self
    {
      let mut f : File = File::open(&path).unwrap();
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer);
      DocFile
      {
        filename: path,
        fileId: Uuid::new_v4(),
        payload: buffer,
      }
    }

    fn newSent(fln : String, fid: Uuid, py: Vec<u8>) -> Self
    {
      DocFile
      {
        filename: fln,
        fileId: fid,
        payload: py,
      }
    }

    fn writeFile(&self) -> io::Result<()> {
      let mut f = try!(File::create(self.filename));
      f.write_all(s.as_bytes())
    }


}

fn main() {
    let args : Args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.decode())
                      .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    
    if(args.cmd_sync)
    {
      sync(args);
    }
    else if(args.cmd_get)
    {
      get(args);
    }
    else if(args.cmd_delete)
    {
      delete(args);
    }
}

fn sync(args: Args)
{
  let rp = &args.arg_path.unwrap();
  let p = Path::new(&rp);
  println!("{}", readAll(&p).unwrap());
  //println!("{}", RestClient::post("https://jsonplaceholder.typicode.com/posts",
  //                                  &json::encode(&object).unwrap(), 
  //                                  "application/json").unwrap());
}

fn get(args: Args)
{
    let st = RestClient::get("https://jsonplaceholder.typicode.com/posts").unwrap().body;
    //decode
    let ps = "Data.txt";
    let p = Path::new(&ps);
}

fn delete(args: Args)
{
    println!("{}", RestClient::delete("http://localhost:8081/file/1").unwrap());
}

//File Handler

fn readAll(path: &Path) -> io::Result<String> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}