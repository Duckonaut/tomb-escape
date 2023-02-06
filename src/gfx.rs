use agb::{
    display::object::{Graphics, Tag},
    include_aseprite,
};

pub const SPRITES: &Graphics = include_aseprite!("gfx/sprites.aseprite");
pub const BIG_SPRITES: &Graphics = include_aseprite!("gfx/big_sprites.aseprite");
pub const UI_CARDS: &Graphics = include_aseprite!("gfx/ui_cards.aseprite");

pub const FONT: &Graphics = include_aseprite!("gfx/font.aseprite");
pub const COFFIN: &Graphics = include_aseprite!("gfx/coffin.aseprite");

pub const PLAYER_RUN: &Tag = SPRITES.tags().get("run");
pub const PLAYER_IDLE: &Tag = SPRITES.tags().get("idle");
pub const PLAYER_JUMP_UP: &Tag = SPRITES.tags().get("jump_up");
pub const PLAYER_JUMP_MID: &Tag = SPRITES.tags().get("jump_mid");
pub const PLAYER_JUMP_FALL: &Tag = SPRITES.tags().get("jump_fall");

pub const CLOCK_ROTATE: &Tag = SPRITES.tags().get("clock_rotate");
pub const CLOCK_DISAPPEAR: &Tag = SPRITES.tags().get("clock_disappear");

pub const COFFIN_OPEN: &Tag = COFFIN.tags().get("open");

pub const TITLE: &Tag = BIG_SPRITES.tags().get("title");

pub const TIMER: &Tag = UI_CARDS.tags().get("timer");
pub const GAME_OVER: &Tag = UI_CARDS.tags().get("game_over");
pub const PRESS_A_TO_START: &Tag = UI_CARDS.tags().get("a_to_start");

pub const DIGITS: &Tag = FONT.tags().get("digits");
