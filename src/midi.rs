//*****************************
// Output to MIDI file

extern crate classreader;

use codecity::{MeasureMeta, MusicMeta};
use midi_file::core::{Channel, Clocks, DurationName, GeneralMidi, NoteNumber, Velocity};
use midi_file::file::{QuartersPerMinute, Track};
use midi_file::MidiFile;
use std::cmp::{min, max};

// durations
const QUARTER: u32 = 1024;
const EIGHTH: u32 = QUARTER / 2;
const WHOLE: u32 = QUARTER * 4;


// channel zero (displayed as channel 1 in any sequencer UI)
const CH: Channel = Channel::new(0);

pub fn write_to_file(path: &String, music: &Vec<MusicMeta>) {
    let mut mfile = MidiFile::new();

    // set up track metadata
    let mut track = Track::default();
    track.set_name("Singer").unwrap();
    track.set_instrument_name("The Java Code").unwrap();
    track.set_general_midi(CH, GeneralMidi::Oboe).unwrap();

    // set time signature and tempo
    track
        .push_time_signature(0, 6, DurationName::Eighth, Clocks::Quarter)
        .unwrap();
    track.push_tempo(0, QuartersPerMinute::new(116)).unwrap();

    for c in music {
        for (i, m) in c.methods().iter().enumerate() {
            render_chord(&mut track, m, i%3);
        }
    }

    // finish and write the file ///////////////////////////////////////////////////////////////////

    // add the track to the file
    mfile.push_track(track).unwrap();

    mfile.save(path).unwrap();
}

fn render_chord(track: &mut Track, m: &MeasureMeta, finger: usize) {
    if m.lines == 0 {return;}
    let note = get_tone( m.size, finger);
    let duration = max(min((m.lines as u32)*32, WHOLE), EIGHTH);
    track.push_lyric(0, format!("{}\n", m.name)).unwrap();
    track
        .push_note_on(0, CH, note, Velocity::new((max(60, m.complexity)) as u8))
        .unwrap();
    // the note-off event determines the duration of the note
    track
        .push_note_off(duration, CH, note, Velocity::new(m.size as u8))
        .unwrap();
}

fn get_tone(size: usize, offset: usize) -> NoteNumber {
    let i = 40 + min(128, size)/4 + offset*8;
    NoteNumber::new(i as u8)
}

//*****************************
// Unit tests
