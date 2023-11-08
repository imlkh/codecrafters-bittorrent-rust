use std::env;

// use std::io;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use serde_json::Value;
// Available if you need it!
// use serde_bencode
pub mod bencode;

use crate::bencode::Bencode;

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded_value = buffer.bdecode();
        // println!("{}", decoded_value.to_string());

        let map = decoded_value.as_object().unwrap();
        let info = &map["info"];
        // println!("info\n{}", info.to_string());
        print!("Tracker URL: {}", &map["announce"].as_str().unwrap());
        println!();
        println!("Length: {}", info["length"]);
        println!("Info Hash: {}", &map["info hash"].as_str().unwrap());
        println!("Piece Length: {}", info["piece length"]);
        println!("Piece Hashes:\n{}", &info["pieces"].as_str().unwrap());

        Ok(())
    } else if command == "peers" {
        let file_path = &args[2];
        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded_value = buffer.bdecode();
        println!("{}", decoded_value.to_string());

        let map = decoded_value.as_object().unwrap();
        let left = map["info"].as_object().unwrap()["piece length"].as_i64().unwrap().to_string();
        let info_hash = map["info hash"].as_str().unwrap();
        let info_hash = info_hash
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            // .inspect(|arr| println!("%{}{}", arr[0], arr[1]))
            .map(|arr| format!("%{}{}", arr[0], arr[1]).to_string())
            .collect::<String>();

        // let info_hash = "%d6%9f%91%e6%b2%ae%4c%54%24%68%d1%07%3a%71%d4%ea%13%87%9a%7f";
        println!("{}", info_hash);

        let query_params = vec!(
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("compact", "1"),
            ("left", &left),
            ("peer_id", "00112233445566778899"),
            ("port", "6881"),
        );

        let url = &map["announce"].as_str().unwrap();
        let url_with_query = format!("{}?{}&info_hash={}", url, serde_urlencoded::to_string(query_params).unwrap(), info_hash);
        // println!("{}", url_with_query);

        // let resp = reqwest::get("http://bittorrent-test-tracker.codecrafters.io/announce?uploaded=0&downloaded=0&left=23768&peer_id=00112233445566778899&port=6881&info_hash=%d6%9f%91%e6%b2%ae%4c%54%24%68%d1%07%3a%71%d4%ea%13%87%9a%7f").await?;
        let resp = reqwest::get(url_with_query).await?;
        // println!("Status: {}", resp.status());
        // println!("Headers:\n{:#?}", resp.headers());

        // let body = resp.text().await?;
        // println!("Body:\n{}", body.to_string());
        // println!("Body:\n{}", decoded.to_string());

        let body = resp.bytes().await?;
        // body.iter().for_each(|b| eprintln!("{},", *b as char));
        let decoded = body.to_vec().bdecode();
        // println!("Body:\n{}", decoded.to_string());

        // println!("{}", decoded["peers"].to_string());
        if let Value::Array(vec) = &decoded["peers"] {
            vec.iter().for_each(|map| println!("{}:{}", map["ip"].as_str().unwrap(), map["port"].to_string()));
        }
        // println!("{:#?}", resp);

        // println!("finished!!");

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
