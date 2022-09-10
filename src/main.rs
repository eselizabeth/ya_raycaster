extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::render::TextureCreator;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;

use ya_raycaster::*;
pub mod map;

pub fn main() {
    let mut main_player = Player{
        pos_x: 256.0,
        pos_y: 256.0,
        angle: 60.0,
        dir_x: get_deltas(60.0).0,
        dir_y: get_deltas(60.0).1,
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window: Window = video_subsystem.window("YA Raycaster", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas: Canvas<Window> = window.into_canvas().target_texture().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game_textures: [sdl2::render::Texture; 2] = [
        texture_creator.load_texture("texture_1.png").expect("Couldn't load texture"),
        texture_creator.load_texture("texture_1_dark.png").expect("Couldn't load texture"),
    ];
    let mut bullets: Vec<Rect> = Vec::new();
    let texture_gun = texture_creator.load_texture("texture_gun.png").expect("Couldn't load texture");
    let texture_bullet = texture_creator.load_texture("texture_bullet.png").expect("Couldn't load texture");
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let mut do_fire: bool = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                Event::MouseButtonDown { .. } => {
                    do_fire = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                _ => {}
            }
        }
        // Resets screen to black, if not hall of mirrors effect will be displayed
        canvas.set_draw_color(BLACK);
        canvas.clear();
        // ** //


        move_player(&event_pump, &mut main_player, ya_raycaster::map::GAME_MAP);
        let rays = get_rays(&main_player, ya_raycaster::map::GAME_MAP, &mut canvas);
        draw_2d_world(&mut canvas, &main_player, ya_raycaster::map::GAME_MAP);
        draw_rays(&mut canvas, rays, &texture_gun, &mut game_textures);
        if do_fire{ bullets = fire(&main_player, ya_raycaster::map::GAME_MAP);}
        if !bullets.is_empty(){
            let bullet = Rect::new(0, 0, 64, 64); // src
            let position = bullets.pop().unwrap(); // dst
            canvas.copy(&texture_bullet, bullet, position).expect("Couldn't draw the ray");
        }
        canvas.present(); // Put changes to the screen
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}