extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use ya_raycaster::*;
use soloud::*;
pub mod map;

pub fn main() {
    let mut game_instance = Game{
        player: Player{
            pos_x: 300.0,
            pos_y: 300.0,
            angle: 60.0,
            dir_x: get_deltas(60.0).0,
            dir_y: get_deltas(60.0).1,
            fired: false
        },
        wall_rays: [[Ray::new(); RAY_COUNT], [Ray::new(); RAY_COUNT], [Ray::new(); RAY_COUNT]],
        floor_rays: [Ray::new(); RAY_COUNT],
        game_map: ya_raycaster::map::GAME_MAP
    };

    // Loading sounds
    let soloud_player = Soloud::default().unwrap();
    let mut gun_shoot = audio::Wav::default();
    let mut gun_hit = audio::Wav::default();
    gun_shoot.load(&std::path::Path::new("assets/sounds/gun_shoot.wav")).unwrap();
    gun_hit.load(&std::path::Path::new("assets/sounds/gun_hit.wav")).unwrap();

    // Video
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window: Window = video_subsystem.window("YA Raycaster", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    
    let mut canvas: Canvas<Window> = window.into_canvas().target_texture().present_vsync().build().unwrap();
    // Textures
    let texture_creator = canvas.texture_creator();
    let mut bullets: Vec<Rect> = Vec::new();
    let mut wall_textures: [sdl2::render::Texture; 4] = [
        texture_creator.load_texture("assets/textures/block_1.png").expect("Couldn't load texture"),
        texture_creator.load_texture("assets/textures/block_1_dark.png").expect("Couldn't load texture"),
        texture_creator.load_texture("assets/textures/block_2.png").expect("Couldn't load texture"),
        texture_creator.load_texture("assets/textures/block_2_dark.png").expect("Couldn't load texture"),
    ];
    let mut gun_textures: [sdl2::render::Texture; 3] = [
        texture_creator.load_texture("assets/textures/gun_normal.png").expect("Couldn't load texture"),
        texture_creator.load_texture("assets/textures/gun_fired.png").expect("Couldn't load texture"),
        texture_creator.load_texture("assets/textures/bullet.png").expect("Couldn't load texture"),
    ];
    let mut floor_textures: [sdl2::render::Texture; 1] = [
        texture_creator.load_texture("assets/textures/block_3.png").expect("Couldn't load texture"),
    ];
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        game_instance.player.fired = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                Event::MouseButtonDown { .. } => {
                    game_instance.player.fired  = true;
                    soloud_player.play(&gun_shoot);
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
        move_player(&event_pump, &mut game_instance);
        get_rays(&mut game_instance, &mut canvas);
        draw_rays(&mut canvas, game_instance, &mut wall_textures, &mut floor_textures);
        draw_2d_world(&mut canvas, game_instance, &mut gun_textures);
        if game_instance.player.fired { bullets = fire(game_instance);}
        if !bullets.is_empty(){
            let bullet = Rect::new(0, 0, 64, 64); // src
            let position = bullets.pop().unwrap(); // dst
            canvas.copy(&gun_textures[2], bullet, position).expect("Couldn't draw the bullet");

        }
         // Put changes to the screen
        canvas.present();
        // ** //
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}