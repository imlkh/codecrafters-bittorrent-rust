use anyhow::Context;
use std::fmt;
// use super::bencode;
use serde_json::Value;

pub struct Torrent {
    pub url: String,
    pub length: usize,
    pub info_hash: String,
    pub piece_length: usize,
    pub piece_hashes: String,
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
            info_hash: map["info hash"]
                .as_str()
                .context("this is not a string")?
                .to_string(),
            piece_length: map["info"]["piece length"]
                .as_i64()
                .context("this is not an integer")? as usize,
            piece_hashes: map["info"]["pieces"]
                .as_str()
                .context("this is not a string")?
                .to_string(),
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
    // print!("Tracker URL: {}", &torrent.url);
    // println!();
    // println!("Length: {}", &torrent.length);
    // println!("Info Hash: {}", &torrent.info_hash);
    // println!("Piece Length: {}", &torrent.piece_length);
    // println!("Piece Hashes:\n{}", &torrent.piece_hashes);
}
