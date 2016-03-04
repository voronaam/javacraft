extern crate classreader;
extern crate zip;
extern crate flate2;

use classreader::*;
use std::env;
use std::fs::File;
use flate2::Compression;
use flate2::write::ZlibEncoder;

fn main() {
    env::args().nth(1).expect("usage: javamoose <class or jar file>...");
    let mut args = env::args();
    args.next(); // Skip exe name
    for f in args {
        if f.ends_with("class") {
            process_class_file(&f);
        } else if f.ends_with("jar") {
            process_jar_file(&f);
        } else {
            println!("Ignoring unknown file type {}", f);
        }

    }
    println!("Done!");
}

fn process_class_file(file_name: &String) {
    println!("Loading class file {}", file_name);
    println!("======================================================");
    let class = ClassReader::new_from_path(&file_name).unwrap();
    process_class(&class);
}

fn process_jar_file(file_name: &String) {
    let file = File::open(file_name).expect("couldn't find a file!");
    let mut zip = zip::ZipArchive::new(file).expect("could not read JAR");
    for i in 0..zip.len() {
        let mut class_file = zip.by_index(i).unwrap();
        if class_file.name().ends_with("class") {
            let class = ClassReader::new_from_reader(&mut class_file).unwrap();
            process_class(&class);
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
    output_sql(class);
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

// Minetest/Freemine data model

struct NodeBlob {
    param0: [u16; 4096],
    param1: [u8; 4096],
    param2: [u8; 4096]
}

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

fn hex(v: &Vec<u8>) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for &byte in v {
        write!(&mut s, "{:02X}", byte).unwrap();
    }
    s
}

fn compute_position(x: i32, y: i32, z: i32) -> i32 {
    x + 4096 * (y + 4096 * z)
}

fn output_blob(blob: &NodeBlob, pos: i32) {
    use std::io::prelude::*;
    let blob_encoded = to_bytes(blob);
    let mut e = ZlibEncoder::new(Vec::new(), Compression::Default);
    e.write(&blob_encoded);
    let blob_compressed = e.finish().unwrap();
    let blob_hex = hex(&blob_compressed);
    let block = format!("19020202{}785E636460E458C3C0C0C0CCC091965F945B5C909A0CE489A465A6E6A44497A45694585BAB5483E8DA58068ECCBCB47C101BA8825BC91D06941858A0829C7031D7BC14CFBCB2D4BC92FCA24A2E0021791B940000000000024900000A0000000D64656661756C743A73746F6E650001000C64656661756C743A73616E640002000C64656661756C743A64697274000300036169720004001064656661756C743A646972745F6472790005001764656661756C743A646972745F776974685F67726173730006001564656661756C743A77617465725F666C6F77696E670007000F64656661756C743A67726173735F310008001164656661756C743A7369676E5F77616C6C0009000E64656661756C743A67726176656C0A0000", blob_hex);
    println!("INSERT INTO \"blocks\" VALUES({},X'{}');", pos, block);
}

fn output_sql(class: &Class) {
    // create base
    let blob = NodeBlob {
        param0: [0xe; 4096],
        param1: [0; 4096],
        param2: [0; 4096]
    };
    output_blob(&blob, compute_position(0, 1, 0));
    // create our class
    let mut parr = [0x3; 4096];
    // The location of a node in each of those arrays is (z*16*16 + y*16 + x)
    for x in 4..8 {
        for y in 4..8 {
            for z in 4..8 {
                let i = (z * 16 + y) * 16 + x;
                parr[i] = 0x0;
            }
        }
    }
    let blob2 = NodeBlob {
        param0: parr,
        param1: [0; 4096],
        param2: [0; 4096]
    };
    output_blob(&blob2, compute_position(0, 2, 0));

}
