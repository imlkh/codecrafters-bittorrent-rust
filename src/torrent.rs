use crate::bencode::Bencode;
use anyhow::Context;
use serde_json::Value;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;

pub struct HandShake {
    // length: u8,
    // kind: [u8; 19],
    // reserved: [u8; 8],
    pub info_hash: InfoHash,
    pub peer_id: Vec<u8>,
}
impl HandShake {
    pub fn to_message(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.push(19u8);
        vec.extend_from_slice(b"BitTorrent protocol");
        vec.extend_from_slice(&[0u8; 8]);
        vec.extend_from_slice(&self.info_hash.to_hex());
        vec.extend_from_slice(&self.peer_id); // peer id
        vec
    }
    pub fn peer_id_as_str(&self) -> String {
        self.peer_id.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02x}");
            output
        })
    }
}

pub trait ToHandShake {
    fn to_handshake(&self) -> HandShake;
}

impl ToHandShake for Torrent {
    fn to_handshake(&self) -> HandShake {
        HandShake {
            info_hash: InfoHash {
                val: (&self.info_hash.val[..]).into(),
            },
            peer_id: (&self.peer_id[..]).into(),
        }
    }
}

impl ToHandShake for Vec<u8> {
    fn to_handshake(&self) -> HandShake {
        HandShake {
            info_hash: InfoHash {
                val: hex::encode(&self[self.len() - 40..self.len() - 20]),
            },
            peer_id: self[self.len() - 20..].to_vec(),
        }
    }
}

pub struct InfoHash {
    val: String,
}

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl InfoHash {
    pub fn to_url(&self) -> String {
        self.val
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            // .inspect(|arr| println!("%{}{}", arr[0], arr[1]))
            .map(|arr| format!("%{}{}", arr[0], arr[1]).to_string())
            .collect::<String>()
    }
    pub fn to_hex(&self) -> Vec<u8> {
        hex::decode(&(self.val)).expect("to be hexadecimal")
    }
}

pub struct Torrent {
    pub url: String,
    pub length: usize,
    pub info_hash: InfoHash,
    pub piece_length: usize,
    pub piece_hashes: String,
    pub peer_id: Vec<u8>,
}

impl Torrent {
    pub fn new(decoded_value: &Value) -> anyhow::Result<Torrent> {
        let map = decoded_value.as_object().context("read map object")?;

        Ok(Torrent {
            url: map["announce"].as_str().context("read url")?.to_string(),
            length: map["info"]["length"].as_i64().context("read length")? as usize,
            info_hash: InfoHash {
                val: map["info hash"]
                    .as_str()
                    .context("read info hash")?
                    .to_string(),
            },
            piece_length: map["info"]["piece length"]
                .as_i64()
                .context("read piece length")? as usize,
            piece_hashes: map["info"]["pieces"]
                .as_str()
                .context("read peiece hashes")?
                .to_string(),
            peer_id: b"00112233445566778899".to_vec(),
        })
    }
    pub fn download(&self, piece_index: usize) -> anyhow::Result<Vec<u8>> {
        eprintln!(
            "total length: {}, piece length: {}",
            self.length, self.piece_length
        );
        eprintln!("info_hash: {}, peer_id: {:?}", self.info_hash, self.peer_id);
        eprintln!("|||||||||||||| Query Peer |||||||||||||||||");
        let left = self.piece_length.to_string();
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
            self.url,
            serde_urlencoded::to_string(query_params)?,
            self.info_hash.to_url()
        );

        let resp = reqwest::blocking::get(url_with_query)?;

        let body = resp.bytes()?;
        let decoded = body.to_vec().bdecode();

        let mut ip_addresses: Vec<String> = Vec::new();
        if let Value::Array(vec) = &decoded["peers"] {
            vec.iter().for_each(|map| {
                ip_addresses.push(format!("{}:{}", map["ip"].as_str().unwrap(), map["port"]))
            });
        }

        // handshake
        eprintln!("|||||||||||||| HandShake ||||||||||||||||||");
        let message = self.to_handshake().to_message();
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
        // eprintln!("{:?}", &message_recevied[..message_size]);

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
        // eprintln!("{:?}", &message_recevied[..message_size]);

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
            let n_total = f64::ceil(self.length as f64 / (self.piece_length as f64)) as usize;
            // let n_total = 1;
            const BLOCK_CHUNK_SIZE: usize = usize::pow(2, 14);
            // for index in 0..n_total {
            eprintln!("=== Pieces: {} of {}", piece_index + 1, n_total);
            let piece_length = {
                if piece_index == n_total - 1 {
                    self.length - piece_index * self.piece_length
                } else {
                    self.piece_length
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
                    piece_index,
                    block_index * BLOCK_CHUNK_SIZE,
                    block_size,
                )
                .to_message();
                // eprintln!("{:?} message sent", message);
                stream.write_all(&message)?;

                let mut message_recevied = vec![0u8; block_size + 13]; // initialize message buffer
                                                                       // eprintln!("size: {}", message_recevied.len());
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
                    let block = &message.payload[8..];
                    piece_received.extend_from_slice(block);
                    eprintln!(
                        "the current size of recieved pieces: {}",
                        piece_received.len()
                    );
                }
            }
        }
        Ok(piece_received)
    }
    pub fn download_all(&self) -> anyhow::Result<Vec<u8>> {
        eprintln!(
            "total length: {}, piece length: {}",
            self.length, self.piece_length
        );
        eprintln!("info_hash: {}, peer_id: {:?}", self.info_hash, self.peer_id);
        // query peer
        eprintln!("|||||||||||||| Query Peer |||||||||||||||||");
        let left = self.piece_length.to_string();
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
            self.url,
            serde_urlencoded::to_string(query_params)?,
            self.info_hash.to_url()
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
        // let handshake = torrent.to_handshake();
        // eprintln!(
        //     "info_hash: {}, peer_id: {:?}",
        //     handshake.info_hash, handshake.peer_id
        // );
        let message = self.to_handshake().to_message();
        // eprintln!("message.len(): {:?}", message.len());
        // eprintln!("{:?}", message);
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
            // eprintln!("the length of the received message is {message_size}");
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
        // eprintln!("{:?}", &message_recevied[..message_size]);

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
        // eprintln!("{:?}", &message_recevied[..message_size]);

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
            let n_total = f64::ceil(self.length as f64 / (self.piece_length as f64)) as usize;
            // let n_total = 1;
            const BLOCK_CHUNK_SIZE: usize = usize::pow(2, 14);
            for piece_index in 0..n_total {
                eprintln!("=== Pieces: {} of {}", piece_index + 1, n_total);
                let piece_length = {
                    if piece_index == n_total - 1 {
                        self.length - piece_index * self.piece_length
                    } else {
                        self.piece_length
                    }
                };
                let n_blocks = f64::ceil(piece_length as f64 / BLOCK_CHUNK_SIZE as f64) as usize;
                for block_index in 0..n_blocks {
                    let block_size = {
                        if block_index == n_blocks - 1 {
                            piece_length - BLOCK_CHUNK_SIZE * block_index
                        } else {
                            BLOCK_CHUNK_SIZE
                        }
                    };
                    let message = PeerMessage::new(
                        MessageType::Request,
                        piece_index,
                        block_index * BLOCK_CHUNK_SIZE,
                        block_size,
                    )
                    .to_message();
                    // eprintln!("{:?} message sent", message);
                    stream.write_all(&message)?;

                    let mut message_recevied = vec![0u8; block_size + 13]; // initialize message buffer
                                                                           // let mut message_recevied = Vec::new(); // initialize message buffer
                    stream
                        .read_exact(&mut message_recevied)
                        .context("message read failed")?;
                    let peer_message = message_recevied
                        .to_peer_message()
                        .context("This is not a peer message")?;
                    if let PeerMessage::Piece(message) = peer_message {
                        let block = &message.payload[8..];
                        piece_received.extend_from_slice(block);
                        eprintln!(
                            "Blocks: {} of {}, downloaded size: {}",
                            block_index + 1,
                            n_blocks,
                            piece_received.len()
                        );
                    }
                }
            }
        }
        Ok(piece_received)
    }
}

impl fmt::Display for Torrent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Tracker URL: {}\nLength: {}\nInfo Hash: {}\nPiece Length: {}\nPiece Hashes:\n{}",
            &self.url, &self.length, &self.info_hash, &self.piece_length, &self.piece_hashes
        )
    }
}
pub enum PeerMessage {
    Bitfield(Message),
    Interested(Message),
    Unchoke(Message),
    Request(Message),
    Piece(Message),
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum MessageType {
    Bitfield = 5,
    Interested = 2,
    Unchoke = 1,
    Request = 6,
    Piece = 7,
}

pub trait ToPeerMessage {
    fn to_peer_message(&self) -> anyhow::Result<PeerMessage>;
}

impl From<MessageType> for u8 {
    fn from(m: MessageType) -> u8 {
        m as u8
    }
}

impl ToPeerMessage for Vec<u8> {
    fn to_peer_message(&self) -> anyhow::Result<PeerMessage> {
        if self.len() < 5 {
            panic!("length is too short");
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&self[..4]);
        // let message_length = ((self[0] as i32) << 24)
        //     + ((self[1] as i32) << 16)
        //     + ((self[2] as i32) << 8)
        //     + self[3] as i32;

        let message_length = u32::from_be_bytes(length_bytes) as usize;
        // eprintln!("len: {}, message_lenth: {}", self.len(), message_length);
        if self.len() - 4 < message_length {
            panic!("message is not completely parsed or wrong message");
        }

        let message = Message {
            length: [self[0], self[1], self[2], self[3]],
            payload: self[5..(5 + message_length - 1)].into(),
        };

        let message_type = if self[4] == MessageType::Bitfield as u8 {
            MessageType::Bitfield
        } else if self[4] == MessageType::Interested as u8 {
            MessageType::Interested
        } else if self[4] == MessageType::Unchoke as u8 {
            MessageType::Unchoke
        } else if self[4] == MessageType::Request as u8 {
            MessageType::Request
        } else if self[4] == MessageType::Piece as u8 {
            MessageType::Piece
        } else {
            panic!("It's not a PeerMessage");
        };

        match message_type {
            MessageType::Bitfield => Ok(PeerMessage::Bitfield(message)),
            MessageType::Interested => Ok(PeerMessage::Interested(message)),
            MessageType::Unchoke => Ok(PeerMessage::Unchoke(message)),
            MessageType::Request => Ok(PeerMessage::Request(message)),
            MessageType::Piece => Ok(PeerMessage::Piece(message)),
        }
    }
}

pub struct Message {
    pub length: [u8; 4],
    // pub id: u8,
    pub payload: Vec<u8>,
}

impl PeerMessage {
    pub fn new(message_type: MessageType, index: usize, begin: usize, length: usize) -> Self {
        let mut payload = Vec::<u8>::new();

        if length != 0 {
            payload.extend((index as u32).to_be_bytes());
            payload.extend((begin as u32).to_be_bytes());
            payload.extend((length as u32).to_be_bytes());
        }
        let message = Message {
            length: [0, 0, 0, 1 + payload.len() as u8],
            // id,
            payload,
        };
        match message_type {
            MessageType::Bitfield => PeerMessage::Bitfield(message),
            MessageType::Interested => PeerMessage::Interested(message),
            MessageType::Unchoke => PeerMessage::Unchoke(message),
            MessageType::Request => PeerMessage::Request(message),
            MessageType::Piece => PeerMessage::Piece(message),
        }
    }
    pub fn to_message(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        // vec.push(19u8);
        let (message, id) = match self {
            PeerMessage::Bitfield(message) => (message, MessageType::Bitfield.into()),
            PeerMessage::Interested(message) => (message, MessageType::Interested.into()),
            PeerMessage::Unchoke(message) => (message, MessageType::Unchoke.into()),
            PeerMessage::Request(message) => (message, MessageType::Request.into()),
            PeerMessage::Piece(message) => (message, MessageType::Piece.into()),
        };
        vec.extend(message.length);
        vec.push(id);
        vec.extend_from_slice(&message.payload);
        vec
    }
}
