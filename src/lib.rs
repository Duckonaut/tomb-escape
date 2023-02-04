#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb::{
    display::{
        object::{Graphics, Tag},
        tiled::{InfiniteScrolledMap, RegularBackgroundSize, TileFormat, TileSet, TileSetting},
        Priority,
    },
    fixnum::{FixedNum, Vector2D},
    include_aseprite,
};
use alloc::{boxed::Box, vec, rc::Rc};
use game::Game;
use world::World;

mod entity;
mod game;
mod timer;
mod world;

mod tilemap {
    include!(concat!(env!("OUT_DIR"), "/tilemap.rs"));
}

agb::include_gfx!("gfx/tileset.toml");

const SPRITES: &Graphics = include_aseprite!("gfx/sprites.aseprite");
const UI_CARDS: &Graphics = include_aseprite!("gfx/ui_cards.aseprite");
const FONT: &Graphics = include_aseprite!("gfx/font.aseprite");
const COFFIN: &Graphics = include_aseprite!("gfx/coffin.aseprite");

const PLAYER_RUN: &Tag = SPRITES.tags().get("run");
const PLAYER_IDLE: &Tag = SPRITES.tags().get("idle");

const CLOCK_ROTATE: &Tag = SPRITES.tags().get("clock_rotate");
const CLOCK_DISAPPEAR: &Tag = SPRITES.tags().get("clock_disappear");

const COFFIN_OPEN: &Tag = COFFIN.tags().get("open");

const TIMER: &Tag = UI_CARDS.tags().get("timer");
const TITLE: &Tag = UI_CARDS.tags().get("title");
const GAME_OVER: &Tag = UI_CARDS.tags().get("game_over");
const PRESS_A_TO_START: &Tag = UI_CARDS.tags().get("a_to_start");

const DIGITS: &Tag = FONT.tags().get("digits");

type Number = FixedNum<8>;

pub fn main(mut gba: agb::Gba) -> ! {
    let vblank = agb::interrupt::VBlank::get();
    vblank.wait_for_vblank();
    let (background, mut vram) = gba.display.video.tiled0();

    vram.set_background_palettes(tileset::PALETTES);
    let tileset = TileSet::new(tileset::background.tiles, TileFormat::FourBpp);
    let tileset = Rc::new(&tileset);

    let world = World::new(tileset, &background, &mut vram);

    let object_controller = gba.display.object.get();

    let mut game = Game::new(&object_controller, world);

    let mut input = agb::input::ButtonController::new();

    loop {
        input.update();
        game.update(&input);

        vblank.wait_for_vblank();
        object_controller.commit();
        game.commit(&mut vram);
    }
}
