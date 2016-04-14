//*****************************
// Java Codecity data model

extern crate classreader;

use self::classreader::*;
use std::collections::HashMap;
use std::cmp;

#[derive(Debug)]
pub struct Rect {
    // Dimenstions
    width:  u16,
    depth:  u16,
    // Placement
    pos_w:  u16,
    pos_d:  u16
}

impl Rect {
    fn new(w: u16, d: u16) -> Rect {
        Rect {
            width: w,
            depth: d,
            pos_w: 0,
            pos_d: 0
        }
    }

    fn area(self: &Rect) -> u16 {self.width * self.depth}

    fn place(self: &mut Rect, state: &mut PackState) {
        // println!("Used: {}x{} free: {}x{} placing {}x{}", cur_w, cur_d, free_w, free_d, b.width, b.depth);
        if self.width > state.free_w || self.depth > state.free_d {
            // decide the direction for growth
            if state.width <= state.depth {
                // Grow by width
                state.dir = Direction::Width;
                self.pos_w = state.width;
                self.pos_d = 1;
                state.cur_w = state.width;
                state.cur_d = 2 + self.depth;
                state.width += self.width + 1;
                state.depth = cmp::max(state.depth, self.depth + 2);
                state.free_w = self.width;
                state.free_d = state.depth - self.depth - 2;
            } else {
                // Grow by depth
                state.dir = Direction::Depth;
                self.pos_d = state.depth;
                self.pos_w = 1;
                state.cur_d = state.depth;
                state.cur_w = 2 + self.width;
                state.depth += self.depth + 1;
                state.width = cmp::max(state.width, self.width + 2);
                state.free_d = self.depth;
                state.free_w = state.width - self.width - 2;
            }
        } else {
            // Enough room
            match state.dir {
                Direction::Width => {
                    self.pos_w = state.cur_w;
                    self.pos_d = state.cur_d;
                    state.free_d -= self.depth + 1;
                    state.cur_d += self.depth + 1;
                }
                Direction::Depth => {
                    self.pos_d = state.cur_d;
                    self.pos_w = state.cur_w;
                    state.free_w -= self.width + 1;
                    state.cur_w += self.width + 1;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Building {
    name:   String,
    rect:   Rect,
    height: u16,
}

impl Building {
    fn new(class: &Class) -> Building {
        Building {
            name: get_class_name(class).to_string(),
            rect: Rect::new(class.methods.len() as u16 + 1, class.fields.len() as u16 + 1),
            height: get_total_code_size(&class) as u16 / 10 + 1,
        }
    }

    fn pos(self: &Building) -> (u16, u16) {
        (self.rect.pos_w, self.rect.pos_d)
    }
}

enum Direction {
    Width,
    Depth
}

#[derive(Debug)]
pub struct Package {
    name:  String,
    width: u16,
    depth: u16,
    packages:  HashMap<String, Package>,
    buildings: Vec<Building>
}

struct PackState {
    // Total aread
    width: u16,
    depth: u16,
    // Free in the corrent row
    free_w: u16,
    free_d: u16,
    // Position of the next placement
    cur_w: u16,
    cur_d: u16,
    // Direction we just grew the area
    dir: Direction
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

    fn size(self: &Package) -> (u16, u16) {
        (self.width, self.depth)
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
        let mut state = PackState {
            width: self.width,
            depth: self.depth,
            free_w: 0,
            free_d: 0,
            cur_w: 1,
            cur_d: 1,
            dir: Direction::Width
        };

        // TODO Packages
        // Buildings
        self.buildings.sort_by_key(|b| b.rect.area());
        self.buildings.reverse();
        for b in &mut self.buildings {
            b.rect.place(&mut state);
        }
        self.width = state.width;
        self.depth = state.depth;
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

fn mock(name: &str, w: u16, d: u16) -> Building {
  Building {
    name: name.to_string(),
    rect: Rect::new(w, d),
    height: 0
  }
}

/// A function to print out details about placement
fn debug_test(pkg: &Package) {
    println!("{:?}", pkg);
    for b in &pkg.buildings {
        println!("{:?}", b);
    }
}


#[test]
fn pack_4_singles() {
    let mut pkg = Package::new("_root_");
    for _ in 0..4 {
        pkg.buildings.push(mock("a", 1, 1));
    }
    pkg.pack();
    debug_test(&pkg);
    assert!(pkg.size() == (5, 5));
    // Make sure the last building is in the corner
    assert!(pkg.buildings[3].pos() == (3, 3));
}

#[test]
fn pack_5_singles() {
    let mut pkg = Package::new("_root_");
    for _ in 0..5 {
        pkg.buildings.push(mock("a", 1, 1));
    }
    pkg.pack();
    debug_test(&pkg);
    assert!(pkg.size() == (7, 5));
    assert!(pkg.buildings[4].pos() == (5, 1));
}

#[test]
fn pack_16_singles() {
    let mut pkg = Package::new("_root_");
    for _ in 0..16 {
        pkg.buildings.push(mock("a", 1, 1));
    }
    pkg.pack();
    debug_test(&pkg);
    assert!(pkg.size() == (9, 9));
    // Make sure the last building is in the corner
    assert!(pkg.buildings[15].pos() == (7, 7));
}

#[test]
fn pack_smart_square_width() {
    let mut pkg = Package::new("_root_");
    pkg.buildings.push(mock("w", 3, 1));
    for _ in 0..2 {
        pkg.buildings.push(mock("a", 1, 1));
    }
    pkg.pack();
    debug_test(&pkg);
    assert!(pkg.size() == (5, 5));
    // Make sure the last building is in the corner
    assert!(pkg.buildings[2].pos() == (3, 3));
}

#[test]
fn pack_buildings() {
    let mut pkg = Package::new("_root_");
    pkg.buildings.push(mock("b", 8, 3));
    pkg.buildings.push(mock("c", 4, 2));
    pkg.buildings.push(mock("a", 10, 7));
    pkg.buildings.push(mock("d", 3, 2));
    pkg.buildings.push(mock("e1", 1, 1));
    pkg.buildings.push(mock("e2", 1, 1));
    pkg.buildings.push(mock("e3", 1, 1));
    println!("Starting the test");
    pkg.pack();
    debug_test(&pkg);
    assert!(pkg.size() == (17, 13));
    assert!(pkg.buildings[0].pos() == (1, 1));
    assert!(pkg.buildings[1].pos() == (1, 9));
    assert!(pkg.buildings[2].pos() == (12, 1));
    assert!(pkg.buildings[3].pos() == (12, 4));
    assert!(pkg.buildings[4].pos() == (12, 7));
    assert!(pkg.buildings[5].pos() == (12, 9));
    assert!(pkg.buildings[6].pos() == (12, 11));

}