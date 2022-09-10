use std::fmt;
use std::collections::HashSet;
use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::keyboard::Scancode;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::Texture;
pub mod map;

pub const WINDOW_HEIGHT: u32 = 512;
pub const WINDOW_WIDTH: u32 = 1024;

pub const MAP_LENGTH: usize = 8;
pub const MAP_WIDTH: usize = 8;


pub const BLACK: Color = Color::RGB(0, 0, 0);
pub const WHITE: Color =  Color::RGB(255, 255, 255);
pub const GRAY: Color = Color::RGB(112, 128, 144);
pub const RED: Color =  Color::RGB(255, 0, 0);
pub const GREEN: Color =  Color::RGB(0, 255, 0);
pub const DARK_GREEN: Color =  Color::RGB(0, 100, 0);
pub const BLUE: Color =  Color::RGB(0, 0, 255);


pub const BLOCKSIZE: u32 = 64;
const PLAYER_SPEED: f32 = 4.0;
const ROTATION_SPEED: f32 = 4.0;
const RAY_COUNT: usize = 60; // Ray Count must be even
const BULLET_SPEED: f32 = 4.0;




#[derive(Debug, Copy, Clone)]
pub struct Player{
    pub pos_x: f32, // X position
    pub pos_y: f32, // Y position
    pub angle: f32, // Player angle
    pub dir_x: f32, // Delta X
    pub dir_y: f32, // Delta Y

}

#[derive(Debug, Copy, Clone)]
pub struct Ray{
    pub distance: f32, // distance between the player and where the ray hit
    pub hit_side: i32, // where the ray hit, 0 if horizontal, 1 if vertical
    pub pos_x: i32, // x position of ray hit
}

impl Ray{
    fn new() -> Ray{
        Ray{
            distance: -1.0,
            hit_side: -1,
            pos_x: -1,
        }
    }

}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(POS[X={} | Y={}], Angle = {}, Dir[X={} | Y={}])", self.pos_x, self.pos_y, self.angle, self.dir_x, self.dir_y)
    }
}

/// Moves the player according to pressed keys(W/A/S/D)
pub fn move_player(e: &sdl2::EventPump, player: &mut Player, game_map: map::GameMap){
    let pressed_keys:HashSet<Scancode> = e.keyboard_state().pressed_scancodes().collect();
    if pressed_keys.contains(&Scancode::W){
        player.pos_y += player.dir_y * PLAYER_SPEED;
        player.pos_x += player.dir_x * PLAYER_SPEED;
        let idx_y = (player.pos_x / BLOCKSIZE as f32) as usize; // THESE TWO ARE CORRECT
        let idx_x = (player.pos_y / BLOCKSIZE as f32) as usize; // DUE TO HOW SDL2 HANDLES X/Y AXIS'
        if game_map.walls[idx_x][idx_y] == 1{
            player.pos_y -= player.dir_y * PLAYER_SPEED;
            player.pos_x -= player.dir_x * PLAYER_SPEED;
        }
    }
    else if pressed_keys.contains(&Scancode::S){
        player.pos_y -= player.dir_y * PLAYER_SPEED;
        player.pos_x -= player.dir_x * PLAYER_SPEED;
        let idx_y = (player.pos_x / BLOCKSIZE as f32) as usize; // THESE TWO ARE CORRECT
        let idx_x = (player.pos_y / BLOCKSIZE as f32) as usize; // DUE TO HOW SDL2 HANDLES X/Y AXIS'
        if game_map.walls[idx_x][idx_y] == 1{
            player.pos_y += player.dir_y * PLAYER_SPEED;
            player.pos_x += player.dir_x * PLAYER_SPEED;
        }
    }
    if pressed_keys.contains(&Scancode::A){
        player.angle += ROTATION_SPEED;
        player.angle = normalize_angle(player.angle);
        (player.dir_x, player.dir_y) = get_deltas(player.angle);

    }
    else if pressed_keys.contains(&Scancode::D){
        player.angle -= ROTATION_SPEED;
        player.angle = normalize_angle(player.angle);
        (player.dir_x, player.dir_y) = get_deltas(player.angle);
    }
}


// Draws the 2D world
pub fn draw_2d_world(canvas: &mut Canvas<Window>, player: &Player, game_map: map::GameMap){
    let mut x_position = 0;
    let mut y_position = 0;
    canvas.set_draw_color(WHITE);

    for (_, row) in game_map.walls.iter().enumerate() {
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
                      (player.pos_x) as i16, (player.pos_y) as i16, 2, RED)
                      .expect("Couldn't draw the direction pointer");
}

// Draws the 2.5D world
pub fn draw_rays(canvas: &mut Canvas<Window>, rays: [Ray; RAY_COUNT], texture_gun: &Texture<'_>, textures: &mut[Texture; 2]){
    let mut x_pos: i16 = WINDOW_WIDTH as i16;
    for (_, ray) in rays.iter().enumerate(){
        x_pos -= 8;
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
        if ray.hit_side == 0{
            texture = &textures[0];
        }
        else if ray.hit_side == 1{
            texture = &textures[1];
        }
        

        // In case the line height is bigger than screen normalize it
        if line_height > WINDOW_HEIGHT as f32{
           line_start = (WINDOW_HEIGHT as f32 - line_height) / 2.0;
        }
        let buffer = Rect::new(ray.pos_x, 0, 8, line_height as u32); // src
        let position = Rect::new(x_pos as i32, line_start as i32, 8, line_height as u32); // dst
        canvas.copy(&texture, buffer, position).expect("Couldn't draw the ray");
    }
    // Drawing the gun
    let gun = Rect::new(0, 0, 256, 128); // src
    let position = Rect::new(512 + 128, 512 - 128, 256, 128); // dst
    canvas.copy(&texture_gun, gun, position).expect("Couldn't draw the ray");
}


/// Returns the ray distance(s) and the side(s) they were hit
pub fn get_rays(player: &Player, game_map: map::GameMap, canvas: &mut Canvas<Window>) -> [Ray; RAY_COUNT]{
    let mut rays: [Ray; RAY_COUNT] = [Ray::new(); RAY_COUNT]; 
    let player_x = player.pos_x;
    let player_y = player.pos_y;
    let player_angle = player.angle;
    // ** //
    let mut current_x: f32 = player_x;
    let mut current_y: f32 = player_y;
    let mut ray_angle: f32 = player.angle - (RAY_COUNT as f32 / 2.0);
    let mut array_idx: usize = 0;

    loop {
        ray_angle = normalize_angle(ray_angle);
        let mut horizontal_hit_pos: (f32, f32) = (-1.0, -1.0);
        let mut vertical_hit_pos: (f32, f32) = (-1.0, -1.0);
        let mut horizontal_distance: f32 = 9999.0;
        let mut vertical_distance: f32 = 9999.0;
        let mut x_step: f32 = -1.0;
        let mut y_step: f32 = -1.0;
        let inverse_tan = 1.0/(ray_angle.to_radians().tan());
        let tan = (ray_angle).to_radians().tan();

        // Horizontal Check //
        if ray_angle > 0.0 && ray_angle < 180.0 { // facing up
            y_step = -(BLOCKSIZE as f32);
            current_y = ((player_y as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) - 0.001;
        }
        else if ray_angle > 180.0 && ray_angle < 360.0 { // facing down
            y_step = BLOCKSIZE as f32;
            current_y = ((player_y as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) + 64.0;
        }
        x_step = -y_step * inverse_tan;
        current_x = (player_y - current_y) * inverse_tan + player_x;
        'inner: loop{
            if ray_angle == 0.0 || ray_angle == 180.0 {break 'inner;}
            if out_of_index(current_x, current_y) {horizontal_hit_pos = (current_x, current_y); break 'inner};
            if game_map.walls[(current_y as usize/64)][current_x as usize/64 as usize] == 1{
                    horizontal_distance = get_distance(player_x, player_y, current_x, current_y, ray_angle);
                    horizontal_hit_pos = (current_x, current_y);
                    break 'inner;
            }
            
            current_x += x_step;
            current_y += y_step;
        }
            
        // Horizontal Check end //

        // Vertical Check //
        if ray_angle > 90.0 && ray_angle < 270.0  { // facing left
            x_step = -(BLOCKSIZE as f32);
            current_x = ((player_x as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) - 0.001;
        }
        else if ray_angle > 270.0 || ray_angle < 90.0{ // facing right
            x_step = BLOCKSIZE as f32; 
            current_x = ((player_x as i32 / BLOCKSIZE as i32) as f32 * BLOCKSIZE as f32) + 64.0;
        }
        y_step = -x_step * tan;
        current_y = (player_x - current_x) * tan + player_y;
        'inner: loop{       
            if ray_angle == 90.0 || ray_angle == 270.0 {break 'inner};     
            if out_of_index(current_x, current_y){vertical_hit_pos = (current_x, current_y); break 'inner };
            if game_map.walls[(current_y as usize/64)][current_x as usize/64 as usize] == 1{
                vertical_distance = get_distance(player_x, player_y, current_x, current_y, ray_angle);
                vertical_hit_pos = (current_x, current_y);
                
                break 'inner;
            }
            current_x += x_step;
            current_y += y_step;
        }
        // Vertical Check end //
        let mut current_ray = Ray::new();

        if horizontal_distance < vertical_distance{
            current_ray.distance = fix_fisheye(player_angle, ray_angle, horizontal_distance);
            current_ray.hit_side = 0;
            current_ray.pos_x = (horizontal_hit_pos.0.floor() as u32 % BLOCKSIZE) as i32;
            canvas.thick_line((horizontal_hit_pos.0) as i16, (horizontal_hit_pos.1) as i16,
            (player_x) as i16, (player_y) as i16, 2, GREEN)
            .expect("Couldn't draw the ray");
        }
        else if vertical_distance < horizontal_distance{
            current_ray.distance = fix_fisheye(player_angle, ray_angle, vertical_distance);
            current_ray.hit_side = 1;
            current_ray.pos_x = (horizontal_hit_pos.0.floor() as u32 % BLOCKSIZE) as i32;
            canvas.thick_line((vertical_hit_pos.0) as i16, (vertical_hit_pos.1) as i16,
            (player_x) as i16, (player_y) as i16, 2, RED)
            .expect("Couldn't draw the ray");
        }
        rays[array_idx] = current_ray;
        ray_angle += 1.0;
        array_idx += 1;
        if array_idx == RAY_COUNT{
            return rays;
        }
        }
}

pub fn fire(player: &Player, game_map: map::GameMap) -> Vec<Rect>{
    let mut bullets: Vec<Rect> = Vec::new();
    let mut bullet_x = player.pos_x;
    let mut bullet_y = player.pos_y;
    let mut drawing_x_pos = 512 + 235;
    let mut drawing_y_pos = 512-120;
    let mut height: i32 = 64;
    let mut width: i32 = 64;
    loop {
        bullet_x += player.dir_x * BULLET_SPEED;
        bullet_y += player.dir_y * BULLET_SPEED;
        let idx_y: usize = bullet_x as usize / BLOCKSIZE as usize; // THESE TWO ARE CORRECT
        let idx_x: usize = bullet_y as usize / BLOCKSIZE as usize; // DUE TO HOW SDL2 HANDLES X/Y AXIS'
        if {(bullet_x.abs() > WINDOW_WIDTH as f32 || bullet_y.abs() > WINDOW_HEIGHT as f32)
            || game_map.walls[idx_x][idx_y] == 1} 
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
        drawing_y_pos -= 10;
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
    if idx_y >= MAP_LENGTH || idx_x >= MAP_WIDTH {
        return true;
    }
    else{
        return false;
    }

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