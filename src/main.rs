use std::env;

// use std::io;
use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
// use hex;
use serde_json::Value;
use std::io::Write;

#[allow(unused_imports)]
use std::net::{TcpStream, ToSocketAddrs};
// Available if you need it!
// use serde_bencode

use bittorrent_starter_rust::bencode::Bencode;
use bittorrent_starter_rust::torrent::*;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = encoded_value.bdecode();
        println!("{}", decoded_value);

        Ok(())
    } else if command == "info" {
        let file_path = &args[2];
        let mut f = File::open(file_path).context("could not open the info file")?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)
            .context("could not read the info file")?;
        let decoded_value = buffer.bdecode();
        println!("{}", Torrent::new(&decoded_value)?);

        Ok(())
    } else if command == "peers" {
        let file_path = &args[2];
        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded_value = buffer.bdecode();
        // println!("{}", decoded_value.to_string());
        let torrent = Torrent::new(&decoded_value)?;

        let left = torrent.piece_length.to_string();
        let query_params = vec![
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("compact", "1"),
            ("left", &left),
            ("peer_id", "00112233445566778899"),
            ("port", "6881"),
        ];

        let url_with_query = format!(
            "{}?{}&info_hash={}",
            torrent.url,
            serde_urlencoded::to_string(query_params)?,
            torrent.info_hash.to_url()
        );
        // println!("{}", url_with_query);

        // let resp = reqwest::get("http://bittorrent-test-tracker.codecrafters.io/announce?uploaded=0&downloaded=0&left=23768&peer_id=00112233445566778899&port=6881&info_hash=%d6%9f%91%e6%b2%ae%4c%54%24%68%d1%07%3a%71%d4%ea%13%87%9a%7f").await?;
        // let resp = reqwest::get(url_with_query).await?;
        let resp = reqwest::blocking::get(url_with_query)?;
        // println!("Status: {}", resp.status());
        // println!("Headers:\n{:#?}", resp.headers());

        let body = resp.bytes()?;
        let decoded = body.to_vec().bdecode();
        // println!("Body:\n{}", decoded.to_string());

        if let Value::Array(vec) = &decoded["peers"] {
            vec.iter()
                .for_each(|map| println!("{}:{}", map["ip"].as_str().unwrap(), map["port"]));
        }
        Ok(())
    } else if command == "handshake" {
        let file_path = &args[2];
        let ip_address = &args[3];
        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let message = {
            let decoded_value = buffer.bdecode();
            Torrent::new(&decoded_value)?.to_handshake().to_message()
        };

        let mut message_recevied = vec![0u8; message.len()]; // initialize message buffer
        let mut stream = TcpStream::connect(ip_address)?;
        stream.write_all(&message)?;
        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        // eprintln!("{:?}", message_recevied);

        println!(
            "Peer ID: {}",
            message_recevied.to_handshake().peer_id_as_str()
        );

        Ok(())
    } else if command == "download_piece" {
        if args.len() > 2 && &args[2] != "-o" {
            // panic!("Not enough arguements")
            println!("The second arguement has to be '-o': {}", args[2]);
            return Ok(());
        }
        if args.len() < 6 {
            println!("Not enough arguements, length: {}", args.len());
            return Ok(());
        }
        let download_file_path = &args[3];
        let file_path = &args[4];

        eprintln!("file_path: {:?}", file_path);
        eprintln!("download_file_path: {:?}", download_file_path);

        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let message = {
            let decoded_value = buffer.bdecode();
            Torrent::new(&decoded_value)?.to_handshake().to_message()
        };
        let ip_address = "178.62.82.89:51470";
        let mut message_recevied = vec![0u8; message.len()]; // initialize message buffer
        let mut stream = TcpStream::connect(ip_address)?;
        stream.write_all(&message)?;
        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        // eprintln!("{:?}", message_recevied);

        println!(
            "Peer ID: {}",
            message_recevied.to_owned().to_handshake().peer_id_as_str()
        );

        // for i in 0..3 {
        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        println!("{:?}", &message_recevied[..message_size]);
        // println!("{}: {:?}", i, &message_recevied[..message_size]);
        // }
        //
        if message_recevied[4] == 5u8 {
            let message = PeerMessage {
                length: [0, 0, 0, 1],
                id: 2,
                payload: Vec::<u8>::new(),
            }
            .to_message();
            stream.write_all(&message)?;
        }

        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        println!("{:?}", &message_recevied[..message_size]);

        // request a piece
        if message_recevied[4] == 1u8 {
            let index = [0u8, 0u8, 0u8, 0u8];
            let begin = [0u8, 0u8, 0u8, 0u8];
            let length = [0u8, 0u8, 64u8, 0u8];
            let mut payload = Vec::<u8>::new();
            payload.extend(&index);
            payload.extend(&begin);
            payload.extend(&length);

            let message = PeerMessage {
                length: [0, 0, 0, 1 + payload.len() as u8],
                id: 6,
                payload,
            }
            .to_message();
            println!("{:?} message sent", message);
            stream.write_all(&message)?;
        }

        let mut message_recevied = vec![0u8; 16384]; // initialize message buffer
        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        // println!("{:?}", &message_recevied[..message_size]);

        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        // println!("{:?}", &message_recevied[..message_size]);

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
