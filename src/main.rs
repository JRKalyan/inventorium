extern crate ggez;
extern crate rand;
use rand::{thread_rng, Rng};

use ggez::audio;
use ggez::conf;
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics;
use ggez::graphics::{Point2, Vector2, Rect, Color, Text, Font};
use ggez::nalgebra as na;
use ggez::timer;
use ggez::{Context, GameResult};

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 800.0;

const SHRINK: f32 = 20.0;

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

const COIN_COLOR: Color = Color {
    r: 1.0,
    g: 0.9,
    b: 0.17,
    a: 1.0,
};

const ENEMY_COLOR: Color = Color {
    r: 1.0,
    g: 0.38,
    b: 0.227,
    a: 1.0,
};

struct Object {
    pub pos: Point2,
    pub vel: Vector2,
    pub alive: bool,
    pub radius: f32,
    pub speed: f32,
}

impl Object {
    fn collides_with(&self, other: &Object) -> bool {
        na::distance(&self.pos, &other.pos) < self.radius + other.radius
    }
}

/*
 * IDEA: give the player a gun and if they hit the wall then it shrinks but if they hit an
 * enemy then it kills them
 * TODO - fix the shrinking unevenly.
 */

struct State {
    player: Object,
    enemies: Vec<Object>,
    shots: Vec<Object>,
    boundary: Rect,
    coin: Object,
    score: u32,
    shrink_target: Option<f32>,
    ammo: u32,
    angle: Vector2,
    dirty: bool,
    mousex: f32,
    mousey: f32,
    score_display: Text,
    ammo_display: Text,
    over_display: Text,
    font: Font,
    over: bool,
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.boundary.h <= self.player.radius || 
            self.boundary.w <= self.player.radius {
            self.over = true;
            return Ok(());
        }
        let dt = timer::get_delta(ctx);
        let seconds = dt.subsec_nanos() as f32;
        let seconds = seconds / 1000000000.0;
        self.move_player(seconds);
        self.update_angle();
        for ball in &mut self.enemies {
            move_ball(ball, seconds, self.boundary);
        }
        for shot in &mut self.shots {
            if move_shot(shot, seconds, self.boundary) {
                self.shrink_target = match self.shrink_target {
                    Some(t) => Some(t + SHRINK),
                    None => Some(SHRINK)
                };
            }
        }

        if self.player.collides_with(&self.coin) {
            self.score += 1;
            self.ammo += 1;
            self.dirty = true;
            self.spawn_enemy();
            self.coin.pos = self.random_location(self.coin.radius);
            clamp_object(&mut self.coin, self.boundary);
        }
        for enemy in &mut self.enemies {
            if enemy.collides_with(&self.player) {
                self.shrink_target = match self.shrink_target {
                    Some(t) => Some(t + SHRINK),
                    None => Some(SHRINK)
                };
                enemy.alive = false;
            }
            for shot in &mut self.shots {
                if enemy.collides_with(shot) {
                    enemy.alive = false;
                    shot.alive = false;
                    self.score += 1;
                    self.dirty = true;
                }
            }
        }
        self.shrink_boundary();
        self.enemies.retain(|e| {e.alive});
        self.shots.retain(|s| {s.alive});

        // update ui
        if self.dirty {
            let score_str = format!("score: {}", self.score);
            let ammo_str = format!("ammo: {}", self.ammo);

            self.score_display = Text::new(ctx, &score_str, &self.font).unwrap();
            self.ammo_display = Text::new(ctx, &ammo_str, &self.font).unwrap();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::set_color(ctx, BACK_COLOR);
        if !self.over {
            graphics::rectangle(
                ctx,
                graphics::DrawMode::Fill,
                self.boundary);
            draw_obj(&self.player, ctx, MAIN_COLOR);
            self.draw_turret(ctx);
            draw_obj(&self.coin, ctx, COIN_COLOR);
            for enemy in &self.enemies {
                draw_obj(enemy, ctx, ENEMY_COLOR);
            }
            for shot in &self.shots {
                draw_obj(shot, ctx, MAIN_COLOR);
            }
        }
        else {
            graphics::set_color(ctx, MAIN_COLOR);
            graphics::rectangle(
                ctx,
                graphics::DrawMode::Fill,
                self.boundary);
        }

        // draw ui
        graphics::set_color(ctx, BACK_COLOR);
        let score_location = Point2::new(20.0, 3.0);
        let ammo_location = Point2::new(120.0, 3.0);
        let over_location = Point2::new(WINDOW_WIDTH / 2.0 - 170.0, WINDOW_HEIGHT / 2.0);
        if self.over {
            graphics::draw(ctx, &self.over_display, over_location, 0.0);
        }
        graphics::draw(ctx, &self.score_display, score_location, 0.0);
        graphics::draw(ctx, &self.ammo_display, ammo_location, 0.0);


        graphics::present(ctx);
        std::thread::yield_now();
        Ok(())
    }
    // TODO keyup keydown callbacks
    fn key_down_event(&mut self, 
                      ctx: &mut Context, 
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
            Keycode::Space => {
                if (self.ammo > 0) {
                    self.fire_shot();
                    self.ammo -= 1;
                    self.dirty = true;
                }
            }
            Keycode::Return => {
                if self.over {
                    self.reset(ctx);
                }
            }
            Keycode::Escape => {
                ctx.quit().unwrap();
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
            Keycode::A => { self.player.vel.x += 1.0;
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
    fn mouse_motion_event(&mut self,
                          _ctx: &mut Context,
                          _state: ggez::event::MouseState,
                          x: i32,
                          y: i32,
                          _xrel: i32,
                          _yrel: i32) {
        self.mousex = x as f32;
        self.mousey = y as f32;
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
        window_setup: conf::WindowSetup {
            title: "Not another snake variant!".to_owned(),
            icon: "".to_owned(),
            resizable: false,
            allow_highdpi: true,
            samples: conf::NumSamples::One,

        },
        ..Default::default()
    };
    let ctx = &mut Context::load_from_conf("LD42", "John Kalyan", c).unwrap();
    graphics::set_background_color(ctx, MAIN_COLOR);
    let font = graphics::Font::default_font().unwrap();
    let state = &mut State::new(ctx, font).unwrap();
    event::run(ctx, state).unwrap();
}


fn create_player() -> Object {
    Object {
        pos: Point2::new(30.0, WINDOW_HEIGHT / 2.0),
        vel: Vector2::new(0.0, 0.0),
        alive: true,
        radius: PLAYER_RADIUS,
        speed: 300.0,
    }
}

fn create_coin() -> Object {
    Object {
        pos: Point2::new(WINDOW_WIDTH - 30.0, WINDOW_HEIGHT / 2.0),
        vel: Vector2::new(0.0, 0.0),
        alive: true,
        radius: 6.0,
        speed: 0.0,
    }
}

// TODO extract the player function
impl State {
    fn move_player(&mut self, dt: f32) {
        move_object(&mut self.player, dt);
        clamp_object(&mut self.player, self.boundary);
    }

    fn shrink_boundary(&mut self) {
        if let Some(t) = self.shrink_target {
            if t > 0.0 {
                let shrinkage = 1.0;
                self.boundary.x += shrinkage;
                self.boundary.y += shrinkage;
                self.boundary.w -= 2.0 * shrinkage;
                self.boundary.h -= 2.0 * shrinkage;
                self.shrink_target = Some(t - shrinkage);
                clamp_object(&mut self.coin, self.boundary);
            }
        }
    }

    fn random_location(&self, rad: f32) -> Point2 {
        let mut rng = thread_rng();
        let mut startx = self.boundary.x + rad;
        if self.player.pos.x < self.boundary.x + self.boundary.w / 2.0 {
            startx += self.boundary.w / 2.0;
        }
        let randx: f32 = rng.gen_range(startx, startx + self.boundary.w / 2.0);
        let starty: f32 = self.boundary.y + rad;
        let randy = rng.gen_range(starty, starty + self.boundary.h);
        Point2::new(randx,  randy)
    }

    fn spawn_enemy(&mut self) {
        let mut rng = thread_rng();
        let radius = 8.0;
        let velx = rng.gen_range(0.2, 0.5);
        let vely = rng.gen_range(0.2, 0.5);
        let vel = Vector2::new(velx, vely);
        let enemy = Object {
            pos: self.random_location(radius),
            alive: true,
            radius: radius,
            speed: rng.gen_range(250.0,400.0),
            vel: vel,
        };
        self.enemies.push(enemy);
    }

    fn draw_turret(&self, ctx: &mut Context) {
        graphics::set_color(ctx, MAIN_COLOR);
        let point = self.angle * 20.0;
        let mut point =  Point2::new(point.x, point.y);
        point.x += self.player.pos.x;
        point.y += self.player.pos.y;
        graphics::circle(
            ctx,
            graphics::DrawMode::Fill,
            Point2::new(point.x, point.y),
            4.0,
            0.01).unwrap();
    }

    fn update_angle(&mut self) {
        let diff = Vector2::new(self.mousex, self.mousey) - 
            Vector2::new(self.player.pos.x, self.player.pos.y);
        self.angle = diff.normalize();
    }

    fn fire_shot(&mut self) {
        let spawn = self.angle * 20.0;
        let spawn = Point2::new(spawn.x + self.player.pos.x, spawn.y + self.player.pos.y);
        let shot = Object {
            radius: 3.0,
            vel: self.angle,
            speed: 500.0,
            alive: true,
            pos: spawn
        };
        self.shots.push(shot);
    }

    fn new(ctx: &mut Context, font: Font) -> GameResult<State> {
        let score_display = Text::new(ctx, "score:", &font)?;
        let ammo_display = Text::new(ctx, "ammo:", &font)?;
        let over_display = Text::new(ctx, "game over (ENTER to play again, ESC to quit)", &font)?;
        let padding = 20.0;
        let s = State { 
            player: create_player(),
            enemies: Vec::new(),
            shots: Vec::new(),
            boundary: Rect::new(padding,
                                padding, 
                                WINDOW_WIDTH - 2.0 * padding,
                                WINDOW_HEIGHT - 2.0 * padding),
            score: 0,
            coin: create_coin(),
            shrink_target: None,
            ammo: 0,
            angle: Vector2::new(1.0, 0.0),
            dirty: true,
            mousex: 0.0,
            mousey: 0.0,
            score_display: score_display,
            ammo_display: ammo_display,
            over_display: over_display,
            font: font,
            over: false
        };
        Ok(s)
    }

    fn reset(&mut self, ctx: &mut Context) {
        let padding = 20.0;
        self.player = create_player();
        self.enemies =  Vec::new();
        self.shots = Vec::new();
        self.boundary = Rect::new(padding,
                                  padding,
                                  WINDOW_WIDTH - 2.0 * padding,
                                  WINDOW_HEIGHT - 2.0 * padding);
        self.over = false;
        self.score = 0;
        self.coin = create_coin();
        self.shrink_target = None;
        self.ammo = 0;
        self.dirty = true;
    }
}

fn move_ball(ball: &mut Object, dt: f32, boundary: Rect) {
    let left = boundary.x + ball.radius;
    let right = boundary.x + boundary.w - ball.radius;
    let top  = boundary.y + ball.radius;
    let bot = boundary.y + boundary.h - ball.radius;
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

fn move_shot(ball: &mut Object, dt: f32, boundary: Rect) -> bool {
    let left = boundary.x + ball.radius;
    let right = boundary.x + boundary.w - ball.radius;
    let top  = boundary.y + ball.radius;
    let bot = boundary.y + boundary.h - ball.radius;
    move_object(ball, dt);
    if ball.pos.x > right || ball.pos.x < left || ball.pos.y < top || ball.pos.y > bot {
        ball.alive = false;
        return true;
    }
    return false;
}



fn draw_obj(obj: &Object, ctx: &mut Context, color: Color) {
    graphics::set_color(ctx, color);
    graphics::circle(
        ctx,
        graphics::DrawMode::Fill,
        obj.pos,
        obj.radius,
        0.01);
}

fn clamp_object(obj: &mut Object, boundary: Rect) {
        let left = boundary.x + obj.radius;
        let right = boundary.x + boundary.w - obj.radius;
        let top  = boundary.y + PLAYER_RADIUS;
        let bot = boundary.y + boundary.h - obj.radius;
        if obj.pos.x > right {
            obj.pos.x = right;
        }
        if obj.pos.y > bot {
            obj.pos.y = bot;
        }
        if obj.pos.x < left {
            obj.pos.x = left;
        }
        if obj.pos.y < top {
            obj.pos.y = top;
        }
}

fn move_object(obj: &mut Object, dt: f32) {
        let mut vel = obj.vel;
        println!("x{} y{}", vel.x, vel.y);
        if (vel.x != 0.0 || vel.y != 0.0) {
            vel = vel.normalize();
        }
        println!("x{} y{}", vel.x, vel.y);
        obj.pos.x += vel.x * obj.speed * dt;
        obj.pos.y += vel.y * obj.speed * dt;
}
