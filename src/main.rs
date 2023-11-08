use std::env;

use std::io;
use std::io::Read;
use std::fs::File;

use serde_json::Value;
// Available if you need it!
// use serde_bencode
pub mod bencode;

use crate::bencode::Bencode;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = encoded_value.bdecode();
        println!("{}", decoded_value.to_string());

        Ok(())
    } else if command == "info" {
        let file_path = &args[2];
        let mut f = File::open(file_path)?;
        let mut buffer : Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;

        let decoded_value = buffer.bdecode();
        // println!("{}", decoded_value.to_string());

        if let Value::Object(map) = decoded_value {
            let result = &map["announce"];
            let info = &map["info"];
            // println!("info\n{}", info.to_string());
            if let Value::String(str) = result {
                print!("Tracker URL: {}", str);
            }
            println!();
            println!("Length: {}", info["length"]);
            if let Value::String(str) = &map["info hash"] {
                println!("Info Hash: {}", str);
            }
            println!("Piece Length: {}", info["piece length"]);
            if let Value::String(str) = &info["pieces"] {
                println!("Piece Hashes:\n{}", str);
            }
        }

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
