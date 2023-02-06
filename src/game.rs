use agb::{display::object::Object, fixnum::Vector2D};
use alloc::vec::Vec;

use crate::{
    entity::{Clock, ClockState, Player},
    timer::Timer,
    world::World,
    Number,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Start,
    Intro,
    Playing,
    GameOver,
}

pub struct Game<'o, 't> {
    pub object_controller: &'o agb::display::object::ObjectController,
    pub world: World<'t>,
    pub player: Player<'o>,
    pub clocks: Vec<Clock<'o>>,
    pub timer: Timer<'o>,
    pub state: GameState,
    pub title_cards: [Object<'o>; 2],
    pub press_start_card: Object<'o>,
    pub game_over_card: Object<'o>,
}

impl<'o, 't> Game<'o, 't> {
    pub fn new(
        object_controller: &'o agb::display::object::ObjectController,
        world: World<'t>,
    ) -> Self {
        let mut player = Player::new(object_controller);
        player.entity.object.hide();
        let timer = Timer::new(object_controller);
        let clocks = Vec::new();

        let mut title_card_left = object_controller.object_sprite(crate::gfx::TITLE.sprite(0));
        title_card_left.set_position((56, 32).into());
        title_card_left.hide();

        let mut title_card_right = object_controller.object_sprite(crate::gfx::TITLE.sprite(1));
        title_card_right.set_position((120, 32).into());
        title_card_right.hide();

        let mut press_start_card =
            object_controller.object_sprite(crate::gfx::PRESS_A_TO_START.sprite(0));
        press_start_card.set_position((88, 100).into());
        press_start_card.hide();

        let mut game_over_card = object_controller.object_sprite(crate::gfx::GAME_OVER.sprite(0));
        game_over_card.set_position((88, 64).into());
        game_over_card.hide();

        Self {
            object_controller,
            world,
            player,
            clocks,
            timer,
            state: GameState::Start,
            title_cards: [title_card_left, title_card_right],
            press_start_card,
            game_over_card,
        }
    }

    pub fn update(&mut self, input: &agb::input::ButtonController) {
        match self.state {
            GameState::Start => {
                if input.is_just_pressed(agb::input::Button::START)
                    || input.is_just_pressed(agb::input::Button::A)
                {
                    self.transition_to_state(GameState::Intro);
                }
            }
            GameState::Intro => {
                self.transition_to_state(GameState::Playing);
            }
            GameState::Playing => {
                self.player
                    .update(&self.world, &mut self.clocks, &mut self.timer, input);
                self.timer.update();
                self.world.update();
                for clock in self.clocks.iter_mut() {
                    clock.update(&self.world);
                }
                self.clocks
                    .retain(|clock| clock.state != ClockState::Destroy);
            }
            GameState::GameOver => {
                self.state = GameState::Start;
            }
        }
    }

    pub fn transition_to_state(&mut self, state: GameState) {
        self.state = state;
        match state {
            GameState::Playing => {
                self.world.start();
                self.populate_clocks();

                self.timer.show();
                self.timer.reset();
                self.player.entity.object.show();
                self.player.entity.object.set_position((64, 104).into());
                self.title_cards[0].hide();
                self.title_cards[1].hide();
                self.press_start_card.hide();
                self.game_over_card.hide();
            }
            GameState::Start => {
                self.timer.hide();
                self.world.stop();
                self.title_cards[0].show();
                self.title_cards[1].show();
                self.press_start_card.show();
            }
            GameState::GameOver => {
                self.timer.hide();
                self.player.entity.object.hide();
                self.game_over_card.show();
            }
            _ => {}
        }
    }

    fn populate_clocks(&mut self) {
        let section_generator = &self.world.section_generator.clone().unwrap();
        for i in 0..3 {
            let clock_positions = &crate::tilemap::CLOCK_POSITIONS[section_generator.get_at(i)];
            for clock_position in clock_positions.iter() {
                let clock = Clock::new(
                    self.object_controller,
                    (clock_position.0 - 8 + 512 * i as i32, clock_position.1 - 16).into(),
                );
                self.clocks.push(clock);
            }
        }
    }

    pub fn commit(&mut self) {
        self.world.commit();
    }
}
