extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::render::Canvas;
use sdl2::video::Window;

use ya_raycaster::Player;
use ya_raycaster::draw_2d_world;
use ya_raycaster::move_player;
use ya_raycaster::get_rays;
use ya_raycaster::get_deltas;
use ya_raycaster::draw_rays;

const BLACK: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 0);
const BLUE: sdl2::pixels::Color =  sdl2::pixels::Color::RGB(0, 0, 30);


const WINDOW_HEIGHT: u32 = 512;
const WINDOW_WIDTH: u32 = 1024;

pub fn main() {
    let mut main_player = Player{
        pos_x: 256.0,
        pos_y: 256.0,
        angle: 60,
        dir_x: get_deltas(60).0,
        dir_y: get_deltas(60).1,
    };

    let game_map: [[i32; 8]; 8] = [
        [1, 1, 1, 1, 1, 1, 1, 1, ],
        [1, 0, 0, 0, 0, 0, 1, 1, ],
        [1, 0, 1, 0, 0, 0, 0, 1, ],
        [1, 0, 1, 0, 0, 1, 0, 1, ],
        [1, 0, 1, 0, 0, 1, 0, 1, ],
        [1, 0, 0, 0, 0, 1, 0, 1, ],
        [1, 1, 0, 1, 0, 1, 0, 1, ],
        [1, 1, 1, 1, 1, 1, 1, 1, ],


    ];


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window: Window = video_subsystem.window("YA Raycaster", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas: Canvas<Window> = window.into_canvas().target_texture().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // Resets screen to black, if not hall of mirrors effect will be displayed
        canvas.set_draw_color(BLUE);
        canvas.clear();
        // ** //


        move_player(&event_pump, &mut main_player, &game_map);
        let (ray_distances, ray_hit_sides) = get_rays(&main_player, &game_map, &mut canvas);
        draw_2d_world(&mut canvas, &main_player, &game_map);
        draw_rays(&mut canvas, ray_distances, ray_hit_sides);
        //println!("{:?}", ray_distances);

        // Put changes to the screen
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}