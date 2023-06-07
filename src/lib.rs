#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb::{
    display::tiled::{TileFormat, TileSet},
    fixnum::FixedNum,
};
use alloc::rc::Rc;
use game::Game;
use world::World;

mod entity;
mod game;
mod gfx;
mod timer;
mod world;

mod tilemap {
    include!(concat!(env!("OUT_DIR"), "/tilemap.rs"));
}

agb::include_background_gfx!(tileset, tiles => "gfx/tileset.png");

type Number = FixedNum<8>;

pub fn main(mut gba: agb::Gba) -> ! {
    let vblank = agb::interrupt::VBlank::get();
    vblank.wait_for_vblank();
    let (background, mut vram) = gba.display.video.tiled0();

    vram.set_background_palettes(tileset::PALETTES);
    let tileset = TileSet::new(tileset::tiles.tiles, TileFormat::FourBpp);
    let tileset = Rc::new(&tileset);

    let world = World::new(tileset, &background, &mut vram);

    let object_controller = gba.display.object.get_managed();

    let mut game = Game::new(&object_controller, world);
    game.transition_to_state(game::GameState::Start);

    let mut input = agb::input::ButtonController::new();

    loop {
        input.update();
        game.update(&input);

        vblank.wait_for_vblank();
        object_controller.commit();
        game.commit();
    }
}
