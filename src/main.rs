use std::fs::File;
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;

// external crates
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;
// Available if you need it!
// use serde_bencode
use bittorrent_starter_rust::bencode::Bencode;
use bittorrent_starter_rust::torrent::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[clap(rename_all = "snake_case")]
enum Commands {
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf,
    },
    Peers {
        torrent: PathBuf,
    },
    Handshake {
        torrent: PathBuf,
        peer: String,
    },
    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        piece: usize,
    },
    Download {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Decode { value } => {
            let encoded_value = value;
            let decoded_value = encoded_value.bdecode();
            println!("{}", decoded_value);
        }
        Commands::Info { torrent } => {
            let file_path = torrent;
            let mut f = File::open(file_path).context("could not open the info file")?;
            let mut buffer: Vec<u8> = Vec::new();
            f.read_to_end(&mut buffer)
                .context("could not read the info file")?;
            let decoded_value = buffer.bdecode();
            println!("{}", Torrent::new(&decoded_value)?);
        }
        Commands::Peers { torrent } => {
            let file_path = torrent;
            let mut f = File::open(file_path)?;
            let mut buffer: Vec<u8> = Vec::new();
            f.read_to_end(&mut buffer)?;
            let decoded_value = buffer.bdecode();
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

            let resp = reqwest::blocking::get(url_with_query)?;
            let body = resp.bytes()?;
            let decoded = body.to_vec().bdecode();

            if let Value::Array(vec) = &decoded["peers"] {
                vec.iter()
                    .for_each(|map| println!("{}:{}", map["ip"].as_str().unwrap(), map["port"]));
            }
        }
        Commands::Handshake { torrent, peer } => {
            let file_path = torrent;
            let ip_address = peer;
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

            println!(
                "Peer ID: {}",
                message_recevied.to_handshake().peer_id_as_str()
            );
        }
        Commands::DownloadPiece {
            output,
            torrent,
            piece,
        } => {
            eprintln!("file_path: {:?}", torrent);
            eprintln!("download_file_path: {:?}", output);
            eprintln!("piece_index: {:?}", piece);

            let mut f = File::open(torrent)?;
            let mut buffer: Vec<u8> = Vec::new();
            f.read_to_end(&mut buffer)?;
            let decoded_value = buffer.bdecode();
            let torrent = Torrent::new(&decoded_value)?;
            let piece_received = torrent.download(piece)?;
            std::fs::write(&output, piece_received).context("save downloaded piece into file")?;
            eprintln!("File saved completed, path: {}", output.display());
        }
        Commands::Download { output, torrent } => {
            eprintln!("file_path: {:?}", torrent);
            eprintln!("download_file_path: {:?}", output);

            let mut f = File::open(torrent)?;
            let mut buffer: Vec<u8> = Vec::new();
            f.read_to_end(&mut buffer)?;
            let decoded_value = buffer.bdecode();
            let torrent = Torrent::new(&decoded_value)?;
            let piece_received = torrent.download_all()?;
            std::fs::write(&output, piece_received).context("save downloaded piece into file")?;
            eprintln!("File saved completed, path: {}", output.display());
        }
    }

    Ok(())
}
