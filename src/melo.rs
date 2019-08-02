//*****************************
// Output to MELO format

extern crate classreader;
use std::io::prelude::*;
use std::fs::File;
use codecity::{MusicMeta, MeasureMeta};

const TONES: &'static [&'static str] = &["C", "D", "E", "F", "G", "a", "b", "c", "d", "e", "f", "g", "a'", "b'", "c'", "d'", "e'", "f'", "g'"];
    
pub fn write_to_file(path: &String, music: &Vec<MusicMeta>) {
    let mut file = File::create(path).unwrap();
    
    write!(file, "tempo: 200\n\nvoice Right {{ channel: 2 }}\nvoice Left {{ channel: 1, octave: -1 }}\n\n").unwrap();
    
    // Right hand
    write!(file, "play Right\n{{\n:|").unwrap();
    for c in music {
        for m in c.methods() {
            render_measure(&mut file, m);
        }
    }
    write!(file, "\n}}\n\n").unwrap();
    
    // Left hand
    write!(file, "play Left\n{{\n:|").unwrap();
    for c in music {
        for m in c.methods() {
            render_chord(&mut file, m, 0);
        }
    }
    write!(file, "\n:|").unwrap();
    for c in music {
        for m in c.methods() {
            render_chord(&mut file, m, 1);
        }
    }
    write!(file, "\n:|").unwrap();
    for c in music {
        for m in c.methods() {
            render_chord(&mut file, m, 2);
        }
    }
    write!(file, "\n}}\n\n").unwrap();
}

fn render_measure(file: &mut File, m: &MeasureMeta) {
    let tone_count = get_tone_count(m.lines);
    let base = get_base(m.size);
    for x in 0..tone_count {
        write!(file, "{} ", get_tone(base, m.complexity, x)).unwrap();
    }
    if tone_count > 0 {
        write!(file, "|").unwrap();
    }
}

fn render_chord(file: &mut File, m: &MeasureMeta, finger: u16) {
    let tone_count = get_tone_count(m.lines);
    let base = get_base(m.size);
    if tone_count > 0 {
        write!(file, "{} |", get_tone(base, m.complexity, finger)).unwrap();
    }
}

fn get_tone_count(c: u16) -> u16 {
    match c / 4 {
        0...8 => c/4,
        _      => 8
    }
}

fn get_base(c: usize) -> usize {
    match c/2 {
        0  ... 10  => c,
        11 ... 20  => 10 + (c - 10)/2,
        21 ... 40  => 15 + (c - 20)/4,
        41 ... 80  => 20 + (c - 40)/8,
        81 ... 160 => 30 + (c - 80)/16,
        _ => 18
    }
}

fn get_complexity_shift(complexity: u16) -> u16 {
    match complexity {
        0  ...  2 => 2,
        3  ...  4 => 3,
        _         => 3 + complexity / 5
    }
}

fn get_tone(base: usize, complexity: u16, offset: u16) -> &'static str {
    let i = base as u16 + (offset % 3)*get_complexity_shift(complexity);
    TONES[(i % 19) as usize]
}

//*****************************
// Unit tests
