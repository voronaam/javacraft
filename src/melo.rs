//*****************************
// Output to MELO format

extern crate classreader;
use std::io::prelude::*;
use std::fs::File;
use codecity::MusicMeta;
    
pub fn write_to_file(path: &String, music: &Vec<MusicMeta>) {
    let mut file = File::create(path).unwrap();
    
    let text = b"Episode topic:\nWriting to file";
    file.write_all(text);
}

//*****************************
// Unit tests
