//*****************************
// Java Codecity data model

extern crate classreader;

use self::classreader::*;
use std::collections::HashMap;
use std::cmp;

#[derive(Debug)]
pub struct Building {
    name:   String,
    width:  u16,
    depth:  u16,
    height: u16,
    // Placement
    pos_w:  u16,
    pos_d:  u16
}

impl Building {
    fn new(class: &Class) -> Building {
        Building {
            name: get_class_name(class).to_string(),
            width: class.methods.len() as u16 + 1,
            depth: class.fields.len() as u16 + 1,
            height: get_total_code_size(&class) as u16 / 10 + 1,
            pos_w: 0,
            pos_d: 0
        }
    }
}

#[derive(Debug)]
pub struct Package {
    name:  String,
    width: u16,
    depth: u16,
    packages:  HashMap<String, Package>,
    buildings: Vec<Building>
}

impl Package {
    /// Create a new Package.
    pub fn new(name: &str) -> Package {
        Package {
            name: name.to_string(),
            width: 1,
            depth: 1,
            packages: HashMap::new(),
            buildings: Vec::new()
        }
    }
    /// Add a class or package to this package
    fn add(&mut self, names: &[&str], class: &Class) {
        if names.len() == 1 {
            self.buildings.push(Building::new(class));
        } else {
            match names.split_first() {
                Some((head, tail)) => {
                    self.packages.entry(head.to_string()).or_insert_with(|| Package::new(head)).add(&tail, class);
                },
                _ => {}
            }
        }
    }
    
    /// Pack own members for the nice rendering
    fn pack(&mut self) {
        for (_, ch) in &mut self.packages {
            ch.pack();
        }
        let mut free_w = 0;
        let mut free_d = 0;
        let mut cur_w = 1;
        let mut cur_d = 1;
        // TODO Packages
        // Buildings
        self.buildings.sort_by_key(|b| b.width * b.depth);
        self.buildings.reverse();
        for b in &mut self.buildings {
            // println!("Used: {}x{} free: {}x{} placing {}x{}", cur_w, cur_d, free_w, free_d, b.width, b.depth);
            if b.width >= free_w || b.depth >= free_d {
                b.pos_w = self.width;
                b.pos_d = 1;
                cur_w = self.width;
                cur_d = 1 + b.depth;
                // grow by width always for now
                self.width += b.width + 1;
                self.depth = cmp::max(self.depth, b.depth + 2);
                free_w = b.width;
                free_d = self.depth - b.depth - 2;
            } else {
                // Enough room
                b.pos_w = cur_w;
                b.pos_d = cur_d;
                free_d -= b.depth;
                cur_d += b.depth;
            }
        }
    }
}

pub fn build_packages(classes: &Vec<Class>) -> Package {
    let mut root_pkg = Package::new("_root_");
    for c in classes {
        let name = get_class_name(&c);
        let names: Vec<&str> = name.split('/').collect();
        root_pkg.add(&names, c);
    }
    return root_pkg;
}

pub fn process_class(class: &Class) {
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
// Unit tests

#[test]
fn pack_buildings() {
    fn mock(name: &str, w: u16, d: u16) -> Building {
        Building {
            name: name.to_string(),
            width: w,
            depth: d,
            height: 0,
            pos_w: 0,
            pos_d: 0
        }
    };
    let mut pkg = Package::new("_root_");
    pkg.buildings.push(mock("b", 8, 3));
    pkg.buildings.push(mock("c", 4, 2));
    pkg.buildings.push(mock("a", 10, 7));
    pkg.buildings.push(mock("d", 3, 2));
    pkg.buildings.push(mock("e1", 1, 1));
    pkg.buildings.push(mock("e2", 1, 1));
    pkg.buildings.push(mock("e3", 1, 1));
    println!("Starting the test");
    println!("{:?}", pkg);
    pkg.pack();
    println!("{:?}", pkg);
    assert!(false);
}
