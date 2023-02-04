use agb::display::{object::Object, tiled::VRamManager};

use crate::{entity::Player, timer::Timer, world::World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Start,
    Intro,
    Playing,
    GameOver,
}

struct GameplayEntities<'o> {
    player: Player<'o>,
    timer: Timer<'o>,
}

struct IntroEntities<'o> {
    coffin: Object<'o>,
    coffin_animation_timer: usize,
    coffin_animation_frame: usize,
}

pub struct Game<'o, 't> {
    pub object_controller: &'o agb::display::object::ObjectController,
    pub world: World<'t>,
    pub player: Player<'o>,
    pub timer: Timer<'o>,
    pub state: GameState,
}

impl<'o, 't> Game<'o, 't> {
    pub fn new(
        object_controller: &'o agb::display::object::ObjectController,
        world: World<'t>,
    ) -> Self {
        let mut player = Player::new(object_controller);
        player.entity.object.hide();
        let timer = Timer::new(object_controller);
        Self {
            object_controller,
            world,
            player,
            timer,
            state: GameState::Start,
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
                self.player.update(&self.world, input);
                self.timer.update();
                self.world.update();
            }
            GameState::GameOver => {
                self.state = GameState::Start;
            }
        }
    }

    fn transition_to_state(&mut self, state: GameState) {
        self.state = state;
        if state == GameState::Playing {
            self.timer.reset();
            self.player.entity.object.show();
            self.player.entity.object.set_position((64, 104).into());
        }
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.world.commit(vram);
    }
}
