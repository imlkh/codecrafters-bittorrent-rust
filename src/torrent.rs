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
    fn to_handshake(self) -> HandShake;
}

impl ToHandShake for Torrent {
    fn to_handshake(self) -> HandShake {
        HandShake {
            info_hash: self.info_hash,
            peer_id: self.peer_id,
        }
    }
}

impl ToHandShake for Vec<u8> {
    fn to_handshake(self) -> HandShake {
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

pub struct PeerMessage {
    pub length: [u8; 4],
    pub id: u8,
    pub payload: Vec<u8>,
}

impl PeerMessage {
    pub fn to_message(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        // vec.push(19u8);
        vec.extend(self.length);
        vec.push(self.id);
        vec.extend_from_slice(&self.payload);
        vec
    }
}

// print!("Tracker URL: {}", &torrent.url);
// println!();
// println!("Length: {}", &torrent.length);
// println!("Info Hash: {}", &torrent.info_hash);
// println!("Piece Length: {}", &torrent.piece_length);
// println!("Piece Hashes:\n{}", &torrent.piece_hashes);
