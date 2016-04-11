extern crate classreader;
extern crate zip;

use classreader::*;
use std::env;
use std::fs::File;

#[path = "codecity.rs"]
mod codecity;

mod freeminer;

fn main() {
    env::args().nth(1).expect("usage: javamoose <class or jar file>...");
    let mut args = env::args();
    args.next(); // Skip exe name
    let mut classes: Vec<Class> = Vec::new();
    for f in args {
        if f.ends_with("class") {
            process_class_file(&f, &mut classes);
        } else if f.ends_with("jar") {
            process_jar_file(&f, &mut classes);
        } else {
            println!("Ignoring unknown file type {}", f);
        }
    }
    if classes.len() > 0 {
        println!("======================================================");
        let root_pkg = codecity::build_packages(&classes);
        println!("{:?}", root_pkg);
        freeminer::write_to_freeminer(&classes);
    }
    println!("Done!");
}

fn process_class_file(file_name: &String, classes: &mut Vec<Class>) {
    println!("Loading class file {}", file_name);
    println!("======================================================");
    let class = ClassReader::new_from_path(&file_name).unwrap();
    codecity::process_class(&class);
    classes.push(class);
}

fn process_jar_file(file_name: &String, classes: &mut Vec<Class>) {
    let file = File::open(file_name).expect("couldn't find a file!");
    let mut zip = zip::ZipArchive::new(file).expect("could not read JAR");
    for i in 0..zip.len() {
        let mut class_file = zip.by_index(i).unwrap();
        if class_file.name().ends_with("class") {
            let class = ClassReader::new_from_reader(&mut class_file).unwrap();
            codecity::process_class(&class);
            classes.push(class);
        }
    }
}
