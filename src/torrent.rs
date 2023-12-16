use anyhow::Context;
use serde_json::Value;
use std::fmt;

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
        self.peer_id
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
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
        hex::decode(&(self.val)).expect("Decoding failed")
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
        let map = decoded_value.as_object().context("this is not an object")?;

        Ok(Torrent {
            url: map["announce"]
                .as_str()
                .context("this is not a string")?
                .to_string(),
            length: map["info"]["length"]
                .as_i64()
                .context("this is not an integer")? as usize,
            info_hash: InfoHash {
                val: map["info hash"]
                    .as_str()
                    .context("this is not a string")?
                    .to_string(),
            },
            piece_length: map["info"]["piece length"]
                .as_i64()
                .context("this is not an integer")? as usize,
            piece_hashes: map["info"]["pieces"]
                .as_str()
                .context("this is not a string")?
                .to_string(),
            peer_id: b"00112233445566778899".to_vec(),
        })
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
