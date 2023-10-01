use macroquad::{prelude::*, rand::RandomRange};

#[macroquad::main("limspce")]
async fn main() {
    let scr_w = screen_width();
    let scr_h = screen_height();
    let hscr_w = scr_w/2.0;
    let hscr_h = scr_h/2.0;
    let bg_col = Color::from_hex(0x141414);
    let mut g = Game::default();
    loop {
        clear_background(bg_col);
        match &g.state {
            GameState::Play => {
                Game::update(&mut g);
                Game::draw(&g);
                if is_key_pressed(KeyCode::Escape) {g.state = GameState::Pause;}
                g.elapsed_time += get_frame_time();
            },
            GameState::Pause => {
                Game::draw(&g);
                if is_key_pressed(KeyCode::Escape) {g.state = GameState::Play;}
            },
            GameState::GameOver => {
                Game::update(&mut g);
                if !g.on_plat {
                    g.player.pos.y -= 120.0 * 1.0 * get_frame_time();
                    g.player.col1.a -= 1.0 * get_frame_time();
                    g.player.col2.a -= 1.0 * get_frame_time();
                    g.player.size = f32::max(g.player.size - 4.0 * get_frame_time(), 0.0);
                    if g.frame % 50 == 0 {g.player.health = (g.player.health - 1).max(0)};
                } else {
                    g.player.col1.a = 0.0;
                    g.player.col2.a = 0.0;        
                }

                Game::draw(&g);
                draw_text("GAME OVER", 0.0, hscr_h, 200.0, WHITE);
                draw_text("PRESS 'R' TO RESTART", 10.0, hscr_h + 70.0, 30.0, WHITE);
                draw_text(format!("YOU LASTED {:.2} SECONDS", g.elapsed_time).as_str(), 10.0, hscr_h + 40.0, 50.0, WHITE);
                if is_key_pressed(KeyCode::R) {
                    g = Game::default();
                    g.state = GameState::Play;
                }
            },
            GameState::MainMenu => {
                draw_text("PRESS 'SPC' TO PLAY", 10.0, hscr_h, 92.0, WHITE);
                if is_key_pressed(KeyCode::Space) {g.state = GameState::Play;}
                if is_key_pressed(KeyCode::Escape) {break;}
            }
        }
        next_frame().await
    }   
}

fn p_on_plat(p_pos: Vec2, p_size: f32, plat_pos: Vec2, plat_hsize: Vec2) -> bool {
    let plat_min = Vec2::new(plat_pos.x - plat_hsize.x, plat_pos.y - plat_hsize.y);
    let plat_max = Vec2::new(plat_pos.x + plat_hsize.x, plat_pos.y + plat_hsize.y);

    plat_min.x - p_size <= p_pos.x && p_pos.x <= plat_max.x + p_size &&
    plat_min.y <= p_pos.y-p_size && p_pos.y-p_size <= plat_max.y
}
fn draw_enemy(pos: Vec2, r: f32, color_a: Color, color_b: Color) {
    let rot_mat = rot_mat(r);
    let tri_a = Vec2::new(0.0, 1.0) * 0.75;
    let tri_b = Vec2::new(-0.5, -0.25) * 0.75;
    let tri_c = Vec2::new(0.0, 0.0) * 0.75;
    let tri_d = Vec2::new(0.5, -0.25) * 0.75;
    draw_triangle(rot_mat * tri_a * 25.0 + pos, rot_mat * tri_b * 25.0 + pos, rot_mat * tri_c * 25.0 + pos, color_b);
    draw_triangle(rot_mat * tri_a * 25.0 + pos, rot_mat * tri_d * 25.0 + pos, rot_mat * tri_c * 25.0 + pos, color_b);
    draw_triangle(rot_mat * tri_a * 20.0 + pos, rot_mat * tri_b * 20.0 + pos, rot_mat * tri_c * 20.0 + pos, color_a);
    draw_triangle(rot_mat * tri_a * 20.0 + pos, rot_mat * tri_d * 20.0 + pos, rot_mat * tri_c * 20.0 + pos, color_a);
}
fn draw_platform(pos: Vec2, hx: f32, hy: f32, color_a: Color, color_b: Color) {
    draw_rec(pos + Vec2::new(0.0,15.0), hx, hy, color_b);
    draw_rec(pos, hx, hy, color_a);
}
fn draw_rec(pos: Vec2, hx: f32, hy: f32, color: Color) {
    draw_rectangle(pos.x-hx, pos.y-hy, 2.0*hx, 2.0*hy, color);
    draw_rectangle_lines(pos.x-hx, pos.y-hy, 2.0*hx, 2.0*hy, 3.0, BLACK);
}
fn draw_cir(pos: Vec2, r: f32, color_a: Color, color_b: Color) {
    draw_circle(pos.x, pos.y, r, color_a);
    draw_circle_lines(pos.x, pos.y, r+1.0, 2.0, color_b);
}
fn rot_mat(theta: f32) -> Mat2 {
    Mat2 {
        x_axis: Vec2::new(theta.cos(), -theta.sin()),
        y_axis: Vec2::new(theta.sin(), theta.cos()),
    }
}
fn circle_overlap(c1: Vec2, r1: f32, c2: Vec2, r2: f32) -> bool {
    Vec2::distance(c1, c2) < r1 + r2
}

struct Player {
    pos: Vec2,
    size: f32,
    speed: f32,
    health: i32,
    blank: i32,
    col1: Color,
    col2: Color,
}

enum GameState {
    MainMenu,
    Play,
    Pause,
    GameOver,
}
struct Enemy {
    pos: Vec2,
    scale: f32,
    proj: Vec<(Vec2, Vec2, f32)>,
    rdm: f32,
    rdmf: i32,
}

struct Platform {
    pos: Vec2,
    hsize: Vec2,
    lerp_pos: Lerp, 
    lerp_size: Lerp,
}

struct Lerp {
    p0: Vec2,
    p1: Vec2,
    t0: f64,
    speed: f32,
}

impl Lerp {
    fn s(&self, current_time: f64) -> f32 {
        let current_time = current_time as f32;
        let start_time = self.t0 as f32;
        let ratio = (current_time - start_time) / Vec2::distance(self.p0, self.p1);
        let ratio = ratio * self.speed;
        f32::min(ratio, 1.0)
    }
}
struct Game {
    frame: i32,
    elapsed_time: f32,
    state: GameState,
    on_plat: bool,
    player: Player,
    platform: Platform,
    enemies: Vec<Enemy>,
    blanks: Vec<(Vec2, Lerp, f32, Color)>,
}

impl Default for Game {
    fn default() -> Self {
        let player = Player {
            pos: Vec2::new(0.0, 0.0),
            size: 7.0,
            speed: 150.0,
            health: 3,
            blank: 3,
            col1: Color::from_hex(0x5990de),
            col2: Color::from_hex(0x93abcd),
        };
        let platform = Platform {
            pos: Vec2::new(0.0, 0.0),
            hsize: Vec2::new(200.0, 200.0),
            lerp_pos: Lerp { p0: Vec2::new(0.0, 0.0), p1: Vec2::new(0.0, 70.0), t0: 0.0,speed: 20.0,}, 
            lerp_size: Lerp { p0: Vec2::new(200.0, 200.0), p1: Vec2::new(200.0, 200.0), t0: 0.0, speed: 20.0,},
        };
        let mut enemies: Vec<Enemy> = Vec::new();
        let e = Enemy {
            pos: Vec2::new(45.0, 100.0),
            scale: 150.0,
            proj: Vec::new(),
            rdm: RandomRange::gen_range(0.0, 100.0),
            rdmf: RandomRange::gen_range(1, 5),
        };
        enemies.push(e);

        Game { 
            frame: 1,
            elapsed_time: 0.0,
            state: GameState::MainMenu,
            on_plat: true,
            player: player,
            platform: platform,
            enemies: enemies,
            blanks: Vec::new(),
        }
    }
}

impl Game {
    fn update(g: &mut Self) {
        let scr_w = screen_width();
        let scr_h = screen_height();
        let hscr_w = scr_w/2.0;
        let hscr_h = scr_h/2.0;
        let offscreen = |pos: Vec2| { 
            -hscr_w > pos.x || pos.x > hscr_w ||
            -hscr_h > pos.y || pos.y > hscr_h
        };
        g.frame += 1;
        g.on_plat = p_on_plat(g.player.pos, g.player.size, g.platform.pos, g.platform.hsize);

        //Platform
        let s = g.platform.lerp_pos.s(g.elapsed_time as f64);
        let n_plat_pos = Vec2::lerp(g.platform.lerp_pos.p0, g.platform.lerp_pos.p1, s);
        let d_plat_pos = n_plat_pos - g.platform.pos;
        g.platform.pos = n_plat_pos;
        if s == 1.0 && g.frame % 500 == 0 {
            g.platform.lerp_pos.p1 = Vec2::new(RandomRange::gen_range(-hscr_w + g.platform.hsize.x, hscr_w - g.platform.hsize.x), RandomRange::gen_range(-hscr_h + g.platform.hsize.y, hscr_h - g.platform.hsize.y));
            g.platform.lerp_pos.p0 = g.platform.pos;
            g.platform.lerp_pos.t0 = g.elapsed_time as f64;
        }
        
        if g.frame % 1000 == 0 {
            g.platform.lerp_size.p0 = g.platform.hsize;
            g.platform.lerp_size.p1 = g.platform.hsize * 0.75;
            g.platform.lerp_size.t0 = g.elapsed_time as f64;
        }
        let s = g.platform.lerp_size.s(g.elapsed_time as f64);
        g.platform.hsize = Vec2::lerp(g.platform.lerp_size.p0, g.platform.lerp_size.p1, s);

        //Player:
        let mut dx = 0.0;
        let mut dy = 0.0;
        if g.on_plat {
            if is_key_down(KeyCode::A) {dx += 1.0 * g.player.speed * get_frame_time();}
            if is_key_down(KeyCode::D) {dx += -1.0 * g.player.speed * get_frame_time();} 
            if is_key_down(KeyCode::S) {dy += -1.0 * g.player.speed * get_frame_time();}
            if is_key_down(KeyCode::W) {dy += 1.0 * g.player.speed * get_frame_time();}
        } else {
            g.player.pos.y -= 120.0 * 1.0 * get_frame_time();
            g.player.col1.a -= 1.0 * get_frame_time();
            g.player.col2.a -= 1.0 * get_frame_time();
            g.player.size = f32::max(g.player.size - 4.0 * get_frame_time(), 0.0);
            if g.frame % 30 == 0 {g.player.health = (g.player.health - 1).max(0)};
        }

        g.player.pos += Vec2::new(dx, dy);
        g.player.pos += d_plat_pos; //Parent player to platform

        if g.frame % 1000 == 0 {
            g.player.blank = i32::min(3, g.player.blank + 1);
        }
        if is_key_pressed(KeyCode::Space) && g.player.blank > 0 {
            g.player.blank -= 1;
            let blank = (g.player.pos, Lerp{p0:Vec2::splat(1.0), p1:Vec2::splat(100.0), t0:g.elapsed_time as f64, speed:400.0}, 1.0, WHITE);
            g.blanks.push(blank);
        }
        if g.player.health <= 0 {g.state = GameState::GameOver;}

        // Blanks:
        let mut blank_rem: Vec<usize> = Vec::new();
        for (i, b) in g.blanks.iter_mut().enumerate() {
            let s = b.1.s(g.elapsed_time as f64);
            b.3.a = 1.0 - s; 
            if s == 1.0 {blank_rem.push(i);}
            b.2 = Vec2::lerp(b.1.p0, b.1.p1, s).x;
        }
        while blank_rem.len() > 0 {
            g.blanks.swap_remove(blank_rem.pop().unwrap());
        }
        
        //Player proj collision:
        let mut proj_rem: Vec<(usize, usize)> = Vec::new();
        for (i, e) in g.enemies.iter_mut().enumerate() {
            for (j, p) in e.proj.iter_mut().enumerate() {
                if circle_overlap(g.player.pos, g.player.size, p.0, 3.0) {
                    proj_rem.push((i,j));
                    g.player.health -= 1;
                    continue;
                }
                for b in g.blanks.iter() {
                    if circle_overlap(b.0, b.2, p.0, 3.0) {
                        proj_rem.push((i,j));
                    }
                }
            }
        }
        while proj_rem.len() > 0 {
            let rem = proj_rem.pop().unwrap();
            g.enemies.get_mut(rem.0).unwrap().proj.swap_remove(rem.1);
        }

        // Enemies:
        if g.frame % 1100 == 0 {
            let e = Enemy {
                pos: Vec2::new(RandomRange::gen_range(-1.0, 1.0), RandomRange::gen_range(-1.0, 1.0)),
                scale: RandomRange::gen_range(175.0, 300.0),
                proj: Vec::new(),
                rdm: RandomRange::gen_range(0.0, 100.0),
                rdmf: RandomRange::gen_range(1, 5),
            };
            g.enemies.push(e);
        }
        for e in g.enemies.iter_mut() {
            let time = get_time() + e.rdm as f64;
            let enemy_theta = time.sin()+ (time/2.0).cos();
            e.pos = (rot_mat(enemy_theta as f32 * get_frame_time()) * (e.pos - g.platform.pos).normalize()) * e.scale + g.platform.pos;

            if g.frame % (100 + e.rdmf) == 0 || g.frame % (110 + e.rdmf) == 0 || g.frame % (120 + e.rdmf) == 0 {
                e.proj.push((e.pos, -(e.pos - g.player.pos).normalize(), 20.0));
            }
            for p in e.proj.iter_mut() {
                p.2 = p.2 + 4.0;
                p.0 = p.0 + (p.1 * p.2 * get_frame_time());
            }
            e.proj.retain(|&pos| !offscreen(pos.0)); //cull offscreen
        }
    }

    fn draw(g: &Self) {
        let grid_col = Color::from_hex(0x1e3821);
        let platform1_col = Color::from_hex(0x605569);
        let platform2_col = Color::from_hex(0x3b3342);
        let scr_w = screen_width();
        let scr_h = screen_height();
        let hscr_w = scr_w/2.0;
        let hscr_h = scr_h/2.0;
        let pos_to_world = |pos: Vec2| Vec2::new(scr_w-(hscr_w+pos.x), scr_h-(hscr_h+pos.y));
        // Draw:
        for i in 0..(scr_h/30.0) as i32 +1 {
            draw_line(0.0, i as f32*30.0, scr_w, i as f32 *30.0, 1.0, grid_col);
        }
        for i in 0..(scr_w/30.0)  as i32 +1 {
            draw_line(i as f32*30.0, 0.0, i as f32*30.0, scr_h, 1.0, grid_col);
        }
        draw_platform(pos_to_world(g.platform.pos), g.platform.hsize.x, g.platform.hsize.y, platform1_col, platform2_col);
        draw_cir(pos_to_world(g.player.pos), g.player.size, g.player.col1, g.player.col2);
        for e in g.enemies.iter() {
            for p in e.proj.iter() {
                draw_cir(pos_to_world(p.0), 3.0, MAROON, PINK);    
            }
            draw_enemy(pos_to_world(e.pos), Vec2::angle_between(e.pos - g.player.pos, Vec2::Y), ORANGE, GOLD);
        }
        for b in g.blanks.iter() {
            draw_cir(pos_to_world(b.0), b.2, b.3, GRAY);
        }
        for i in 0..g.player.health {
            draw_cir(Vec2::splat(20.0) + Vec2::new(i as f32 * 20.0, 0.0), 10.0, GREEN, WHITE);
        }
        for i in 0..g.player.blank {
            draw_cir(Vec2::splat(20.0) + Vec2::new(i as f32 * 20.0, 20.0), 10.0, BLUE, WHITE);
        }
        draw_text(format!("{:.2}", g.elapsed_time).as_str(), 5.0, screen_height()-10.0, 20.0, WHITE);
    }
}