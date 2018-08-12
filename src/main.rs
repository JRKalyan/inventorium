extern crate ggez;
use ggez::audio;
use ggez::conf;
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics;
use ggez::graphics::{Point2, Vector2, Rect, Color};
use ggez::nalgebra as na;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 600.0;
const UI_WIDTH: f32 = 200.0;

const PLAYER_HEALTH: f32 = 1.0;
const PLAYER_RADIUS: f32 = 8.0;

const MAIN_COLOR: Color = Color {
    r: 0.37,
    g: 1.0,
    b: 0.37,
    a: 1.0,
};

const BACK_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

struct Object {
    pub pos: Point2,
    pub vel: Vector2,
    pub health: f32,
    pub radius: f32,
    pub speed: f32,
}

impl Object {
    fn collides_with(&self, other: &Object) -> bool {
        na::distance(&self.pos, &other.pos) < self.radius + other.radius
    }
}

struct State {
    player: Object,
    projectiles: Vec<Object>,
    boundary: Rect,
    coin: Object,
    score: u32,
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // TODO restrict the frame rate
        let seconds = 0.1;
        self.move_player(seconds);

        // TODO handle movements
        // TODO handle collisions
        // TODO handle 
        // TODO handle UI changes
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        // draw the stage background
        graphics::set_color(ctx, BACK_COLOR);
        graphics::rectangle(
            ctx,
            graphics::DrawMode::Fill,
            self.boundary);

        graphics::set_color(ctx, MAIN_COLOR);
        graphics::circle(
            ctx,
            graphics::DrawMode::Fill,
            self.player.pos,
            PLAYER_RADIUS,
            0.1);
        graphics::present(ctx);
        std::thread::yield_now();
        Ok(())
    }
    // TODO keyup keydown callbacks
    fn key_down_event(&mut self, 
                      _ctx: &mut Context, 
                      keycode: Keycode, 
                      _keymod: Mod, 
                      repeat: bool) {
        // TODO change based on game state
        if repeat {
            return;
        }
        match keycode {
            Keycode::W => {
                self.player.vel.y -= 1.0;
            }
            Keycode::A => {
                self.player.vel.x -= 1.0;
            }
            Keycode::S => {
                self.player.vel.y += 1.0;
            }
            Keycode::D => {
                self.player.vel.x += 1.0;
            }
            _ => (),

        }
    }
    fn key_up_event(&mut self, 
                _ctx: &mut Context, 
                keycode: Keycode, 
                _keymod: Mod, _repeat: bool) {

        match keycode {
            Keycode::W => {
                self.player.vel.y += 1.0;
            }
            Keycode::A => {
                self.player.vel.x += 1.0;
            }
            Keycode::S => {
                self.player.vel.y -= 1.0;
            }
            Keycode::D => {
                self.player.vel.x -= 1.0;
            }
            _ => (),

        }
    }
}

fn main() {
    let c = conf::Conf {
        window_mode: conf::WindowMode {
            width: WINDOW_WIDTH as u32,
            height: WINDOW_HEIGHT as u32,
             borderless: false,
            fullscreen_type: conf::FullscreenType::Off,
            vsync: true,
            min_width: 0,
            max_width: 0,
            min_height: 0,
            max_height: 0,
        },
        ..Default::default()
    };
    let ctx = &mut Context::load_from_conf("Inventorium", "John Kalyan", c).unwrap();
    graphics::set_background_color(ctx, MAIN_COLOR);

    let padding = 10.0;
    let state = &mut State { 
        player: create_player(),
        projectiles: Vec::new(),
        boundary: Rect::new(UI_WIDTH + padding,
                            0.0 + padding,
                            WINDOW_WIDTH - UI_WIDTH - 2.0 * padding,
                            WINDOW_HEIGHT - 2.0 * padding),
        score: 0,
        coin: create_coin()

    };

    event::run(ctx, state).unwrap();
}


fn create_player() -> Object {
    // TODO spawn the player in a better spot
    Object {
        pos: Point2::new(WINDOW_WIDTH + UI_WIDTH + 30.0, WINDOW_HEIGHT / 2.0),
        vel: Vector2::new(0.0, 0.0),
        health: PLAYER_HEALTH,
        radius: PLAYER_RADIUS,
        speed: 100.0,
    }
}

fn create_coin() -> Object {
    Object {
        pos: Point2::new(WINDOW_WIDTH - 30.0, WINDOW_HEIGHT / 2.0),
        vel: Vector2::new(0.0, 0.0),
        health: 100.0, // redundant
        radius: 16.0,
        speed: 0.0,
    }
}

// TODO - create enemies according to level
fn create_enemies() -> Vec<Object> {
    Vec::new()
}

// TODO extract the player function
impl State {
    fn move_player(&mut self, dt: f32) {
        let left = self.boundary.x + self.player.radius;
        let right = self.boundary.x + self.boundary.w - self.player.radius;
        let top  = self.boundary.y + PLAYER_RADIUS;
        let bot = self.boundary.y + self.boundary.h - self.player.radius;
        move_object(&mut self.player, dt);
        if self.player.pos.x > right {
            self.player.pos.x = right;
        }
        if self.player.pos.y > bot {
            self.player.pos.y = bot;
        }
        if self.player.pos.x < left {
            self.player.pos.x = left;
        }
        if self.player.pos.y < top {
            self.player.pos.y = top;
        }
    }

    fn move_balls(&mut self, ball: &mut Object, dt: f32) {
        let left = self.boundary.x + ball.radius;
        let right = self.boundary.x + self.boundary.w - ball.radius;
        let top  = self.boundary.y + PLAYER_RADIUS;
        let bot = self.boundary.y + self.boundary.h - ball.radius;
        move_object(ball, dt);
        if ball.pos.x > right {
            ball.pos.x = right;
            ball.vel.x = -ball.vel.x;
        }
        if ball.pos.y > bot {
            ball.pos.y = bot;
            ball.vel.y = -ball.vel.y;
        }
        if ball.pos.x < left {
            ball.pos.x = left;
            ball.vel.x = -ball.vel.x;
        }
        if ball.pos.y < top {
            ball.pos.y = top;
            ball.vel.y = -ball.vel.y;
        }
    }

    fn shrink_boundary(&mut self) {
        let shrinkage = 10.0;
        self.boundary.x += shrinkage;
        self.boundary.y += shrinkage / 2.0;
        self.boundary.w -= 2.0 * shrinkage;
        self.boundary.h -= shrinkage;
    }

    fn random_location(&self) -> Point2 {
        // TODO
        //if self.player.pos < self.boundary.x + self.boundary.w / 2.0 {
        //}
        Point2::new(50.0,  50.0)
    }
}

fn move_object(obj: &mut Object, dt: f32) {
        let vel = obj.vel;
        //let mag = vel.norm_sq.sqrt() * obj.speed;
        // calculate the unit vector with the correct direction,
        // multiply it by the speed and then use that to move distance
        // according to how much time has passed (multiply the vector by time and move)
        obj.pos.x += vel.x * obj.speed * dt;
        obj.pos.y += vel.y * obj.speed * dt;
    }
