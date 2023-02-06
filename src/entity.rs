use agb::{
    display::object::{Object, Tag},
    fixnum::{num, Rect, Vector2D},
};

use crate::{timer::Timer, world::World, Number};

extern crate alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

pub struct Entity<'o> {
    pub object_controller: &'o agb::display::object::ObjectController,
    pub position: Vector2D<Number>,
    pub velocity: Vector2D<Number>,
    pub object: Object<'o>,
    pub frame: usize,
    pub frame_counter: usize,
    pub animation: &'static Tag,
    pub animation_speed: usize,
    pub direction: Direction,
    pub collision_mask: Rect<Number>,
}

impl<'o> Entity<'o> {
    pub fn new(
        object_controller: &'o agb::display::object::ObjectController,
        collision_mask: Rect<Number>,
        animation: &'static Tag,
        animation_speed: usize,
    ) -> Self {
        let mut object =
            object_controller.object(object_controller.sprite(animation.animation_sprite(0)));
        object.set_priority(agb::display::Priority::P1);

        Self {
            object_controller,
            position: Vector2D::new(num!(0.), num!(0.)),
            velocity: Vector2D::new(num!(0.), num!(0.)),
            object,
            frame: 0,
            frame_counter: 0,
            animation,
            animation_speed,
            direction: Direction::Right,
            collision_mask,
        }
    }

    pub fn update(&mut self, world: &World) {
        self.update_position(world);
        self.object.set_position(self.position.floor());
        self.object.set_hflip(self.direction == Direction::Left);
        self.frame_counter += 1;
        if self.frame_counter >= self.animation_speed {
            self.frame_counter = 0;
            self.frame += 1;
            self.object.set_sprite(
                self.object_controller
                    .sprite(self.animation.animation_sprite(self.frame)),
            );
        }
    }

    pub fn update_position(&mut self, world: &World) -> Vector2D<Number> {
        let initial_position = self.position;

        let y = self.velocity.y.to_raw().signum();
        if y != 0 {
            let (delta, collided) =
                self.collision_in_direction((0, y).into(), self.velocity.y.abs(), |v| {
                    world.collides(v)
                });
            self.position += delta;
            if collided {
                self.velocity.y = 0.into();
            }
        }
        let x = self.velocity.x.to_raw().signum();
        if x != 0 {
            let (delta, collided) =
                self.collision_in_direction((x, 0).into(), self.velocity.x.abs(), |v| {
                    world.collides(v)
                });
            self.position += delta;
            if collided {
                self.velocity.x = 0.into();
            }
        }
        self.position.x -= world.scroll_velocity();

        self.position - initial_position
    }

    fn collider(&self) -> Rect<Number> {
        let mut number_collision: Rect<Number> = Rect::new(
            (
                self.collision_mask.position.x,
                self.collision_mask.position.y,
            )
                .into(),
            (self.collision_mask.size.x, self.collision_mask.size.y).into(),
        );
        number_collision.position =
            self.position + number_collision.position - number_collision.size / 2;
        number_collision
    }

    fn collision_in_direction(
        &mut self,
        direction: Vector2D<Number>,
        distance: Number,
        collision: impl Fn(Vector2D<Number>) -> Option<Rect<Number>>,
    ) -> (Vector2D<Number>, bool) {
        let number_collision = self.collider();

        let center_collision_point: Vector2D<Number> = number_collision.position
            + number_collision.size / 2
            + number_collision.size.hadamard(direction) / 2;

        let direction_transpose: Vector2D<Number> = direction.swap();
        let small = direction_transpose * Number::new(4) / 64;
        let triple_collider: [Vector2D<Number>; 2] = [
            center_collision_point + number_collision.size.hadamard(direction_transpose) / 2
                - small,
            center_collision_point - number_collision.size.hadamard(direction_transpose) / 2
                + small,
        ];

        let original_distance = direction * distance;
        let mut final_distance = original_distance;

        let mut has_collided = false;

        for edge_point in triple_collider {
            let point = edge_point + original_distance;
            if let Some(collider) = collision(point) {
                let center = collider.position + collider.size / 2;
                let edge = center - collider.size.hadamard(direction) / 2;
                let new_distance = (edge - center_collision_point)
                    .hadamard((direction.x.abs(), direction.y.abs()).into());
                if final_distance.manhattan_distance() > new_distance.manhattan_distance() {
                    final_distance = new_distance;
                }
                has_collided = true;
            }
        }

        (final_distance, has_collided)
    }

    pub fn set_animation(&mut self, animation: &'static Tag) {
        self.animation = animation;
        self.frame = 0;
        self.frame_counter = 0;
        self.object.set_sprite(
            self.object_controller
                .sprite(self.animation.animation_sprite(self.frame)),
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Animation {
    Idle,
    Run,
    JumpUp,
    JumpMid,
    JumpFall,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GroundState {
    Grounded,
    Airborne,
}

pub struct Player<'o> {
    animation: Animation,
    ground_state: GroundState,
    pub entity: Entity<'o>,
}

impl<'o> Player<'o> {
    pub fn new(object_controller: &'o agb::display::object::ObjectController) -> Self {
        Self {
            animation: Animation::Idle,
            ground_state: GroundState::Airborne,
            entity: Entity::new(
                object_controller,
                Rect::new((num!(8.), num!(9.)).into(), (num!(10.), num!(14.)).into()),
                crate::gfx::PLAYER_IDLE,
                8,
            ),
        }
    }

    pub fn update(
        &mut self,
        world: &World,
        clocks: &mut [Clock],
        timer: &mut Timer,
        input: &agb::input::ButtonController,
    ) {
        self.movement(world, input);

        self.update_animation();
        self.entity.update(world);

        for clock in clocks {
            if clock.state == ClockState::Active
                && clock.entity.collider().touches(self.entity.collider())
            {
                clock.disappear();
                timer.add_time(clock.time);
            }
        }

        if self.entity.position.x < num!(0.) {
            self.entity.position.x = num!(0.);
        }
    }

    fn movement(&mut self, world: &World, input: &agb::input::ButtonController) {
        if input.is_pressed(agb::input::Button::LEFT) {
            self.entity.velocity.x -= num!(0.125);
        }
        if input.is_pressed(agb::input::Button::RIGHT) {
            self.entity.velocity.x += num!(0.125);
        }
        if input.is_just_pressed(agb::input::Button::A)
            && self.ground_state == GroundState::Grounded
        {
            self.entity.velocity.y = num!(-4.);
        }
        if self.entity.velocity.x > num!(2.) {
            self.entity.velocity.x = num!(2.);
        }
        if self.entity.velocity.x < num!(-2.) {
            self.entity.velocity.x = num!(-2.);
        }

        self.entity.velocity.x *= num!(0.9);

        if self.entity.velocity.x.abs() < num!(0.0625) {
            self.entity.velocity.x = num!(0.);
        }

        if self
            .entity
            .collision_in_direction((0, 1).into(), num!(1.), |v| world.collides(v))
            .1
        {
            self.ground_state = GroundState::Grounded;
        } else {
            self.ground_state = GroundState::Airborne;
        }

        if self.ground_state == GroundState::Airborne {
            self.entity.velocity.y += num!(0.25);
        }

        if self.entity.velocity.y > num!(4.) {
            self.entity.velocity.y = num!(4.);
        }

        if self.entity.position.y > num!(160.) {
            self.entity.position = (num!(0.), num!(0.)).into();
            self.entity.velocity = (num!(0.), num!(0.)).into();
        }
    }

    pub fn update_animation(&mut self) {
        let old_animation = self.animation;
        if self.ground_state == GroundState::Airborne {
            match self.entity.velocity.y {
                y if y < num!(2.) => {
                    self.animation = Animation::JumpUp;
                }
                y if y > num!(2.) => {
                    self.animation = Animation::JumpFall;
                }
                _ => {
                    self.animation = Animation::JumpMid;
                }
            }
        } else if self.entity.velocity.x.abs() > num!(0.1) {
            self.animation = Animation::Run;
        } else {
            self.animation = Animation::Idle;
        }

        match self.entity.velocity.x {
            x if x > num!(0.) => {
                self.entity.direction = Direction::Right;
            }
            x if x < num!(0.) => {
                self.entity.direction = Direction::Left;
            }
            _ => {}
        }

        if old_animation != self.animation {
            match self.animation {
                Animation::Idle => {
                    self.entity.set_animation(crate::gfx::PLAYER_IDLE);
                    self.entity.animation_speed = 8;
                }
                Animation::Run => {
                    self.entity.set_animation(crate::gfx::PLAYER_RUN);
                    self.entity.animation_speed = 5;
                }
                Animation::JumpUp => {
                    self.entity.set_animation(crate::gfx::PLAYER_JUMP_UP);
                    self.entity.animation_speed = 8;
                }
                Animation::JumpMid => {
                    self.entity.set_animation(crate::gfx::PLAYER_JUMP_MID);
                    self.entity.animation_speed = 8;
                }
                Animation::JumpFall => {
                    self.entity.set_animation(crate::gfx::PLAYER_JUMP_FALL);
                    self.entity.animation_speed = 8;
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClockState {
    Incoming,
    Active,
    Disappearing,
    Destroy,
}

pub struct Clock<'o> {
    pub position: Vector2D<Number>,
    pub entity: Entity<'o>,
    pub state: ClockState,
    pub time: usize,
}

impl<'o> Clock<'o> {
    pub fn new(
        object_controller: &'o agb::display::object::ObjectController,
        position: Vector2D<Number>,
    ) -> Self {
        let mut entity = Entity::new(
            object_controller,
            Rect::new((num!(8.), num!(8.)).into(), (num!(16.), num!(16.)).into()),
            crate::gfx::CLOCK_ROTATE,
            8,
        );
        entity.object.hide();

        entity.position = position;

        Self {
            position,
            entity,
            state: ClockState::Incoming,
            time: 60 * 20,
        }
    }

    pub fn update(&mut self, world: &World) {
        let screen_x = self.position.x - world.scroll;
        if screen_x < num!(-16.) {
            self.state = ClockState::Destroy;
        }

        if screen_x > num!(240.) && self.state != ClockState::Incoming {
            self.entity.object.hide();
            self.state = ClockState::Incoming;
        } else if screen_x < num!(240.) && self.state == ClockState::Incoming {
            self.state = ClockState::Active;
            self.entity.object.show();
        }

        self.entity.update(world);

        if self.state == ClockState::Disappearing
            && self.entity.animation.sprites().len() == self.entity.frame
        {
            self.entity.object.hide();
            self.state = ClockState::Destroy;
        }
    }

    pub fn disappear(&mut self) {
        self.state = ClockState::Disappearing;
        self.entity.set_animation(crate::gfx::CLOCK_DISAPPEAR);
        self.entity.animation_speed = 6;
    }
}
