extern crate classreader;

use classreader::*;
use std::env;

fn main() {
    let file_name = env::args().nth(1).expect("usage: javamoose <class file>");
    println!("Loading class file {}", file_name);
    let class = ClassReader::new_from_path(&file_name).unwrap();

    assert_eq!(0xCAFEBABE, class.magic);
    println!("class: {}", get_class_name(&class));
    println!("method count: {}", class.methods.len());
    println!("field count:  {}", class.fields.len());
    println!("code size:    {}", get_total_code_size(&class));


    // println!("{:?}", class);

    println!("Done!");
}

fn get_const(class: &Class, i: usize) -> &ConstantPoolInfo {
    &class.constant_pool[i - 1]
}

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
    let mut sum: usize = 0;
    for m in &class.methods {
        for a in &m.attributes {
            sum = match a {
                &Attribute::Code{ref code, ..} => sum + &code.len(),
                _ => sum
            };
        }
    }
    sum
}
