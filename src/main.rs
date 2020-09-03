use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::input::keyboard::KeyCode;
use ggez::input::mouse::MouseButton;


const WIN_W: f32 = 800.0;
const WIN_H: f32 = 600.0;


struct MainState {
  player: PlayerState,
  input: InputState,
  zombies: Vec<ZombieState>,
  bullets: Vec<BulletState>,
  spawner: SpawnerState,
  score: i32,
  dead: bool,
}

impl MainState {
  fn new() -> ggez::GameResult<MainState> {
    let s = MainState {
      player: PlayerState::new(),
      input: InputState::new(),
      zombies: vec![],
      bullets: vec![],
      spawner: SpawnerState::new(),
      score: 0,
      dead: false,
    };
    Ok(s)
  }

  fn reset(&mut self) {
    self.player = PlayerState::new();
    self.zombies = vec![];
    self.bullets = vec![];
    self.spawner = SpawnerState::new();
    self.score = 0;
    self.dead = false;
  }
}

struct InputState {
  left: bool,
  right: bool,
  up: bool,
  down: bool,
  shoot: bool,
  restart: bool,
  mouse_pos: na::Vector2<f32>,
}

impl InputState {
  fn new() -> Self {
    Self {
      left: false,
      right: false,
      up: false,
      down: false,
      shoot: false,
      restart: false,
      mouse_pos: na::Vector2::new(0.0, 0.0),
    }
  }
  fn move_dir(&self) -> na::Vector2<f32> {
    let x = (self.right as i32) - (self.left as i32);
    let y = (self.down as i32) - (self.up as i32);
    let md = na::Vector2::new(x as f32, y as f32);
    if md.norm() > 1.0 {
      md.normalize()
    } else {
      md 
    }
  }
  fn mouse_motion_event(&mut self, x: f32, y: f32) {
    self.mouse_pos = na::Vector2::new(x, y);
  }
  fn key_down_event(&mut self, key: KeyCode) {
    match key {
      KeyCode::A => self.left = true,
      KeyCode::D => self.right = true,
      KeyCode::W => self.up = true,
      KeyCode::S => self.down = true,
      KeyCode::Space => self.restart = true,
      _ => (),
    } 
  }
  fn key_up_event(&mut self, key: KeyCode) {
    match key {
      KeyCode::A => self.left = false,
      KeyCode::D => self.right = false,
      KeyCode::W => self.up = false,
      KeyCode::S => self.down = false,
      KeyCode::Space => self.restart = false,
      _ => (),
    } 
  }
  fn mouse_button_down_event(&mut self, button: MouseButton) {
    match button {
      MouseButton::Left => self.shoot = true,
      _ => (),
    }
  }
  fn mouse_button_up_event(&mut self, button: MouseButton) {
    match button {
      MouseButton::Left => self.shoot = false,
      _ => (),
    }
  }

}

struct PlayerState {
  pos: na::Vector2<f32>,
  angle: f32,
  cooldown: f32,
}

impl PlayerState {
  fn new() -> Self {
    Self {
      pos: na::Vector2::new(360.0, 360.0),
      angle: 0.0,
      cooldown: 0.0,
    } 
  }
  fn update(
    &mut self,
    input: &InputState, 
    bullets: &mut Vec<BulletState>,
    ctx: &ggez::Context,
    zombies: &Vec<ZombieState>,
    dead: &mut bool,
  ) -> ggez::GameResult
  {
    // Movement
    let ms = 300.0;
    self.pos += input.move_dir() * delta_time(ctx) * ms;
    self.angle = point_direction(input.mouse_pos, self.pos);
    // Shooting
    if self.cooldown > 0.0 {
      self.cooldown -= delta_time(ctx); 
    } else if input.shoot { 
      self.shoot(bullets);
    }
    // Getting hit
    let r = 16.0;
    for z in zombies.iter() {
      if (z.pos - self.pos).norm() < r {
        *dead = true;
      }
    }
    Ok(())
  }
  fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
    let r = 16.0;
    let color = graphics::Color::new(1.0, 0.1, 0.1, 1.0);
    draw_square(ctx, self.pos, self.angle, r, color)
  }
  fn shoot(&mut self, bullets: &mut Vec<BulletState>) {
    let b = BulletState::new(self.pos, self.angle);
    bullets.push(b);
    self.cooldown = 0.2;
  }
}

struct ZombieState {
  pos: na::Vector2<f32>,
  angle: f32,
  hp: i32,
  ms: f32,
}

impl ZombieState {
  fn new(pos: na::Vector2<f32>) -> Self {
    Self {
      pos, 
      angle: 0.0,
      hp: 2,
      ms: 200.0,
    } 
  }
  fn update(&mut self, ctx: &ggez::Context, player: &PlayerState) -> ggez::GameResult {
    let md = (player.pos - self.pos).normalize();
    self.pos += md * delta_time(ctx) * self.ms;
    let accel = 10.0;
    self.ms += accel * delta_time(ctx);
    self.angle = point_direction(na::Vector2::new(0.0, 0.0), md);
    Ok(())
  }
  fn take_damage(&mut self) {
    self.hp -= 1;
  }
  fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
    let r = 16.0;
    let color = graphics::Color::new(0.1, 1.0, 0.1, 1.0);
    draw_square(ctx, self.pos, self.angle, r, color)
  }
  fn is_dead(&self) -> bool {
    self.hp <= 0
  }
}

struct BulletState {
  pos: na::Vector2<f32>,
  angle: f32,
  hit: bool,
}

impl BulletState {
  fn new(pos: na::Vector2<f32>, angle: f32) -> Self {
    Self {pos, angle, hit: false}
  }

  fn update(&mut self, ctx: &ggez::Context, zombies: &mut Vec<ZombieState>) {
    if !self.hit {
      let ms = 800.0;
      self.pos.x += self.angle.cos() * ms * delta_time(ctx);
      self.pos.y += self.angle.sin() * ms * delta_time(ctx);
      let r = 16.0;
      for z in zombies.iter_mut() {
        if (z.pos - self.pos).norm() < r {
          z.take_damage();
          self.hit = true;
        }
      }
    }
  }

  fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
    let r = 4.0;
    let color = graphics::Color::new(1.0, 1.0, 0.1, 1.0);
    draw_square(ctx, self.pos, self.angle, r, color)
  }

  fn is_dead(&self) -> bool {
    self.pos.x < 0.0 || self.pos.x > WIN_W
    || self.pos.y < 0.0 || self.pos.y > WIN_H || self.hit
  }
}

fn point_direction(from: na::Vector2<f32>, to: na::Vector2<f32>) -> f32 {
  let dr = to - from;
  let bonus = if from.x <= to.x { std::f32::consts::PI } else { 0f32 };
  let angle = (dr.y/dr.x).atan() + bonus;
  angle
}

fn delta_time(ctx: &ggez::Context) -> f32 {
  ggez::timer::delta(ctx).as_secs_f32()
}

struct SpawnerState {
  cooldown: f32,
  freq: f32,
}

impl SpawnerState {
  fn new() -> Self {
    Self {
      cooldown: 0.0,
      freq: 1.0,
    }
  }

  fn spawn_position(&self) -> na::Vector2<f32> {
    let r: u8 = rand::random();
    match r {
      0..=63 => na::Vector2::new(0.0, 0.0),
      64..=127 => na::Vector2::new(0.0, WIN_H),
      128..=191 => na::Vector2::new(WIN_W, 0.0),
      _ => na::Vector2::new(WIN_W, WIN_H),
    }
  }

  fn spawn(&mut self, zombies: &mut Vec<ZombieState>, score: &mut i32) {
   let zomb = ZombieState::new(self.spawn_position()); 
   zombies.push(zomb);
   self.cooldown = 1.0/self.freq;
   *score += 1;
  }

  fn update(&mut self, ctx: &ggez::Context, zombies: &mut Vec<ZombieState>,
  score: &mut i32) {
    if self.cooldown <= 0.0 {
      self.spawn(zombies, score);
    } else {
      self.cooldown -= delta_time(ctx);
    }
    let accel = 0.1;
    self.freq += accel * delta_time(ctx);
  }
}

fn draw_square(
  ctx: &mut ggez::Context,
  pos: na::Vector2<f32>,
  angle: f32,
  radius: f32,
  color: graphics::Color,
) -> ggez::GameResult {
    // Draw a square
    let shape = graphics::Mesh::new_rectangle(
      ctx,
      graphics::DrawMode::fill(),
      graphics::Rect::new(-radius, -radius, radius*2.0, radius*2.0),
      color,
    )?;
    let draw_params = graphics::DrawParam::default()
      .rotation(angle)
      .dest(na::Point2::from(pos));
    graphics::draw(ctx, &shape, draw_params)?;

    Ok(())
}

fn draw_score(ctx: &mut ggez::Context, score: i32) -> ggez::GameResult {
    let mut text = graphics::Text::new(format!("Score: {}", score));
    text.set_font(Default::default(), graphics::Scale::uniform(25.0));
    let draw_params = graphics::DrawParam::default()
      .dest(na::Point2::new(10.0, 10.0));
    graphics::draw(ctx, &text, draw_params)?;
    Ok(())
}

fn draw_dead_message(ctx: &mut ggez::Context) -> ggez::GameResult {
    let mut text = graphics::Text::new("Space to Restart!");
    text.set_font(Default::default(), graphics::Scale::uniform(48.0));
    let draw_params = graphics::DrawParam::default()
      .dest(na::Point2::new(230.0, 280.0));
    graphics::draw(ctx, &text, draw_params)?;
    Ok(())
}

impl event::EventHandler for MainState {
  fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
    if !self.dead {
      self.player.update(&self.input, &mut self.bullets, ctx, &self.zombies,
        &mut self.dead)?;
      self.spawner.update(ctx, &mut self.zombies, &mut self.score);
      for b in self.bullets.iter_mut() {
        b.update(ctx, &mut self.zombies);
      }
      for z in self.zombies.iter_mut() {
        z.update(ctx, &self.player)?;
      }
      // Retention
      self.zombies.retain(|z: &ZombieState| !z.is_dead());
      self.bullets.retain(|b: &BulletState| !b.is_dead());
    } else {
      if self.input.restart {
        self.reset();
      }
    }
    Ok(())
  }

  fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
    graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
    self.player.draw(ctx)?;
    for z in self.zombies.iter() {
      z.draw(ctx)?;
    }
    for b in self.bullets.iter() {
      b.draw(ctx)?;
    }
    draw_score(ctx, self.score)?;
    if self.dead {
      draw_dead_message(ctx)?;
    }
    graphics::present(ctx)?;
    Ok(())
  }

  fn mouse_motion_event(
    &mut self,
    _ctx: &mut ggez::Context,
    x: f32,
    y: f32,
    _dx: f32,
    _dy: f32,
  ) {
    self.input.mouse_motion_event(x, y);
  }

  fn key_down_event(
    &mut self,
    _ctx: &mut ggez::Context,
    key: KeyCode,
    _keymods: ggez::input::keyboard::KeyMods,
    _repeat: bool,
  ) {
    self.input.key_down_event(key);
  }

  fn key_up_event(
    &mut self,
    _ctx: &mut ggez::Context,
    key: KeyCode,
    _keymods: ggez::input::keyboard::KeyMods,
  ) {
    self.input.key_up_event(key);
  }

  fn mouse_button_down_event(
    &mut self,
    _ctx: &mut ggez::Context,
    button: MouseButton,
    _x: f32,
    _y: f32
  ) {
    self.input.mouse_button_down_event(button);
  }

  fn mouse_button_up_event(
    &mut self,
    _ctx: &mut ggez::Context,
    button: MouseButton,
    _x: f32,
    _y: f32
  ) {
    self.input.mouse_button_up_event(button);
  }
}

pub fn main() -> ggez::GameResult { 
  let ws: ggez::conf::WindowSetup = Default::default();
  let cb = ggez::ContextBuilder::new("zombie_squares_rust_v1", "fjij")
    .window_setup(ws.title("Zombie Squares"));
  let (ctx, event_loop) = &mut cb.build()?;
  let state = &mut MainState::new()?;
  event::run(ctx, event_loop, state)
}
