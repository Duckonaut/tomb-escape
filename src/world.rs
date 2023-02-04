use agb::{
    display::{
        tiled::{
            InfiniteScrolledMap, RegularBackgroundSize, TileSet, TileSetting, Tiled0, VRamManager,
        },
        Priority,
    },
    fixnum::{num, Rect, Vector2D},
};
use alloc::{boxed::Box, rc::Rc, vec::Vec};

use crate::{tilemap, Number};

pub struct World<'t> {
    background: InfiniteScrolledMap<'t>,
    sections: InfiniteScrolledMap<'t>,
    section_generator: Rc<SectionIndexGenerator>,
    pub scroll: Number,
}

impl<'t> World<'t> {
    pub fn new(tileset: Rc<&'t TileSet<'t>>, tiled: &'t Tiled0, vram: &mut VRamManager) -> Self {
        let vblank = agb::interrupt::VBlank::get();
        let mut between_updates = || {
            vblank.wait_for_vblank();
        };

        let bg_tileset = tileset.clone();
        let mut background = InfiniteScrolledMap::new(
            tiled.background(Priority::P3, RegularBackgroundSize::Background64x32),
            Box::new(move |pos| {
                (
                    &bg_tileset,
                    TileSetting::from_raw(
                        *tilemap::BACKGROUND_MAP
                            .get((pos.x % tilemap::WIDTH + tilemap::WIDTH * pos.y) as usize)
                            .unwrap_or(&32),
                    ),
                )
            }),
        );

        let section_tileset = tileset.clone();
        let section_generator = Rc::new(SectionIndexGenerator::new(0));
        let for_sections = section_generator.clone();
        let mut sections = InfiniteScrolledMap::new(
            tiled.background(Priority::P2, RegularBackgroundSize::Background64x32),
            Box::new(move |pos| {
                let section_number = (pos.x / 64) as usize;
                let section_index = for_sections.get_at(section_number);

                (
                    &section_tileset,
                    TileSetting::from_raw(if pos.y < tilemap::HEIGHT {
                        *tilemap::SECTION_MAPS[section_index]
                            .get((pos.x % tilemap::WIDTH + tilemap::WIDTH * pos.y) as usize)
                            .unwrap_or(&32)
                    } else {
                        32
                    }),
                )
            }),
        );

        background.init(vram, Vector2D { x: 0, y: 0 }, &mut between_updates);
        sections.init(vram, Vector2D { x: 0, y: 0 }, &mut between_updates);

        background.show();
        sections.show();
        // foreground.show();

        background.commit(vram);
        sections.commit(vram);
        // foreground.commit(vram);

        Self {
            background,
            sections,
            section_generator,
            scroll: num!(0.),
        }
    }

    pub fn collides(&self, v: Vector2D<Number>) -> Option<Rect<Number>> {
        let factor: Number = Number::new(1) / Number::new(8);
        let adjusted_for_scroll = v + Vector2D {
            x: self.scroll,
            y: num!(0.),
        };
        let (x, y) = (
            (adjusted_for_scroll.x * factor).floor(),
            (adjusted_for_scroll.y * factor).floor(),
        );
        let section_number = (x / 64) as usize;

        if !(0..tilemap::HEIGHT).contains(&y) {
            return None;
        }
        let position = tilemap::WIDTH as usize * y as usize + (x % tilemap::WIDTH) as usize;
        let tile_main_section =
            tilemap::SECTION_MAPS[self.section_generator.get_at(section_number)][position];
        let tile_main_section_property = tilemap::TILE_TYPES[tile_main_section as usize];

        if tile_main_section_property == 1 {
            Some(Rect::new((x * 8, y * 8).into(), (8, 8).into()))
        } else {
            None
        }
    }

    pub fn update(&mut self) {
        self.scroll += num!(0.25);
    }

    pub fn clear(&mut self, vram: &mut VRamManager) {
        self.background.clear(vram);
        self.sections.clear(vram);
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        loop {
            match self.sections.set_pos(
                vram,
                Vector2D {
                    x: self.scroll.floor(),
                    y: 0,
                },
            ) {
                agb::display::tiled::PartialUpdateStatus::Done => break,
                agb::display::tiled::PartialUpdateStatus::Continue => (),
            }
        }

        self.background.commit(vram);
        self.sections.commit(vram);
    }
}

struct SectionIndexGenerator {
    seed: usize,
}

impl SectionIndexGenerator {
    fn new(seed: usize) -> Self {
        Self { seed }
    }

    fn get_at(&self, index: usize) -> usize {
        if index == 0 {
            return 0;
        }

        let mut index = index;
        let mut seed = self.seed;
        for _ in 0..index {
            seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            index = seed % 3;
        }
        1 + (index % (tilemap::SECTION_MAPS.len() - 1))
    }
}
