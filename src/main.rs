extern crate classreader;

use classreader::*;
use std::env;

fn main() {
    let file_name = env::args().nth(1).expect("usage: javamoose <class file>");
    println!("Loading class file {}", file_name);
    println!("======================================================");
    let class = ClassReader::new_from_path(&file_name).unwrap();

    assert_eq!(0xCAFEBABE, class.magic);
    println!("class: {}", get_class_name(&class));
    println!("method count: {}", class.methods.len());
    println!("field count:  {}", class.fields.len());
    println!("code size:    {}", get_total_code_size(&class));
    for method in &class.methods {
        println!("\tmethod: {}{}", get_string(&class, method.name_index as usize), get_string(&class, method.descriptor_index as usize));
        println!("\t\tsize:  {}", get_method_size(&method));
        println!("\t\tlines: {}", get_method_lines(&method));
        // println!("{:?}", method);
    }



    println!("======================================================");
    println!("Done!");
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
                &Attribute::LineNumberTable(ref line_table) => get_opt_line_number(line_table.last()) - get_opt_line_number(line_table.first()),
                _ => 0
            }),
            _ => 0
        })

}

fn get_opt_line_number(line: Option<&LineNumber>) -> u16 {
    line.map(|x| x.line_number).unwrap_or(0)
}


