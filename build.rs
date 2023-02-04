use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use quote::quote;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");

    let map_filename = "tiles/main.tmx";
    println!("cargo:rerun-if-changed={map_filename}");
    let tileset_filename = "tiles/ground-tileset.tsx";
    println!("cargo:rerun-if-changed={tileset_filename}");

    let tileset_file = File::open(tileset_filename).unwrap();
    let mut tileset_reader = std::io::BufReader::new(tileset_file);

    let mut tileset_with_types = String::new();

    tileset_reader
        .read_to_string(&mut tileset_with_types)
        .unwrap();

    let tileset_with_types = tileset_with_types.replace("class", "type");

    let tileset_with_types_filename = "tiles/ground-tileset-with-types.tsx";
    let tileset_file = File::create(tileset_with_types_filename).unwrap();

    let mut tileset_writer = std::io::BufWriter::new(tileset_file);

    tileset_writer
        .write_all(tileset_with_types.as_bytes())
        .unwrap();

    tileset_writer.flush().unwrap();

    let mut loader = tiled::Loader::new();

    let map = loader.load_tmx_map(Path::new(map_filename)).unwrap();

    let tileset = loader
        .load_tsx_tileset(Path::new(tileset_with_types_filename))
        .unwrap();

    let width = map.width;
    let height = map.height;

    let background_layer = &map.get_layer(0).unwrap();
    let background_tiles = extract_tiles(background_layer);

    let section_layer_range = 1..map.layers().len();
    let section_tiles = section_layer_range
        .map(|i| {
            let layer = &map.get_layer(i).unwrap();
            let tiles = extract_tiles(layer);
            quote! { &[#(#tiles),*] }
        });

    let mut tile_types = HashMap::new();

    for tile in tileset.tiles() {
        if let Some("Collision") = tile.1.tile_type.as_deref() {
            tile_types.insert(tile.0, 1u8);
        } else {
            tile_types.insert(tile.0, 0u8);
        }
    }

    let tile_types = (0..tileset.tilecount).map(|id| tile_types.get(&(id)).unwrap_or(&0));

    let output = quote! {
        pub const SECTION_MAPS: &'static [&'static [u16]] = &[#(#section_tiles),*];
        pub const BACKGROUND_MAP: &[u16] = &[#(#background_tiles),*];
        pub const WIDTH: i32 = #width as i32;
        pub const HEIGHT: i32 = #height as i32;

        pub const TILE_TYPES: &[u8] = &[#(#tile_types),*];
    };

    let output_file = File::create(format!("{out_dir}/tilemap.rs"))
        .expect("failed to open tilemap.rs file for writing");
    let mut writer = BufWriter::new(output_file);

    write!(&mut writer, "{output}").unwrap();
}

fn extract_tiles<'map>(layer: &'_ tiled::Layer<'map>) -> impl Iterator<Item = u16> + 'map {
    match layer.layer_type() {
        tiled::LayerType::TileLayer(tiled::TileLayer::Finite(tiles)) => {
            let width = tiles.width();
            let height = tiles.height();
            (0..width * height).map(move |i| {
                let tile = tiles
                    .get_tile((i % width) as i32, (i / width) as i32);

                tile.map_or(32, |tile| tile.id())
            })
        }
        _ => unimplemented!("cannot use infinite layer"),
    }
    .map(get_map_id)
}

fn get_map_id(tile_id: u32) -> u16 {
    tile_id as u16
}
