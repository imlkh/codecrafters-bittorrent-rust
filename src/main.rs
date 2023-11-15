use std::env;

// use std::io;
use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
use hex;
use serde_json::Value;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
// Available if you need it!
// use serde_bencode
pub mod bencode;

use crate::bencode::Bencode;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = encoded_value.bdecode();
        println!("{}", decoded_value.to_string());

        Ok(())
    } else if command == "info" {
        let file_path = &args[2];
        let mut f = File::open(file_path).context("could not open the info file")?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)
            .context("could not read the info file")?;
        let decoded_value = buffer.bdecode();
        // println!("{}", decoded_value.to_string());

        let map = decoded_value.as_object().context("this is not an object")?;
        let info = &map["info"];
        // println!("info\n{}", info.to_string());
        print!(
            "Tracker URL: {}",
            &map["announce"].as_str().context("this is not a string")?
        );
        println!();
        println!("Length: {}", info["length"]);
        println!(
            "Info Hash: {}",
            &map["info hash"].as_str().context("this is not a string")?
        );
        println!("Piece Length: {}", info["piece length"]);
        println!(
            "Piece Hashes:\n{}",
            &info["pieces"].as_str().context("this is not a string")?
        );

        Ok(())
    } else if command == "peers" {
        let file_path = &args[2];
        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded_value = buffer.bdecode();
        // println!("{}", decoded_value.to_string());

        let map = decoded_value.as_object().unwrap();
        let left = map["info"].as_object().unwrap()["piece length"]
            .as_i64()
            .unwrap()
            .to_string();
        let info_hash = map["info hash"].as_str().unwrap();
        let info_hash = info_hash
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            // .inspect(|arr| println!("%{}{}", arr[0], arr[1]))
            .map(|arr| format!("%{}{}", arr[0], arr[1]).to_string())
            .collect::<String>();

        let query_params = vec![
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("compact", "1"),
            ("left", &left),
            ("peer_id", "00112233445566778899"),
            ("port", "6881"),
        ];

        let url = &map["announce"].as_str().unwrap();
        let url_with_query = format!(
            "{}?{}&info_hash={}",
            url,
            serde_urlencoded::to_string(query_params)?,
            info_hash
        );
        // println!("{}", url_with_query);

        // let resp = reqwest::get("http://bittorrent-test-tracker.codecrafters.io/announce?uploaded=0&downloaded=0&left=23768&peer_id=00112233445566778899&port=6881&info_hash=%d6%9f%91%e6%b2%ae%4c%54%24%68%d1%07%3a%71%d4%ea%13%87%9a%7f").await?;
        // let resp = reqwest::get(url_with_query).await?;
        let resp = reqwest::blocking::get(url_with_query)?;
        // println!("Status: {}", resp.status());
        // println!("Headers:\n{:#?}", resp.headers());

        if false {
            // let body = resp.text().await?;
            let body = resp.text()?;
            println!("Body:\n{}", body.to_string());
            // println!("Body:\n{}", decoded.to_string());
        } else {
            // let body = resp.bytes().await?;
            let body = resp.bytes()?;
            let decoded = body.to_vec().bdecode();
            // println!("Body:\n{}", decoded.to_string());

            if let Value::Array(vec) = &decoded["peers"] {
                vec.iter().for_each(|map| {
                    println!(
                        "{}:{}",
                        map["ip"].as_str().unwrap(),
                        map["port"].to_string()
                    )
                });
            }
        }
        Ok(())
    } else if command == "handshake" {
        let file_path = &args[2];
        let ip_address = &args[3];
        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded_value = buffer.bdecode();

        let map = decoded_value.as_object().context("this is not an object")?;
        let info_hash = map["info hash"].as_str().context("this is not a string")?;
        let info_hash = hex::decode(info_hash).expect("Decoding failed");

        eprintln!("length : {}, {:?}", info_hash.len(), info_hash);

        // let mut stream = TcpStream::connect("178.62.82.89:51470")?;
        let mut stream = TcpStream::connect(ip_address)?;

        let mut message: Vec<u8> = Vec::new();
        message.push(19u8);
        message.extend_from_slice(b"BitTorrent protocol");
        message.extend_from_slice(&[0u8; 8]);
        message.extend_from_slice(&info_hash);
        message.extend_from_slice(b"00112233445566778899"); // peer id

        eprintln!("total length if the message : {}", message.len());
        let mut message_recevied = vec![0u8; message.len()];
        // stream.write(&message)?;
        stream.write_all(&message)?;
        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        // let message_size = stream.read_to_end(&mut message_recevied).context("message read failed")?;
        //
        eprintln!("the length of the received message is {message_size}");
        eprintln!("{:?}", message_recevied);

        let peer_id = &message_recevied[message_recevied.len() - 20..];
        let peer_id: String = peer_id.iter().map(|b| format!("{:02x}", b)).collect();
        println!("Peer ID: {}", peer_id);

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
