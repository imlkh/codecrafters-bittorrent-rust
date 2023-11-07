use std::env;

use std::io;
use std::io::Read;
use std::fs::File;

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
        let decoded_value = encoded_value.decode();
        println!("{}", decoded_value.to_string());

        Ok(())
    } else if command == "info" {
        let file_path = &args[2];
    //
        // convert Vec<u8> to &str
        let mut f = File::open(file_path)?;
        // let mut buffer : Vec<u8> = Vec::new();
        // let mut buffer = [0u8; 200];
        let mut buffer : Vec<u8> = Vec::new();
        // let mut buffer =  String::new();
        f.read_to_end(&mut buffer)?;

        let decoded_value = buffer.decode();
        println!("{}", decoded_value.to_string());

        // let n = f.read(&mut buffer)?;
        // println!("n = {n}");
        // println!("buffer.len() = {}",buffer.len());
    
        // one byte for two hexa-decimal values

        // match f.read_to_string(&mut buffer) {
        // match f.read_to_string(&mut buffer) {
            // Ok(l) => println!("ok : {}", l),
            // Err(l) => println!("error : {}", l)
        // }

        // println!("buffer[0] = {}", buffer[0] as char);
        // println!("buffer[1] = {}", buffer[1] as char);
        // let i = 212; println!("buffer[{}] = {}, {} as char", i, buffer[i], buffer[i] as char);
        // let i = 181; println!("buffer[{}] = {}, {} as char", i, buffer[i], buffer[i] as char);
        // let i = 182; println!("buffer[{}] = {}", i, buffer[i]);

       //  let _counter = buffer
       //      .iter()
       //      .enumerate()
       //      .filter(|(_,&b)| !b.is_ascii() || b.is_ascii_control())
       //      .count();
       //  // println!("_counter = {}", _counter);
       //
       //  let _non_ascii: Vec<usize> = buffer
       //      .iter()
       //      .enumerate()
       //      .filter(|(_,&b)| !b.is_ascii() || b.is_ascii_control())
       //      .map(|(i,_)| i)
       //      // .inspect(|i| print!("{}, ", i))
       //      .collect();
       //
       //  // println!("");
       //  // buffer.iter().for_each(|&b| println!("{}", b as char));
       //  // let str : String = buffer[..210]
       //  let str : String = buffer
       //  // buffer[..220]
       //  // buffer[..213]
       //  // buffer
       //      .iter()
       //      .enumerate()
       //      .filter(|(_,&b)| b.is_ascii() && !b.is_ascii_control())
       //      // .for_each(|&b| print!("{}", b as char));
       //      // .for_each(|&b| print!("{}", b.to ));
       //      // .inspect(|(i,_)| print!("{},", i))
       //      .map(|(_,&b)| b as char)
       //      .collect();
       //  // println!("{}", str);
       //
       //
       //  // println!("{}", &str[..172]);
       //  // println!("{}", &str[..98]);
       //
       //  // let mut str = (&str[..98]).to_owned();
       //  let mut str = (&str[..169]).to_owned();
       //  str += "4:testee";
       //
       //  // println!("str : {}", str);
       // 
       //  // char::from_u32(i)
       //  // println!("buffer : {}", buffer);
       //  let encoded_value = &str;
       //  // let char = str::from_utf8(&buffer[..4]).unwrap();
       //  // let encoded_value = str::from_utf8(&buffer).unwrap();
       //  // buffer.iter().map(|b| str::from_utf8(b).unwrap()).collect();
       //  // let encoded_value = str::from_utf8(&buffer).unwrap();
       //  // println!("char : {}", char);
       //  // println!("encoded_value : {}", encoded_value);
       //  // println!("buffer.len() = {}, buffer[0] = {}", buffer.len(), &buffer[0]);
       //  let decoded_value = encoded_value.decode();
       //  // buffer.iter().for_each(|b| println!("{}",b));
       //  // buffer.iter().for_each(|b|
       //  //     {
       //  //         if b.is_ascii() {
       //  //             println!("{}",b.to_ascii_uppercase())
       //  //         } else {
       //  //             println!("{} : not ascii",b)
       //  //         }
       //  //
       //  //     });
       //  println!("{}", decoded_value.to_string());
       //  if let Value::Object(map) = decoded_value
       //  {
       //      let result = &map["announce"];
       //      let map = &map["info"];
       //      // println!("map : {}", map.to_string());
       //      if let Value::String(str) = result {
       //          print!("Tracker URL: {}", str);
       //      }
       //      println!();
       //      println!("Length: {}", (*map)["length"]);
       //  }

        Ok(())
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
