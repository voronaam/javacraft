//*****************************
// Minetest/Freemine data model

extern crate classreader;
extern crate rusqlite;
extern crate flate2;
extern crate multiarray;

use self::flate2::Compression;
use self::flate2::write::ZlibEncoder;
use self::rusqlite::Connection;
use self::multiarray::{MultiArray, Dim3};
use codecity::Package;


pub fn write_to_freeminer(path: &String, root: &Package) {
	let voxels = build_array(root);
    // Open DB connection
    let conn = Connection::open(path).unwrap();
	output_sql(&conn, root, &voxels);
    conn.close().unwrap();
}

fn build_array(root: &Package) -> MultiArray<u8, Dim3> {
	use self::multiarray::*;
	let height = root.height();
	let (width, depth) = root.size();
	// u8 for the future use, when we'll have more than just non-emptiness to track
	let mut voxels = Array3D::new([width as usize, depth as usize, height as usize], 0 as u8);
	fill_from_package(root, &mut voxels, 0);
	return voxels;
}

fn fill_from_package(pkg: &Package, voxels: &mut MultiArray<u8, Dim3>, oz: usize) {
	let (w, d) = pkg.size();
	let (ox, oy) = pkg.position();
	// Fill the package itself
	for x in ox .. ox + w {
		for y in oy .. oy + d {
			voxels[[x as usize, y as usize, oz]] = 1u8;
		}
	}
	// Fill the subpackages
	for p in &pkg.packages() {
		fill_from_package(p, voxels, oz + 1);
	}
	// Fill the buildings
	for b in pkg.buildings() {
		let (px, py) = b.position();
		let (bw, bd) = b.size();
		let h = b.height() as usize;
		for x in px .. px + bw {
			for y in py .. py + bd {
				for z in oz + 1 .. oz + 1 + h {
					voxels[[x as usize, y as usize, z]] = 1;
				}
			}
		}
	}

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
// Note that in freeminer x and z are horizontals and y (!) is the vertical (grows up)
fn compute_position(x: usize, y: usize, z: usize) -> usize {
    x + 4096 * (y + 4096 * z)
}

// node position inside the blob
fn node_pos(x: usize, y: usize, z: usize) -> usize {
    (z * 16 + y) * 16 + x
}

fn output_blob(blob: &NodeBlob, conn: &Connection, pos: usize, sign: &str, sign_pos: usize) {
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

static BLOB_DIM: usize = 16;

fn voxels_to_blob(voxels: &MultiArray<u8, Dim3>, sx: usize, sy: usize, sz: usize) -> NodeBlob {
    let mut parr = [0x3 as u16; 4096];
    let mut parr2 = [0x0 as u8; 4096];
    let parr3 = [0x0 as u8; 4096];
    for x in sx .. sx + BLOB_DIM {
        for y in sy .. sy + BLOB_DIM {
            for z in sz .. sz + BLOB_DIM {
                if x + 1 < voxels.extents()[0] &&
                   y + 1 < voxels.extents()[1] &&
                   z + 1 < voxels.extents()[2] &&
                   voxels[[x,y,z]] == 1u8 {
                    let i = node_pos(x - sx, z - sz, y - sy);
                    parr[i] = 0x0;
                    parr2[i] = 0xf;
                }
            }
        }
    }
    return NodeBlob {
        param0: parr,
        param1: parr2,
        param2: parr3
    };
}

fn output_sql(conn: &Connection, package: &Package, voxels: &MultiArray<u8, Dim3>) {
    // First, determine the dimensions
    let dimx = voxels.extents()[0] / BLOB_DIM + 1;
    let dimy = voxels.extents()[1] / BLOB_DIM + 1;
    let dimz = voxels.extents()[2] / BLOB_DIM + 1;
    println!("Total map dimensions: {}x{}x{} blobs", dimx, dimy, dimz);
    
    for x in 0 .. dimx {
        for y in 0 .. dimy {
            // create base
            let blob = NodeBlob {
                param0: [0xe; 4096],
                param1: [0; 4096],
                param2: [0; 4096]
            };
            output_blob(&blob, conn, compute_position(x, 1, y), "", node_pos(0, 0, 0));
            for z in 0 .. dimz {
                println!("Working on a blob: {}x{}x{}", x, y, z);
                // create our class
                let blob2 = voxels_to_blob(voxels, x * BLOB_DIM, y * BLOB_DIM, z * BLOB_DIM);
                output_blob(&blob2, conn, compute_position(x, 2 + z, y), "", node_pos(0, 0, 0));
            }
            // Create air on top of our last block
            let blob3 = NodeBlob {
                param0: [0x3; 4096],
                param1: [0; 4096],
                param2: [0; 4096]
            };
            output_blob(&blob3, conn, compute_position(x, 2 + dimz, y), "", node_pos(0, 0, 0));
            output_blob(&blob3, conn, compute_position(x, 3 + dimz, y), "", node_pos(0, 0, 0));
        }
    }
    /*
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
    output_blob(&blob2, conn, compute_position(0, 2, 0), &package.name(), sign_pos);
    */
}

//*****************************
// Unit tests
