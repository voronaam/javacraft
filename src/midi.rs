//*****************************
// Output to MELO format

extern crate classreader;

use codecity::{MeasureMeta, MusicMeta};
use midi_file::MidiFile;
use midi_file::core::{GeneralMidi, DurationName, Clocks, Velocity, NoteNumber, Channel};
use midi_file::file::{Track, QuartersPerMinute};
use std::fs::File;
use std::io::prelude::*;

const TONES: &'static [NoteNumber] = &[
    NoteNumber::new(72),
    NoteNumber::new(74),
    NoteNumber::new(76),
    NoteNumber::new(78),
    NoteNumber::new(80),
    NoteNumber::new(82),
    NoteNumber::new(84),
    NoteNumber::new(86),
    NoteNumber::new(88),
    NoteNumber::new(90),
    NoteNumber::new(92),
    NoteNumber::new(94),
    NoteNumber::new(96),
    NoteNumber::new(98),
    NoteNumber::new(100),
    NoteNumber::new(102),
    NoteNumber::new(104),
    NoteNumber::new(106),
    NoteNumber::new(108),
];

// durations
const QUARTER: u32 = 1024;
const EIGHTH: u32 = QUARTER / 2;
const DOTTED_QUARTER: u32 = QUARTER + EIGHTH;

// pitches
const C4: NoteNumber = NoteNumber::new(72);
const D4: NoteNumber = NoteNumber::new(74);
const E4: NoteNumber = NoteNumber::new(76);

// some arbitrary velocity
const V: Velocity = Velocity::new(64);

// channel zero (displayed as channel 1 in any sequencer UI)
const CH: Channel = Channel::new(0);

pub fn write_to_file(path: &String, music: &Vec<MusicMeta>) {
    let mut mfile = MidiFile::new();

    // set up track metadata
    let mut track = Track::default();
    track.set_name("Singer").unwrap();
    track.set_instrument_name("Alto").unwrap();
    track.set_general_midi(CH, GeneralMidi::SynthVoice).unwrap();


    
    // set time signature and tempo
    track
        .push_time_signature(0, 6, DurationName::Eighth, Clocks::DottedQuarter)
        .unwrap();
    track.push_tempo(0, QuartersPerMinute::new(116)).unwrap();

    for c in music {
        for m in c.methods() {
            render_chord(&mut track, m, 0);
        }
    }

    // finish and write the file ///////////////////////////////////////////////////////////////////

    // add the track to the file
    mfile.push_track(track).unwrap();

    
    mfile.save(path).unwrap();
}

fn render_chord(track: &mut Track, m: &MeasureMeta, finger: u16) {
    let tone_count = get_tone_count(m.lines);
    let base = get_base(m.size);
    if tone_count > 0 {
        let note = get_tone(base, m.complexity, finger);
        track.push_lyric(0, format!("{}\n", m.name)).unwrap();
        track.push_note_on(0, CH, note, V).unwrap();
        // the note-off event determines the duration of the note
        track
            .push_note_off(EIGHTH, CH, note, Velocity::default())
            .unwrap();
    
    }
}

fn get_tone_count(c: u16) -> u16 {
    match c / 8 {
        0..=8 => c / 8,
        _ => 8,
    }
}

fn get_base(c: usize) -> usize {
    match c / 2 {
        0..=10 => c,
        11..=20 => 10 + (c - 10) / 2,
        21..=40 => 15 + (c - 20) / 4,
        41..=80 => 20 + (c - 40) / 8,
        81..=160 => 30 + (c - 80) / 16,
        _ => 18,
    }
}

fn get_complexity_shift(complexity: u16) -> u16 {
    match complexity {
        0..=2 => 2,
        3..=4 => 3,
        _ => 4,
    }
}

fn get_tone(base: usize, complexity: u16, offset: u16) -> NoteNumber {
    let i = base as u16 + (offset % get_complexity_shift(complexity));
    TONES[(i % 19) as usize]
}

//*****************************
// Unit tests
