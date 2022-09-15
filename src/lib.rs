use std::fmt;
use std::collections::HashSet;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::video::Window;
use sdl2::render::{Canvas, Texture};
use sdl2::keyboard::Scancode;
pub mod map;

pub const WINDOW_HEIGHT: u32 = 512;
pub const WINDOW_WIDTH: u32 = 720;

pub const MAP_LENGTH: usize = 16;
pub const MAP_WIDTH: usize = 16;


pub const BLACK: Color = Color::RGB(0, 0, 0);
pub const WHITE: Color =  Color::RGB(255, 255, 255);
pub const GRAY: Color = Color::RGB(112, 128, 144);
pub const RED: Color =  Color::RGB(255, 0, 0);
pub const GREEN: Color =  Color::RGB(0, 255, 0);
pub const DARK_GREEN: Color =  Color::RGB(0, 100, 0);
pub const BLUE: Color =  Color::RGB(0, 0, 255);


pub const BLOCKSIZE: u32 = 60;
const PLAYER_SPEED: f32 = 4.0;
const ROTATION_SPEED: f32 = 3.0;
pub const RAY_COUNT: usize = 60; // Ray Count must be even
const BULLET_SPEED: f32 = 1.0;
const RAY_DRAWING_WIDTH: u32 = 12; // Basically WINDOW_WIDTH / RayCount

const WORLDSIZE: u32 = BLOCKSIZE * MAP_LENGTH as u32;

const MINIMAP_SIZE: u32 = 128;
const MINIMAP_OFFSET_X: i32 = (WINDOW_WIDTH - MINIMAP_SIZE) as i32; 
const MINIMAP_OFFSET_Y: i32 = 0;
const MINIMAP_BLOCK_SIZE: u32 = 8; // 	inversely proportional with MAP_LENGTH & MAP_WIDTH


#[derive(Debug, Copy, Clone)]
pub struct Player{
    pub pos_x: f32, // X position
    pub pos_y: f32, // Y position
    pub angle: f32, // Player angle
    pub dir_x: f32, // Delta X
    pub dir_y: f32, // Delta Y
    pub fired: bool,

}

#[derive(Debug, Copy, Clone)]
pub struct Ray{
    pub distance: f32, // distance between the player and where the ray hit
    pub hit_side: i32, // where the ray hit, 0 if horizontal, 1 if vertical
    pub pos_x: i32, // x position of ray hit
    pub pos_y: i32,
}

impl Ray{
    pub fn new() -> Ray{
        Ray{
            distance: -1.0,
            hit_side: -1,
            pos_x: -1,
            pos_y: -1,
        }
    }

}
#[derive(Debug, Copy, Clone)]
pub struct Game{
    pub player: Player,
    pub rays: [[Ray; RAY_COUNT]; 3],
    pub game_map: map::GameMap,

}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(POS[X={} | Y={}], Angle = {}, Dir[X={} | Y={}], Fired: {}", self.pos_x, self.pos_y, self.angle, self.dir_x, self.dir_y, self.fired)
    }
}

/// Moves the player according to pressed keys(W/A/S/D)
pub fn move_player(e: &sdl2::EventPump, game: &mut Game){
    let pressed_keys:HashSet<Scancode> = e.keyboard_state().pressed_scancodes().collect();
    if pressed_keys.contains(&Scancode::W){
        game.player.pos_y += game.player.dir_y * PLAYER_SPEED;
        game.player.pos_x += game.player.dir_x * PLAYER_SPEED;
        if game.game_map.get_level(0, game.player.pos_x, game.player.pos_y) != 0{
            game.player.pos_y -= game.player.dir_y * PLAYER_SPEED;
            game.player.pos_x -= game.player.dir_x * PLAYER_SPEED;
        }
    }
    else if pressed_keys.contains(&Scancode::S){
        game.player.pos_y -= game.player.dir_y * PLAYER_SPEED;
        game.player.pos_x -= game.player.dir_x * PLAYER_SPEED;
        if game.game_map.get_level(0, game.player.pos_x, game.player.pos_y) != 0{
            game.player.pos_y += game.player.dir_y * PLAYER_SPEED;
            game.player.pos_x += game.player.dir_x * PLAYER_SPEED;
        }
    }
    if pressed_keys.contains(&Scancode::A){
        game.player.angle += ROTATION_SPEED;
        game.player.angle = normalize_angle(game.player.angle);
        (game.player.dir_x, game.player.dir_y) = get_deltas(game.player.angle);

    }
    else if pressed_keys.contains(&Scancode::D){
        game.player.angle -= ROTATION_SPEED;
        game.player.angle = normalize_angle(game.player.angle);
        (game.player.dir_x, game.player.dir_y) = get_deltas(game.player.angle);
    }
    //println!("player {}", game.player);
}


// Draws the 2D world
pub fn draw_2d_world(canvas: &mut Canvas<Window>, game: Game, gun_textures: &[Texture<'_>;3 ]){
    let mut x_position = 0;
    let mut y_position = 0;
    canvas.set_draw_color(WHITE);
    for (_, row) in game.game_map.first_level.iter().enumerate() {
        for (_, value) in row.iter().enumerate() {
            if *value != 0{
                canvas.set_draw_color(WHITE);
                canvas.fill_rect(Rect::new(MINIMAP_OFFSET_X + x_position, MINIMAP_OFFSET_Y + y_position, MINIMAP_BLOCK_SIZE, MINIMAP_BLOCK_SIZE)).expect("Couldn't draw the block");
            }
            x_position += MINIMAP_BLOCK_SIZE as i32;
        }
        y_position += MINIMAP_BLOCK_SIZE as i32;
        x_position = 0;

    }
    // Drawing the player to the minimap
    let (player_minimap_x, player_minimap_y) = normalize_for_minimap(game.player.pos_x, game.player.pos_y);
    canvas.set_draw_color(RED);
    canvas.fill_rect(Rect::new(player_minimap_x + 2,
                               player_minimap_y + 2,
                                4, 4)).expect("Couldn't draw player");
    // Drawing the gun
    let gun = Rect::new(0, 0, 128, 184); // src
    let position = Rect::new((WINDOW_WIDTH / 2) as i32 - 64, 512 - 184, 128, 184); // dst
    if game.player.fired{
        canvas.copy(&gun_textures[1], gun, position).expect("Couldn't draw the gun_fired");
    }
    else{
        canvas.copy(&gun_textures[0], gun, position).expect("Couldn't draw the gun_normal");
    }
}

// Draws the 2.5D world
pub fn draw_rays(canvas: &mut Canvas<Window>, game: Game, textures: &mut[Texture; 4]){
    // Reverse the layers so the first layer will be drawen last
    for (idx, level) in game.rays.iter().rev().enumerate(){
        let level_counter = 2 - idx;
        let mut x_pos: i16 = WINDOW_WIDTH as i16;
        let mut last_pos = (-1, -1);
        let mut buffer_cut: i32 = 64 - RAY_DRAWING_WIDTH as i32;
        for (_, ray) in level.iter().enumerate(){
            x_pos -= RAY_DRAWING_WIDTH as i16;
            if ray.pos_x == -1 || ray.pos_y == -1 {continue;}

            let line_height = (BLOCKSIZE * WINDOW_HEIGHT) as f32 / ray.distance;
            let mut line_start = (-line_height / 2_f32) + (WINDOW_HEIGHT / 2) as f32;
            if line_start < 0_f32 { 
                line_start = 0_f32;
            }
            // let mut line_end = (line_height / 2_f32) + (WINDOW_HEIGHT / 2) as f32;
            // if line_end >= WINDOW_HEIGHT as f32{
            //     line_end = (WINDOW_HEIGHT - 1) as f32;
            // }
            let mut texture = &textures[0];
            let texture_no = game.game_map.get_level(level_counter as i32, ray.pos_x as f32, ray.pos_y as f32) as usize;
            if ray.hit_side == 0{
                texture = &textures[(texture_no * 2) - 2usize];
            }
            else if ray.hit_side == 1{
                texture = &textures[(texture_no * 2) - 2usize];
            }
            if (ray.pos_x, ray.pos_y) == last_pos{
                buffer_cut -= RAY_DRAWING_WIDTH as i32;
            }
            else{
                buffer_cut = 64 - RAY_DRAWING_WIDTH as i32;
            }
            if buffer_cut < 0 {
                buffer_cut = 64 - RAY_DRAWING_WIDTH as i32;
            }
            // In case the line height is bigger than screen normalize it
            if line_height > WINDOW_HEIGHT as f32{
               line_start = (WINDOW_HEIGHT as f32 - line_height) / 2.0;
            }
            let buffer = Rect::new(buffer_cut, 0, RAY_DRAWING_WIDTH, line_height as u32);
            let position = Rect::new(x_pos as i32, (line_start - line_height * level_counter as f32) as i32, RAY_DRAWING_WIDTH, line_height as u32); // dst
            canvas.copy(&texture, buffer, position).expect("Couldn't draw the ray");

            last_pos = (ray.pos_x, ray.pos_y);
        }
    }
}


/// Casts rays and returns the ray distance(s) and the side(s) they were hit
pub fn get_rays(game: Game) -> [[Ray; RAY_COUNT]; 3]{
    let mut rays: [[Ray; RAY_COUNT]; 3] = [[Ray::new(); RAY_COUNT], [Ray::new(); RAY_COUNT], [Ray::new(); RAY_COUNT]]; 
    let player_x = game.player.pos_x;
    let player_y = game.player.pos_y;
    let player_angle = game.player.angle;
    // ** //
    let mut current_x: f32;
    let mut current_y: f32;
    let mut ray_angle: f32 = game.player.angle - (RAY_COUNT as f32 / 2.0);
    let mut array_idx: usize = 0;
    // For debug purposes
    if RAY_COUNT == 1{
        ray_angle = player_angle;
    }
    loop {
        ray_angle = normalize_angle(ray_angle);
        let mut x_step: f32;
        let mut y_step: f32;
        let inverse_tan = 1.0/(ray_angle.to_radians().tan());
        let tan = (ray_angle).to_radians().tan();

        // Horizontal Check //
        if ray_angle > 0.0 && ray_angle < 180.0 { // facing up
            y_step = -(BLOCKSIZE as f32);
            current_y = ((player_y as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) - 0.001;
        }
        else { // facing down, if ray_angle > 180.0 && ray_angle < 360.0 
            y_step = BLOCKSIZE as f32;
            current_y = ((player_y as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) + BLOCKSIZE as f32;
        }
        x_step = -y_step * inverse_tan;
        current_x = (player_y - current_y) * inverse_tan + player_x;
        let (horizontal_distances, horizontal_hit_poses) = calculate_distances(game.game_map, current_x, current_y, x_step, y_step, ray_angle, player_x, player_y, false);
            
        // Horizontal Check end //

        // Vertical Check //
        if ray_angle > 90.0 && ray_angle < 270.0  { // facing left
            x_step = -(BLOCKSIZE as f32);
            current_x = ((player_x as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) - 0.001;
        }
        else if ray_angle > 270.0 || ray_angle < 90.0{ // facing right
            x_step = BLOCKSIZE as f32; 
            current_x = ((player_x as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) + BLOCKSIZE as f32;
        }
        y_step = -x_step * tan;
        current_y = (player_x - current_x) * tan + player_y;

        let (vertical_distances, vertical_hit_poses) = calculate_distances(game.game_map, current_x, current_y, x_step, y_step, ray_angle, player_x, player_y, true);
        // Vertical Check end //
        for (idx, _) in horizontal_distances.iter().enumerate(){
            let mut current_ray = Ray::new();
            if horizontal_distances[idx] < vertical_distances[idx]{
                current_ray.distance = fix_fisheye(player_angle, ray_angle, horizontal_distances[idx]);
                current_ray.hit_side = 0;
                current_ray.pos_x = horizontal_hit_poses[idx].0;
                current_ray.pos_y = horizontal_hit_poses[idx].1;
            }
            else{
                current_ray.distance = fix_fisheye(player_angle, ray_angle, vertical_distances[idx]);
                current_ray.hit_side = 1;
                current_ray.pos_x = vertical_hit_poses[idx].0;
                current_ray.pos_y = vertical_hit_poses[idx].1;
            }
            rays[idx][array_idx] = current_ray;
        }
        
        ray_angle += 1.0;
        array_idx += 1;
        if array_idx == RAY_COUNT{
            return rays;
        }
        }
}

pub fn fire(game: Game) -> Vec<Rect>{
    let mut bullets: Vec<Rect> = Vec::new();
    let mut bullet_x = game.player.pos_x;
    let mut bullet_y = game.player.pos_y;
    let mut drawing_x_pos = 330;
    let mut drawing_y_pos = 330;
    let mut height: i32 = 64;
    let mut width: i32 = 64;
    loop {
        if {(bullet_x > WORLDSIZE as f32 || bullet_y > WORLDSIZE as f32
            || (bullet_x < 0.0 || bullet_y < 0.0))
            || game.game_map.get_level(0, bullet_x, bullet_y) != 0} 
        {
            bullets.reverse();
            return bullets;
        }
        else{
            let position = Rect::new(drawing_x_pos, drawing_y_pos, width as u32, height as u32); // dst
            bullets.push(position);
        }
        drawing_x_pos += 2;
        width -= 5;
        height -= 5;
        drawing_y_pos -= 5;
        bullet_x += game.player.dir_x * BULLET_SPEED;
        bullet_y += game.player.dir_y * BULLET_SPEED;
    }

}


/// Fixes the fisheye effect caused by get_distance function
fn fix_fisheye(player_angle: f32, current_angle: f32, distance: f32) -> f32{
    let angle_difference = player_angle - current_angle;
    return distance * (angle_difference.to_radians().cos());
}

/// Returns distance of two points
fn get_distance(player_x: f32, player_y: f32, current_x: f32, current_y: f32, ray_angle: f32) -> f32{
    return (ray_angle).to_radians().cos() * (current_x - player_x)-((ray_angle).to_radians().sin())*(current_y-player_y);
}

/// Normalizes angle of the player to 0 <= angle < 360
fn normalize_angle(angle: f32) -> f32{
    if angle < 0.0{
        return angle + 360.0;
    } 
    else if angle >= 360.0{
        return angle - 360.0;
    } 
    else{
        return angle;
    }
}

/// Returns x and y axis' values of the given angle
pub fn get_deltas(angle: f32) -> (f32, f32){
    let delta_x  = (angle).to_radians().cos();
    let delta_y = (angle).to_radians().sin() * -1.0;
    return (delta_x, delta_y);
}

/// Gets x and y position of a point with sizes of the array, returns 1 if they are out of index
fn out_of_index(x_position: f32, y_position: f32) -> bool{
    let idx_y: usize = x_position as usize / BLOCKSIZE as usize; // THESE TWO ARE CORRECT
    let idx_x: usize = y_position as usize / BLOCKSIZE as usize; // DUE TO HOW SDL2 HANDLES X/Y AXIS'
    if idx_y >= MAP_LENGTH || idx_x >= MAP_WIDTH 
       || x_position < 0.0 || y_position < 0.0{
        return true;
    }
    else{
        return false;
    }

}

// Normalizes X and Y position relative to scale of minimap
fn normalize_for_minimap(pos_x: f32, pos_y: f32) -> (i32, i32){
    return (
    MINIMAP_OFFSET_X + (pos_x / (MINIMAP_BLOCK_SIZE ) as f32) as i32,
    MINIMAP_OFFSET_Y +(pos_y / (MINIMAP_BLOCK_SIZE ) as f32) as i32);
}

// Calculates distances of each level, if it doesn't hit an array returns distance of 9999 and hit poses of (-1.0, -1.0)
fn calculate_distances(game_map: map::GameMap, orig_current_x: f32, orig_current_y: f32, x_step: f32, y_step: f32, ray_angle: f32, player_x: f32, player_y: f32, vertical: bool) -> ([f32; 3], [(i32, i32); 3]) {
    let mut distances: [f32; 3] = [9999.0; 3];
    let mut hit_poses: [(i32, i32); 3] = [(-1, -1); 3];
    for idx in 0..3{
        let mut current_x = orig_current_x;
        let mut current_y = orig_current_y;
        'inner: loop{
            if vertical{
                if ray_angle == 90.0 || ray_angle == 270.0 {break 'inner};
            }
            if !vertical{
                if ray_angle == 180.0 || normalize_angle(ray_angle) == 0.0 {break 'inner};
            }
            // println!("Vertical: {}, Level: {}, ray_angle: {}, current_x: {}, current_y: {}, x_step: {}, y_step {}", vertical, idx, ray_angle, current_x, current_y, x_step, y_step);
            if out_of_index(current_x, current_y) {hit_poses[idx] = (-1, -1); break 'inner};
            if game_map.get_level(idx as i32, current_x, current_y) != 0{
                distances[idx] = get_distance(player_x, player_y, current_x, current_y, ray_angle);
                hit_poses[idx] = (current_x as i32, current_y as i32);//((current_x as u32 / BLOCKSIZE) as i32, (current_y as u32 / BLOCKSIZE) as i32);
                break 'inner;
            }
            current_x += x_step;
            current_y += y_step;
        }
    }
    return (distances, hit_poses);
}


/// Tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize_angle() {
        let over_360 = 365.0;
        let negative = -152.0;
        let normal = 129.0;
        assert_eq!(normalize_angle(360.0), 0.0);
        assert_eq!(normalize_angle(over_360), 5.0);
        assert_eq!(normalize_angle(negative), 208.0);
        assert_eq!(normalize_angle(normal), 129.0);
    }
    #[test]
    fn test_get_deltas() {
        let example_1 = 47.0;
        let example_2 = 95.0;
        let example_3 = 192.0;
        let example_4 = 279.0;
        assert_eq!(get_deltas(example_1), (0.6819984, -0.7313537));
        assert_eq!(get_deltas(example_2), (-0.08715577, -0.9961947));
        assert_eq!(get_deltas(example_3), (-0.97814757, 0.20791179));
        assert_eq!(get_deltas(example_4), (0.15643454, 0.9876883));
    }
    #[test]
    fn test_out_of_index() {
        assert_eq!(out_of_index(0_f32, 0_f32), false);
        assert_eq!(out_of_index(64_f32, 64_f32), false);
        assert_eq!(out_of_index(513_f32, 513_f32), true);
        assert_eq!(out_of_index(1000_f32, 70_f32), true);
        assert_eq!(out_of_index(70_f32, 1000_f32), true);
    }

}