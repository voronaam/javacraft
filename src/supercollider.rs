//*****************************
// Output an SuperCollider script

extern crate classreader;

use codecity::{MeasureMeta, MusicMeta};
use File;
use std::io::Write;

pub fn write_to_file(path: &String, music: &Vec<MusicMeta>) {
    let mut file = File::create(path).unwrap();

    for c in music {
        for (i, m) in c.methods().iter().enumerate() {
            render_chord(&mut file, m, i%3);
        }
    }

}

fn render_chord(file: &mut File, m: &MeasureMeta, finger: usize) {
    if m.lines == 0 {return;}
    write!(file, "\"{}\".postln;\n", m.name).unwrap();
    write!(file, "~class.({}, {}, {});\n", m.size, m.lines, m.complexity).unwrap();
}

//*****************************
// Unit tests
