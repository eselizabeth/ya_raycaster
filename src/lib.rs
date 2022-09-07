
use std::fmt;
use std::collections::HashSet;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::keyboard::Scancode;
use sdl2::gfx::primitives::DrawRenderer;

const WINDOW_HEIGHT: f32 = 512_f32;
const WINDOW_WIDTH: u32 = 1024;

const BLACK: Color = Color::RGB(0, 0, 0);
const WHITE: Color =  Color::RGB(255, 255, 255);
const GRAY: Color = Color::RGB(112, 128, 144);
const RED: Color =  Color::RGB(255, 0, 0);
const GREEN: Color =  Color::RGB(0, 255, 0);
const DARK_GREEN: Color =  Color::RGB(0, 100, 0);
const BLUE: Color =  Color::RGB(0, 0, 255);


const BLOCKSIZE: u32 = 64;
const PLAYER_SPEED: f32 = 4.0;
const ROTATION_SPEED: i32 = 4;
const RAY_STEP: f32 = 0.05;
const RAY_COUNT: usize = 60;
const PI_VALUE: f32 = std::f64::consts::PI as f32;


pub struct Player{
    pub pos_x: f32, // X position
    pub pos_y: f32, // Y position
    pub angle: i32, // Player angle
    pub dir_x: f32, // Delta X
    pub dir_y: f32, // Delta Y

}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(POS[X={} | Y={}], Angle = {}, Dir[X={} | Y={}])", self.pos_x, self.pos_y, self.angle, self.dir_x, self.dir_y)
    }
}

/// Moves the player according to pressed keys(W/A/S/D)
pub fn move_player(e: &sdl2::EventPump, player: &mut Player, game_map: &[[i32; 8]; 8]){
    let pressed_keys:HashSet<Scancode> = e.keyboard_state().pressed_scancodes().collect();
    if pressed_keys.contains(&Scancode::W){
        player.pos_y += player.dir_y * PLAYER_SPEED;
        player.pos_x += player.dir_x * PLAYER_SPEED;
    }
    else if pressed_keys.contains(&Scancode::S){
        player.pos_y -= player.dir_y * PLAYER_SPEED;
        player.pos_x -= player.dir_x * PLAYER_SPEED;
    }
    else if pressed_keys.contains(&Scancode::A){
        player.angle += ROTATION_SPEED;
        player.angle = normalize_angle(player.angle);
        (player.dir_x, player.dir_y) = get_deltas(player.angle);

    }
    else if pressed_keys.contains(&Scancode::D){
        player.angle -= ROTATION_SPEED;
        player.angle = normalize_angle(player.angle);
        (player.dir_x, player.dir_y) = get_deltas(player.angle);
    }
    //println!("{}", player);
}



pub fn draw_2d_world(canvas: &mut Canvas<Window>, player: &Player, game_map: &[[i32; 8]; 8]){
    let mut x_position = 0;
    let mut y_position = 0;
    canvas.set_draw_color(WHITE);

    for (_, row) in game_map.iter().enumerate() {
        for (_, value) in row.iter().enumerate() {
            if *value == 1{
                canvas.set_draw_color(WHITE);
                canvas.fill_rect(Rect::new(x_position, y_position, BLOCKSIZE, BLOCKSIZE)).expect("Couldn't draw the block");
            }

            // This draws the grid
            canvas.set_draw_color(GRAY);
            canvas.fill_rect(Rect::new(x_position+BLOCKSIZE as i32 - 1, y_position, 1, BLOCKSIZE)).expect("Couldn't draw horizontal grid");
            canvas.fill_rect(Rect::new(x_position, y_position+BLOCKSIZE as i32 - 1, BLOCKSIZE, 1)).expect("Couldn't draw vertical grid");
            x_position += 64;
        }
        y_position += 64;
        x_position = 0;

    }

    canvas.set_draw_color(RED);
    canvas.fill_rect(Rect::new(player.pos_x as i32 - 4, player.pos_y as i32 - 4, 8, 8)).expect("Couldn't draw player");
    canvas.thick_line((player.pos_x + player.dir_x * 20_f32) as i16, (player.pos_y + player.dir_y * 20_f32) as i16,
                      (player.pos_x) as i16, (player.pos_y) as i16, 2, GREEN)
                      .expect("Couldn't draw the direction pointer");


}

pub fn draw_rays(canvas: &mut Canvas<Window>, ray_distances: [f32; RAY_COUNT]){
    let mut x_pos: i16 = WINDOW_WIDTH as i16;
    for (_, wall_distance) in ray_distances.iter().enumerate(){
        x_pos -= 8;
        let line_height =(BLOCKSIZE as f32 * WINDOW_HEIGHT) / wall_distance;
        println!("line height {}", line_height);
        let mut line_start = (-line_height / 2_f32) + (WINDOW_HEIGHT / 2_f32);
        if line_start < 0_f32 { 
            line_start = 0_f32;
        }
        let mut line_end = (line_height / 2_f32) + (WINDOW_HEIGHT / 2_f32);
        if line_end >= WINDOW_HEIGHT{
            line_end = WINDOW_HEIGHT - 1_f32;
        }
        canvas.thick_line((x_pos) as i16, (line_end) as i16,
        (x_pos) as i16, (line_start) as i16, 8, GREEN)
        .expect("Couldn't draw the ray");
    }
}



pub fn get_rays(player: &Player, game_map: &[[i32; 8]; 8], canvas: &mut Canvas<Window>) -> [f32; RAY_COUNT]{
    let mut ray_distances: [f32; RAY_COUNT] = [0_f32; RAY_COUNT]; 
    let orig_pos_x = player.pos_x;
    let orig_pos_y = player.pos_y;
    let player_angle = player.angle;
    // ** //
    let mut current_x = player.pos_x;
    let mut current_y = player.pos_y;
    let mut ray_angle = player.angle - (RAY_COUNT as i32 / 2);
    let mut array_idx: usize = 0;
    loop{
        let player_dir_x;
        let player_dir_y;
        (player_dir_x, player_dir_y) = get_deltas(ray_angle);
        'inner: loop{
            current_y += player_dir_y * RAY_STEP;
            current_x += player_dir_x * RAY_STEP;
            //println!("Current angle {}, dir_x {}, dir_y {}", player_angle, player_dir_x, player_dir_y);
            let idx_y = (current_x / BLOCKSIZE as f32) as usize; // THESE TWO ARE CORRECT
            let idx_x = (current_y / BLOCKSIZE as f32) as usize; // DUE TO HOW SDL2 HANDLES X/Y AXIS'

            if game_map[idx_x][idx_y] == 1{
                let distance = get_distance((orig_pos_x, orig_pos_y), (current_x, current_y));
                ray_distances[array_idx] = fix_fisheye(player_angle, ray_angle, distance);
                array_idx += 1;
                canvas.thick_line((current_x) as i16, (current_y) as i16,
                (orig_pos_x) as i16, (orig_pos_y) as i16, 2, GREEN)
                .expect("Couldn't draw the ray");
                break 'inner;
            }
        }
        ray_angle += 1;
        ray_angle = normalize_angle(ray_angle);
        current_x = orig_pos_x;
        current_y = orig_pos_y;
        if array_idx == RAY_COUNT{
            return ray_distances;
        }
        
    }
}

/// Fixes the fisheye effect caused by get_distance function
fn fix_fisheye(player_angle: i32, current_angle: i32, distance: f32) -> f32{
    let angle_difference = (player_angle - current_angle) as f32;
    return distance * (angle_difference.to_radians().cos());
}

/// Returns distance of two points
fn get_distance(pos_start: (f32, f32), pos_end: (f32, f32)) -> f32{
    return (((pos_end.0 - pos_start.0) * (pos_end.0 - pos_start.0)) + 
            ((pos_end.1 - pos_start.1) * (pos_end.1 - pos_start.1))).sqrt();
}

/// Normalizes angle of the player to 0 < angle < 360
fn normalize_angle(angle: i32) -> i32{
    if angle < 0{
        return angle + 360;
    } 
    else if angle > 360{
        return angle - 360;
    } 
    else{
        return angle;
    }
}

/// Returns x and y axis' values of the given angle
pub fn get_deltas(angle: i32) -> (f32, f32){
    let delta_x  = (angle as f32).to_radians().cos();
    let delta_y = (angle as f32).to_radians().sin() * -1_f32;
    return (delta_x, delta_y);
}


// Tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize_angle() {
        let over_360 = 365;
        let negative = -152;
        let normal = 129;
        assert_eq!(normalize_angle(over_360), 5);
        assert_eq!(normalize_angle(negative), 208);
        assert_eq!(normalize_angle(normal), 129);
    }
    #[test]
    fn test_get_deltas() {
        let example_1 = 47;
        let example_2 = 95;
        let example_3 = 192;
        let example_4 = 279;
        assert_eq!(get_deltas(example_1), (0.6819984, -0.7313537));
        assert_eq!(get_deltas(example_2), (-0.08715577, -0.9961947));
        assert_eq!(get_deltas(example_3), (-0.97814757, 0.20791179));
        assert_eq!(get_deltas(example_4), (0.15643454, 0.9876883));
    }
    #[test]
    fn test_get_distance() {
        let point1 = (7_f32, 5_f32);
        let point2 = (11_f32, 8_f32);
        assert_eq!(get_distance(point1, point2), 5_f32);

    }
}