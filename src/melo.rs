//*****************************
// Output to MELO format

extern crate classreader;
use std::io::prelude::*;
use std::fs::File;
use codecity::{MusicMeta, MeasureMeta};
    
pub fn write_to_file(path: &String, music: &Vec<MusicMeta>) {
    let mut file = File::create(path).unwrap();
    
    write!(file, "voice Right {{ channel: 2 }}\nvoice Left {{ channel: 1 }}\n\n").unwrap();
    
    // Right hand
    write!(file, "play Right {{:|").unwrap();
    for c in music {
        for m in c.methods() {
            render_measure(&mut file, m);
        }
    }
    write!(file, "}}").unwrap();
}

fn render_measure(file: &mut File, m: &MeasureMeta) {
    write!(file, "c g e c |").unwrap();
}

//*****************************
// Unit tests
