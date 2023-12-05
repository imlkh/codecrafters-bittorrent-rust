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

use std::thread::sleep;
use std::time::Duration;

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

        eprintln!(
            "Peer ID: {}",
            message_recevied.to_handshake().peer_id_as_str()
        );

        Ok(())
    } else if command == "download_piece" {
        if args.len() > 2 && &args[2] != "-o" {
            // panic!("Not enough arguements")
            eprintln!("The second arguement has to be '-o': {}", args[2]);
            return Ok(());
        }
        if args.len() < 6 {
            eprintln!("Not enough arguements, length: {}", args.len());
            return Ok(());
        }
        let download_file_path = &args[3];
        let file_path = &args[4];

        eprintln!("file_path: {:?}", file_path);
        eprintln!("download_file_path: {:?}", download_file_path);

        let mut f = File::open(file_path)?;
        let mut buffer: Vec<u8> = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded_value = buffer.bdecode();
        let torrent = Torrent::new(&decoded_value)?;
        eprintln!(
            "total length: {}, piece length: {}",
            torrent.length, torrent.piece_length
        );
        eprintln!(
            "info_hash: {}, peer_id: {:?}",
            torrent.info_hash, torrent.peer_id
        );
        // query peer
        eprintln!("|||||||||||||| Query Peer |||||||||||||||||");
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

        let resp = reqwest::blocking::get(url_with_query)?;

        let body = resp.bytes()?;
        let decoded = body.to_vec().bdecode();
        // println!("Body:\n{}", decoded.to_string());

        let mut ip_addresses: Vec<String> = Vec::new();
        if let Value::Array(vec) = &decoded["peers"] {
            vec.iter().for_each(|map| {
                ip_addresses.push(format!("{}:{}", map["ip"].as_str().unwrap(), map["port"]))
            });
        }

        // handshake
        eprintln!("|||||||||||||| HandShake ||||||||||||||||||");
        let handshake = torrent.to_handshake();
        eprintln!(
            "info_hash: {}, peer_id: {:?}",
            handshake.info_hash, handshake.peer_id
        );
        let message = torrent.to_handshake().to_message();
        eprintln!("message.len(): {:?}", message.len());
        eprintln!("{:?}", message);
        // let ip_address = "178.62.82.89:51470"; // change it later
        // let ip_address = "165.232.33.77:51467"; // change it later
        let ip_address = {
            if ip_addresses.len() > 1 {
                &ip_addresses[1]
            } else {
                &ip_addresses[0]
            }
        };
        eprintln!("ip address: {}", ip_address);
        let mut message_recevied = vec![0u8; message.len()]; // initialize message buffer
        let mut stream = TcpStream::connect(ip_address)?;
        stream.write_all(&message)?;
        let mut message_size = 0;
        while message_size == 0 {
            message_size = stream
                .read(&mut message_recevied)
                .context("message read failed")?;
            eprintln!("the length of the received message is {message_size}");
            // eprintln!("{:?}", message_recevied);
            sleep(Duration::from_millis(100))
        }
        eprintln!(
            "Peer ID: {}",
            message_recevied.to_owned().to_handshake().peer_id_as_str()
        );

        eprintln!("|||||||||||||| Wait Messages ||||||||||||||");
        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        eprintln!("{:?}", &message_recevied[..message_size]);
        // println!("{}: {:?}", i, &message_recevied[..message_size]);
        // }
        //

        let peer_message = message_recevied
            .to_peer_message()
            .context("This is not a peer message")?;
        match &peer_message {
            PeerMessage::Bitfield(_) => eprintln!("[Bitfield]"),
            PeerMessage::Interested(_) => eprintln!("[Interested]"),
            PeerMessage::Unchoke(_) => eprintln!("[Unchoke]"),
            PeerMessage::Request(_) => eprintln!("[Request]"),
            PeerMessage::Piece(_) => eprintln!("[Piece]"),
        }
        // println!("Peer message type: {:?}", peer_message);

        if let PeerMessage::Bitfield(_) = peer_message {
            let message = PeerMessage::new(MessageType::Interested, 0, 0, 0).to_message();
            stream.write_all(&message)?;
        }

        let message_size = stream
            .read(&mut message_recevied)
            .context("message read failed")?;
        eprintln!("the length of the received message is {message_size}");
        eprintln!("{:?}", &message_recevied[..message_size]);

        let peer_message = message_recevied
            .to_peer_message()
            .context("This is not a peer message")?;
        match &peer_message {
            PeerMessage::Bitfield(_) => eprintln!("[Bitfield]"),
            PeerMessage::Interested(_) => eprintln!("[Interested]"),
            PeerMessage::Unchoke(_) => eprintln!("[Unchoke]"),
            PeerMessage::Request(_) => eprintln!("[Request]"),
            PeerMessage::Piece(_) => eprintln!("[Piece]"),
        }

        eprintln!("|||||||||||||| Request Data |||||||||||||||");
        let mut piece_received = Vec::<u8>::new();
        // request a piece
        if let PeerMessage::Unchoke(_) = peer_message {
            // let message = PeerMessage::new(6, 0, 0, usize::pow(2, 14)).to_message();
            let n_total = f64::ceil(torrent.length as f64 / (torrent.piece_length as f64)) as usize;
            // let n_total = 2;
            const BLOCK_CHUNK_SIZE: usize = usize::pow(2, 14);
            for index in 0..n_total {
                eprintln!("=== Pieces: {} of {}", index + 1, n_total);
                let piece_length = {
                    if torrent.length - piece_received.len() < torrent.piece_length {
                        torrent.length - piece_received.len()
                    } else {
                        torrent.piece_length
                    }
                };
                let n_blocks = f64::ceil(piece_length as f64 / BLOCK_CHUNK_SIZE as f64) as usize;
                for block_index in 0..n_blocks {
                    eprintln!("------ Blocks: {} of {}", block_index + 1, n_blocks);
                    let block_size = {
                        if block_index == n_blocks - 1 {
                            piece_length - BLOCK_CHUNK_SIZE * block_index
                        } else {
                            BLOCK_CHUNK_SIZE
                        }
                    };
                    let message = PeerMessage::new(
                        MessageType::Request,
                        index,
                        block_index * BLOCK_CHUNK_SIZE,
                        block_size,
                    )
                    .to_message();
                    eprintln!("{:?} message sent", message);
                    stream.write_all(&message)?;

                    let mut message_recevied = vec![0u8; block_size + 13]; // initialize message buffer
                    eprintln!("size: {}", message_recevied.len());
                    // let mut message_recevied = Vec::new(); // initialize message buffer
                    stream
                        .read_exact(&mut message_recevied)
                        .context("message read failed")?;
                    // eprintln!("the length of the received message is {message_size}");
                    // println!("{:?}", &message_recevied[..message_size]);
                    let peer_message = message_recevied
                        .to_peer_message()
                        .context("This is not a peer message")?;
                    match &peer_message {
                        PeerMessage::Bitfield(_) => println!("[Bitfield]"),
                        PeerMessage::Interested(_) => println!("[Interested]"),
                        PeerMessage::Unchoke(_) => println!("[Unchoke]"),
                        PeerMessage::Request(_) => println!("[Request]"),
                        PeerMessage::Piece(_) => println!("[Piece]"),
                    }
                    if let PeerMessage::Piece(message) = peer_message {
                        // println!("a piece is received");
                        // println!("{:?}", &message_recevied[..13]);
                        eprintln!("payload.len(): {}", message.payload.len());

                        let block = &message.payload[8..];
                        piece_received.extend_from_slice(block);
                        eprintln!(
                            "the current size of recieved pieces: {}",
                            piece_received.len()
                        );
                    }
                }
            }
        }

        if piece_received.len() == torrent.length {
            std::fs::write(download_file_path, piece_received)
                .context("something wrong with file saving")?;
            eprintln!("File saved completed, path: {}", download_file_path);
        } else {
            eprintln!(
                "Piece is not downaded completely, downloaded size: {}",
                piece_received.len()
            );
        }

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
