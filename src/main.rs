extern crate classreader;
extern crate zip;
extern crate flate2;
extern crate rusqlite;

use classreader::*;
use std::env;
use std::fs::File;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use rusqlite::Connection;

#[derive(Debug)]
struct Building {
    name:   String,
    width:  u16,
    depth:  u16,
    height: u16
}

impl Building {
    fn new(name: &str) -> Building {
        Building {
            name: name.to_string(),
            width: 1,
            depth: 1,
            height: 1
        }
    }

}

#[derive(Debug)]
struct Package {
    name:  String,
    width: u16,
    depth: u16,
    packages:  HashMap<String, Package>,
    buildings: Vec<Building>
}

impl Package {
    /// Create a new Package.
    fn new(name: &str) -> Package {
        Package {
            name: name.to_string(),
            width: 1,
            depth: 1,
            packages: HashMap::new(),
            buildings: Vec::new()
        }
    }
    /// Add a class or package to this package
    fn add(&mut self, names: &[&str]) {
        if names.len() == 1 {
            self.buildings.push(Building::new(names[0]));
        } else {
            match names.split_first() {
                Some((head, tail)) => {
                    self.packages.entry(head.to_string()).or_insert_with(|| Package::new(head)).add(&tail);
                },
                _ => {}
            }
        }
    }
}

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
        let mut root_pkg = Package::new("_root_");
        for c in &classes {
            let name = get_class_name(&c);
            let names: Vec<&str> = name.split('/').collect();
            root_pkg.add(&names);
        }
        println!("{:?}", root_pkg);
        /*
        let mut max_methods = classes[0].methods.len();
        for c in &classes {
            let methods = c.methods.len();
            if methods > max_methods {
                max_methods = methods;
            }
        }
        // Open DB connection
        let conn = Connection::open("/media/data/source/freeminer/worlds/world/map.sqlite").unwrap();
        let mut index = 1;
        for c in &classes {
            output_sql(&c, &conn, index, max_methods);
            index = index + 1;
        }
        conn.close().unwrap();
        */
    }
    println!("Done!");
}

fn process_class_file(file_name: &String, classes: &mut Vec<Class>) {
    println!("Loading class file {}", file_name);
    println!("======================================================");
    let class = ClassReader::new_from_path(&file_name).unwrap();
    process_class(&class);
    classes.push(class);
}

fn process_jar_file(file_name: &String, classes: &mut Vec<Class>) {
    let file = File::open(file_name).expect("couldn't find a file!");
    let mut zip = zip::ZipArchive::new(file).expect("could not read JAR");
    for i in 0..zip.len() {
        let mut class_file = zip.by_index(i).unwrap();
        if class_file.name().ends_with("class") {
            let class = ClassReader::new_from_reader(&mut class_file).unwrap();
            process_class(&class);
            classes.push(class);
        }
    }
}

fn process_class(class: &Class) {
    assert_eq!(0xCAFEBABE, class.magic);
    println!("class: {}", get_class_name(&class));
    println!("method count: {}", class.methods.len());
    println!("field count:  {}", class.fields.len());
    println!("code size:    {}", get_total_code_size(&class));
    for method in &class.methods {
        println!("\tmethod: {}{}", get_string(&class, method.name_index as usize), get_string(&class, method.descriptor_index as usize));
        println!("\t\tsize:       {}", get_method_size(&method));
        println!("\t\tlines:      {}", get_method_lines(&method));
        println!("\t\tcomplexity: {}", get_method_complexity(&method));
    }

    println!("======================================================");
}

/// Get constant from a pool, correcting for java's 1-based indexes.
fn get_const(class: &Class, i: usize) -> &ConstantPoolInfo {
    &class.constant_pool[i - 1]
}

/// Get string from constant pool
fn get_string(class: &Class, index: usize) -> String {
    match get_const(class, index) {
        &ConstantPoolInfo::Utf8(ref s) => s.clone(),
        _ => "?".to_string()
    }
}


fn get_class_name(class: &Class) -> String {
    match get_const(class, class.this_class as usize) {
        &ConstantPoolInfo::Class(index) => get_string(class, index as usize),
        _ => "?".to_string()
    }
}

fn get_total_code_size(class: &Class) -> usize {
    class.methods.iter().fold(0usize, |sum, method| sum + get_method_size(method))
}

/// Compute method size in byte code instructions
/// If multiple code segments are present in a method (not a javac output) sum it up
fn get_method_size(method: &Method) -> usize {
    method.attributes.iter().fold(0usize, |sum, a| sum + match a {
            &Attribute::Code{ref code, ..} => code.len(),
            _ => 0
        })
}

/// Compute method size in lines of code.
/// If multiple code segments or multiple line number attrbitues are present in a method (not a javac output) sum it up
fn get_method_lines(method: &Method) -> u16 {
    method.attributes.iter().fold(0, |sum, a| sum + match a {
            &Attribute::Code{ref attributes, ..} => attributes.iter().fold(0, |sum, a| sum + match a {
                &Attribute::LineNumberTable(ref line_table) => {
                    let end = get_opt_line_number(line_table.last());
                    let start = get_opt_line_number(line_table.first());
                    if start > end { // TODO: Investigate when it happens
                        0
                    } else {
                        end - start
                    }
                },
                _ => 0
            }),
            _ => 0
        })

}

fn get_opt_line_number(line: Option<&LineNumber>) -> u16 {
    line.map(|x| x.line_number).unwrap_or(0)
}

fn get_method_complexity(method: &Method) -> u16 {
    method.attributes.iter().fold(0, |sum, a| sum + match a {
            &Attribute::Code{ref code, ..} => get_code_complexity(code),
            _ => 0
        })
}

/// Using complexity definition from http://www.leepoint.net/principles_and_practices/complexity/complexity-java-method.html
/// Start with 1 for the method. Add one for each of the following flow-related elements that are found in the method.
/// Returns     Each return that isn't the last statement of a method.
/// Selection   if, else, case, default.
/// Loops   for, while, do-while, break, and continue.
/// Operators   &&, ||, ?, and :
/// Exceptions  catch, finally, throw, or throws clause.
/// Bytecode reference: http://www.angelibrary.com/computer/java_21days/ch32.htm
fn get_code_complexity(code: &Vec<(u32, Instruction)>) -> u16 {
    // println!("{:?}", code);
    // We start from zero to account for always-present final return - it will add the "starting" 1 for us.
    code.iter().fold(0, |sum, instruction| sum + match instruction.1 {
        Instruction::areturn => 1,
        Instruction::athrow => 1,
        Instruction::dreturn => 1,
        Instruction::freturn => 1,
        Instruction::if_acmpeq(..) => 1,
        Instruction::if_acmpne(..) => 1,
        Instruction::if_icmpeq(..) => 1,
        Instruction::if_icmpne(..) => 1,
        Instruction::if_icmplt(..) => 1,
        Instruction::if_icmpge(..) => 1,
        Instruction::if_icmpgt(..) => 1,
        Instruction::if_icmple(..) => 1,
        Instruction::ifeq(..) => 1,
        Instruction::ifne(..) => 1,
        Instruction::iflt(..) => 1,
        Instruction::ifge(..) => 1,
        Instruction::ifgt(..) => 1,
        Instruction::ifle(..) => 1,
        Instruction::ifnonnull(..) => 1,
        Instruction::ifnull(..) => 1,
        Instruction::ireturn => 1,
        Instruction::lookupswitch(_, ref jumps) => jumps.len() as u16, // tricky - lets use the size of the jump table
        Instruction::lreturn => 1,
        Instruction::return_ => 1,
        Instruction::tableswitch(_, _, ref jumps) => jumps.len() as u16, // tricky - lets use the size of the jump table
        // Subroutines in the method. Currently - do not count them towards the complexity. They are used to compile "finally", but I do not see them as branching really. Finally block is executed every time
        // Instruction::jsr(..) => 1,
        // Instruction::jsr_w(..) => 1,
        // Instruction::ret => 1,
        // Instruction::ret_w => 1,
        _ => 0
    })
}

//*****************************
// Minetest/Freemine data model

struct NodeBlob {
    param0: [u16; 4096],
    param1: [u8; 4096],
    param2: [u8; 4096]
}

// blob to bytes...
fn to_bytes(blob: &NodeBlob) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::new();
    for i in 0..4096 {
        let p = blob.param0[i];
        vec.push((p >> 8) as u8);
        vec.push(p as u8);
    }
    for i in 0..4096 {
        vec.push(blob.param1[i]);
    }
    for i in 0..4096 {
        vec.push(blob.param2[i]);
    }
    vec
}

// Write one variable to the vector
fn push_variable(vec: &mut Vec<u8>, name: &str, value: &str) {
    let klen = name.len() as u16;
    vec.push((klen >> 8) as u8);
    vec.push(klen as u8);
    for b in name.bytes() {
        vec.push(b);
    }
    let vlen = value.len() as u32;
    vec.push((vlen >> 24) as u8);
    vec.push((vlen >> 16) as u8);
    vec.push((vlen >> 8) as u8);
    vec.push(vlen as u8);
    for b in value.bytes() {
        vec.push(b);
    }
}

// Encode stuff we want to have in meta (signs mostly).
fn meta_bytes(text: &str, pos: u16) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::new();
    vec.push(0x01); // version
    vec.push(0x00); // count byte 0
    vec.push(0x01); // count byte 1
    vec.push((pos >> 8) as u8); // position byte 0
    vec.push(pos as u8); // position byte 1
    vec.push(0x00); // num_vars byte 0
    vec.push(0x00); // num_vars byte 1
    vec.push(0x00); // num_vars byte 2
    vec.push(0x03); // num_vars byte 3
    push_variable(&mut vec, "formspec", "field[text;;${text}]");
    push_variable(&mut vec, "infotext", text);
    push_variable(&mut vec, "text", text);
    // EndInventory
    vec.append(&mut vec![0x45 as u8, 0x6e, 0x64, 0x49, 0x6e, 0x76, 0x65, 0x6e, 0x74, 0x6f, 0x72, 0x79, 0x0a]);
    vec
}

// bytes to hex. There should be a better way for sure.
fn hex(v: &Vec<u8>) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for &byte in v {
        write!(&mut s, "{:02X}", byte).unwrap();
    }
    s
}

// blob position in the world
fn compute_position(x: i32, y: i32, z: i32) -> i32 {
    x + 4096 * (y + 4096 * z)
}

// node position inside the blob
fn node_pos(x: usize, y: usize, z: usize) -> usize {
    (z * 16 + y) * 16 + x
}

fn output_blob(blob: &NodeBlob, conn: &Connection, pos: i32, sign: &str, sign_pos: usize) {
    use std::io::prelude::*;
    let blob_encoded = to_bytes(blob);
    let mut e = ZlibEncoder::new(Vec::new(), Compression::Default);
    e.write(&blob_encoded).unwrap();
    let blob_compressed = e.finish().unwrap();
    let blob_hex = hex(&blob_compressed);
    
    let meta_encoded = meta_bytes(sign, sign_pos as u16);
    let mut e1 = ZlibEncoder::new(Vec::new(), Compression::Default);
    e1.write(&meta_encoded).unwrap();
    let meta_compressed = e1.finish().unwrap();
    let meta_hex = hex(&meta_compressed);
    let block = format!("19060202{}{}0000000000024900000A0000000D64656661756C743A73746F6E650001000C64656661756C743A73616E640002000C64656661756C743A64697274000300036169720004001064656661756C743A646972745F6472790005001764656661756C743A646972745F776974685F67726173730006001564656661756C743A77617465725F666C6F77696E670007000F64656661756C743A67726173735F310008001164656661756C743A7369676E5F77616C6C0009000E64656661756C743A67726176656C0A0000", blob_hex, meta_hex);
    conn.execute(&format!("DELETE FROM blocks WHERE pos = {};", pos), &[]).unwrap();
    conn.execute(&format!("INSERT INTO blocks VALUES({},X'{}');", pos, block), &[]).unwrap();
}

fn output_sql(class: &Class, conn: &Connection, offset: i32, scale: usize) {
    if class.methods.len() == 0 {
        return;
    }
    // create base
    let blob = NodeBlob {
        param0: [0xe; 4096],
        param1: [0; 4096],
        param2: [0; 4096]
    };
    output_blob(&blob, conn, compute_position(0 + offset * 2, 1, 0), "", node_pos(0, 0, 0));
    // create our class
    let mut parr = [0x3 as u16; 4096];
    let mut parr2 = [0x0 as u8; 4096];
    let mut parr3 = [0x0 as u8; 4096];
    // Let's create a block of proper size
    // The location of a node in each of those arrays is (z*16*16 + y*16 + x)
    let height = (class.methods.len() - 1) * 16 / scale;
    for x in 4..8 {
        for y in 0..height {
            for z in 4..8 {
                let i = node_pos(x, y, z);
                parr[i] = 0x0;
                parr2[i] = 0xf;
            }
        }
    }
    // Let's place a sign
    let sign_pos = node_pos(4, height, 4);
    parr[sign_pos] = 0x8;
    parr2[sign_pos] = 0x0f;
    parr3[sign_pos] = 0x1;
    let blob2 = NodeBlob {
        param0: parr,
        param1: parr2,
        param2: parr3
    };
    output_blob(&blob2, conn, compute_position(0 + offset * 2, 2, 0), &get_class_name(&class), sign_pos);

}
