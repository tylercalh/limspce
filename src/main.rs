use macroquad::{prelude::*, color, rand::RandomRange};

#[macroquad::main("limspce")]
async fn main() {
    let scr_w = screen_width();
    let scr_h = screen_height();
    let hscr_w = scr_w/2.0;
    let hscr_h = scr_h/2.0;
    let world_to_pos = |pos: Vec2| Vec2::new(scr_w-pos.x-hscr_w,scr_h-pos.y-hscr_h);
    let pos_to_world = |pos: Vec2| Vec2::new(scr_w-(hscr_w+pos.x), scr_h-(hscr_h+pos.y));
    let offscreen = |pos: Vec2| { 
        -hscr_w > pos.x || pos.x > hscr_w ||
        -hscr_h > pos.y || pos.y > hscr_h
    };
    let mut frames = 1;
    let mut game_over = false;

    //colors:
    let bg_col = Color::from_hex(0x141414);
    let platform1_col = Color::from_hex(0x5a5a5a);
    let platform2_col = Color::from_hex(0x353535);
    let p_col_1 = Color::from_hex(0x5990de);
    let p_col_2 = Color::from_hex(0x93abcd);

    //Player info:
    let mut p_pos = Vec2::new(hscr_w, hscr_h);
    p_pos = world_to_pos(p_pos);
    let mut p_size = 7.0;
    let p_speed = 150.0;
    let p_z = 0.0;
    let mut p_c_col_1 = p_col_1;
    let mut p_c_col_2 = p_col_2;
    let mut p_health = 3;

    //Map info:
    let mut plat_pos = Vec2::new(hscr_w, hscr_h);
    plat_pos = world_to_pos(plat_pos);
    let mut plat_hsize = Vec2::new(200.0, 200.0);
    let mut lerp_plat_pos_p0 = plat_pos;
    let mut lerp_plat_pos_p1 = Vec2::new(0.0, 70.0);
    let mut lerp_plat_pos_t0 = 0.0;
    let mut lerp_plat_size_p0 = plat_hsize;
    let mut lerp_plat_size_p1 = plat_hsize;
    let mut lerp_plat_size_t0 = 0.0;

    //Enemy info:
    let mut enemy_pos = Vec2::new(45.0, 100.0);
    let mut enemy_scale = 150.0;
    let mut proj: Vec<(Vec2, Vec2, f32)> = Vec::new();

    loop {
        clear_background(bg_col);
        let p_on_plat = p_on_plat(p_pos, p_size, plat_pos, plat_hsize);

        //update platform
        let s = get_s(lerp_plat_pos_p0, lerp_plat_pos_p1, get_time(), lerp_plat_pos_t0, 20.0);
        let n_plat_pos = Vec2::lerp(lerp_plat_pos_p0, lerp_plat_pos_p1, s);
        let d_plat_pos = n_plat_pos - plat_pos;
        plat_pos = n_plat_pos;
        if s == 1.0 && frames % 500 == 0 {
            lerp_plat_pos_p1 = Vec2::new(RandomRange::gen_range(-hscr_w + plat_hsize.x, hscr_w - plat_hsize.x), RandomRange::gen_range(-hscr_h + plat_hsize.y, hscr_h - plat_hsize.y));
            lerp_plat_pos_p0 = plat_pos;
            lerp_plat_pos_t0 = get_time();
        }
        
        if frames % 1000 == 0 {
            lerp_plat_size_p0 = plat_hsize;
            lerp_plat_size_p1 = plat_hsize * 0.75;
            lerp_plat_size_t0 = get_time();
        }
        let s = get_s(lerp_plat_size_p0, lerp_plat_size_p1, get_time(), lerp_plat_size_t0, 20.0);
        plat_hsize = Vec2::lerp(lerp_plat_size_p0, lerp_plat_size_p1, s);


        //update player:
        if p_on_plat && !game_over {
            let mut dx = 0.0;
            let mut dy = 0.0;
            if is_key_down(KeyCode::A) {dx += 1.0 * p_speed * get_frame_time();}
            if is_key_down(KeyCode::D) {dx += -1.0 * p_speed * get_frame_time();} 
            if is_key_down(KeyCode::S) {dy += -1.0 * p_speed * get_frame_time();}
            if is_key_down(KeyCode::W) {dy += 1.0 * p_speed * get_frame_time();}
            p_pos += Vec2::new(dx, dy);

            p_pos += d_plat_pos;
        } else if game_over {
            p_c_col_1.a = 0.0;
            p_c_col_2.a = 0.0;
        } else {
            p_pos.y -= 120.0 * 1.0 * get_frame_time();
            p_c_col_1.a -= 1.0 * get_frame_time();
            p_c_col_2.a -= 1.0 * get_frame_time();
            p_size = f32::max(p_size - 4.0 * get_frame_time(), 0.0);
            if frames % 50 == 0 {p_health = (p_health - 1).max(0)};
        }
        if p_health == 0 {
            game_over = true;
        }

        //update enemy:
        let enemy_theta = get_time().sin()+ (get_time()/2.0).cos();
        //let enemy_theta = 50.0 * get_frame_time();
        enemy_pos = (rot_mat(enemy_theta as f32 * get_frame_time()) * (enemy_pos - plat_pos).normalize()) * enemy_scale + plat_pos;
        if frames % 100 == 0 || frames % 110 == 0 || frames % 120 == 0 {
            proj.push((enemy_pos, -(enemy_pos - p_pos).normalize(), 200.0));
        }
        //update proj:
        for (i, p) in proj.iter_mut().enumerate() {
            p.2 = p.2 + 4.0;
            p.0 = p.0 + (p.1 * p.2 * get_frame_time());
        }
        proj.retain(|&pos| !offscreen(pos.0)); //cull offscreen

        //Player proj collision:
        let mut rem_proj: Option<usize> = None;
        for (i, p) in proj.iter().enumerate() {
            if circle_overlap(p_pos, p_size, p.0, 3.0) {
                rem_proj = Some(i);
                p_health -= 1;
            }
        }
        if let Some(i) = rem_proj {
            proj.swap_remove(i);
        }

        // Draw:
        draw_platform(pos_to_world(plat_pos), plat_hsize.x, plat_hsize.y, platform1_col, platform2_col);
        draw_cir(pos_to_world(p_pos), p_size, p_c_col_1, p_c_col_2);

        for p in proj.iter() {
            let pos = pos_to_world(p.0);
            //draw_circle(pos.x, pos.y, 4.0, RED);
            draw_cir(pos, 3.0, RED, ORANGE);
        }
        draw_enemy(pos_to_world(enemy_pos), Vec2::angle_between(enemy_pos - p_pos, Vec2::Y),RED, WHITE);
        for i in 0..p_health {
            //draw_circle(20.0 + (i as f32 * 20.0), 20.0, 10.0, ORANGE);
            draw_cir(Vec2::splat(20.0) + Vec2::new(i as f32 * 20.0, 0.0), 10.0, GREEN, WHITE);
        }
        if game_over {
            draw_text("GAME OVER", 0.0, hscr_h, 200.0, WHITE);
        }

        frames += 1;
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
fn get_s(p0: Vec2, p1: Vec2, current_time: f64, t0: f64, speed: f32) -> f32 {
    let current_time = current_time as f32;
    let t0 = t0 as f32;
    let ratio = (current_time - t0) / Vec2::distance(p0, p1);
    let ratio = ratio * speed;
    f32::min(ratio, 1.0)
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