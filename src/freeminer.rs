//*****************************
// Minetest/Freemine data model

extern crate rusqlite;
extern crate flate2;

use self::flate2::Compression;
use self::flate2::write::ZlibEncoder;
use self::rusqlite::Connection;

use codecity::*;

pub fn write_to_freeminer(root: &Package) {
    let mut max_methods = classes[0].methods.len();
    for c in classes {
        let methods = c.methods.len();
        if methods > max_methods {
            max_methods = methods;
        }
    }
    // Open DB connection
    let conn = Connection::open("/media/data/source/freeminer/worlds/world/map.sqlite").unwrap();
    let mut index = 1;
    for c in classes {
        output_sql(&c, &conn, index, max_methods);
        index = index + 1;
    }
    conn.close().unwrap();
}

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
    // output_blob(&blob2, conn, compute_position(0 + offset * 2, 2, 0), &get_class_name(&class), sign_pos);
    output_blob(&blob2, conn, compute_position(0 + offset * 2, 2, 0), "?", sign_pos);

}
