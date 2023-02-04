use agb::display::object::Object;

use crate::{TIMER, DIGITS};

pub struct Timer<'o> {
    pub object_controller: &'o agb::display::object::ObjectController,
    pub timer_bg: Object<'o>,
    pub digits: [Object<'o>; 3],
    pub time: usize,
}

const DIGIT_XS: [u16; 3] = [32, 42, 50];

impl<'o> Timer<'o> {
    pub fn new(object_controller: &'o agb::display::object::ObjectController) -> Self {
        let mut timer_bg = object_controller.object_sprite(TIMER.sprite(0));
        timer_bg.set_position((2, 2).into());

        timer_bg.set_priority(agb::display::Priority::P1);

        let mut digits = [
            object_controller.object_sprite(DIGITS.sprite(1)),
            object_controller.object_sprite(DIGITS.sprite(0)),
            object_controller.object_sprite(DIGITS.sprite(0)),
        ];

        for digit in digits.iter_mut() {
            digit.set_priority(agb::display::Priority::P0);
            digit.set_y(14);
        }

        for (digit, x) in digits.iter_mut().zip(DIGIT_XS.iter()) {
            digit.set_x(*x + 2);
        }

        Self {
            object_controller,
            timer_bg,
            digits,
            time: 0,
        }
    }

    pub fn update(&mut self) {
        if self.time == 0 {
            return;
        }

        self.time -= 1;
        let seconds = self.time / 60;
        let seconds_ones = seconds % 10;
        let seconds_tens = ((seconds % 60) / 10) % 60;
        let minutes = seconds / 60;

        self.digits[0].set_sprite(self.object_controller.sprite(DIGITS.sprite(minutes)));
        self.digits[1].set_sprite(self.object_controller.sprite(DIGITS.sprite(seconds_tens)));
        self.digits[2].set_sprite(self.object_controller.sprite(DIGITS.sprite(seconds_ones)));
    }

    pub fn add_time(&mut self, time: usize) {
        self.time += time;
    }

    pub fn reset(&mut self) {
        self.time = 60 * 60;
    }
}



