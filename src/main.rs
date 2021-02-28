use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// Whoa what's this?
// Mod without brackets looks for a nearby file.
mod screen;
// Then we can use as usual.  The screen module will have drawing utilities.
use screen::Screen;
// Collision will have our collision bodies and contact types
mod collision;
// Lazy glob imports
use collision::{Collider, Contact, Mobile, Projectile, Terrain};
// Entity struct
mod entity;
use entity::Entity;
// Texture has our image loading and processing stuff
mod texture;
use texture::Texture;
mod tiles;
use tiles::{Tile, Tilemap, Tileset};
// Animation will define our animation datatypes and blending or whatever
mod animation;
use animation::{Animation, AnimationSM};
// Sprite will define our movable sprites
mod sprite;
// Lazy glob import, see the extension trait business later for why
use sprite::*;
// And we'll put our general purpose types like color and geometry here:
mod types;
use types::*;

// Now this main module is just for the run-loop and rules processing.
struct GameState {
    // What data do we need for this game?  Wall positions?
    // Colliders?  Sprites and stuff?
    // animations: Vec<Animation>,
    // textures: Vec<Rc<Texture>>,
    // sprites: Vec<Sprite>,
    terrains: Vec<Terrain>,
    // entities: Vec<Entity>,
    tilemaps: Vec<Tilemap>,
    mobiles: Vec<Entity<Mobile>>,
    projs: Vec<Projectile>,
    frame_count: usize,
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;

const WIDTH: usize = 600;
const HEIGHT: usize = 800;
const DEPTH: usize = 4;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Unit2Game")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    let sprite_sheet = Rc::new(Texture::with_file(Path::new("content/link_sprites.png")));

    // How many frames have we simulated?
    let mut frame_count: usize = 0;

    // Tiles
    let tileset = Rc::new(Tileset::new(
        vec![Tile { solid: false }; 88 * 69],
        &Rc::new(Texture::with_file(Path::new("content/tilesheet.png"))),
    ));

    let map1 = Tilemap::new(
        Vec2i(0, 0),
        (45, 15),
        &tileset,
        [
            vec![3169; 240],
            vec![1408],
            vec![1409; 13],
            vec![1410],
            vec![3169; 30],
            vec![1496],
            vec![1233; 13],
            vec![1498],
            vec![3169; 30],
            vec![1496],
            vec![1233; 13],
            vec![1498],
            vec![3169; 30],
            vec![1496],
            vec![1233; 13],
            vec![1498],
            vec![3169; 30],
            vec![1496],
            vec![1233; 13],
            vec![1498],
            vec![3169; 30],
            vec![1584],
            vec![1321; 13],
            vec![1586],
            vec![3169; 195],
        ]
        .concat(),
    );

    // Player sprite
    let mut player_sprite = Sprite::new(
        &sprite_sheet,
        animation::AnimationSM::new(
            vec![
                animation::Animation::new(
                    vec![
                        Rect {
                            x: 0,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 30,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 60,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 90,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 120,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 150,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 180,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                        Rect {
                            x: 210,
                            y: 120,
                            w: 30,
                            h: 30,
                        },
                    ],
                    vec![10; 8],
                    frame_count,
                    true,
                ),
                animation::Animation::new(
                    vec![Rect {
                        x: 60,
                        y: 0,
                        w: 30,
                        h: 30,
                    }],
                    vec![60],
                    frame_count,
                    true,
                ),
            ],
            vec![
                (1, 0, String::from("move_up")),
                (0, 1, String::from("stop_moving")),
            ],
            frame_count,
            1,
        ),
        Vec2i(350, 500),
    );

    // Player entity
    let mut player = Entity {
        collider: Mobile::player(350, 500),
        position: Vec2i(350, 500),
        sprite: player_sprite,
    };

    // Initial game state
    let mut state = GameState {
        // entities: vec![player],
        tilemaps: vec![map1],
        terrains: vec![],
        mobiles: vec![player],
        projs: vec![],
        frame_count: 0,
    };
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track beginning of play
    let start = Instant::now();
    // Track end of the last frame
    let mut since = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, Vec2i(0, 0));
            screen.clear(Rgba(0, 0, 0, 0));

            draw_game(&mut state, &mut screen, frame_count);

            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            available_time += since.elapsed().as_secs_f64();
        }
        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize the window if needed
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;

            update_game(&mut state, &input, frame_count);

            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();
    });
}

fn draw_game(state: &mut GameState, screen: &mut Screen, frame_count: usize) {
    // Call screen's drawing methods to render the game state
    screen.clear(Rgba(80, 80, 80, 255));

    for map in state.tilemaps.iter() {
        map.draw(screen);
    }

    for proj in state.projs.iter() {
        screen.rect(proj.rect, Rgba(0, 128, 0, 255));
    }

    for e in state.mobiles.iter_mut() {
        screen.draw_sprite(&mut e.sprite, frame_count);
    }
}

fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    state.frame_count += 1;
    // Player control goes here
    if input.key_held(VirtualKeyCode::Right) {
        state.mobiles[0].move_pos(2, 0);
        state.mobiles[0]
            .sprite
            .animation_sm
            .input("stop_moving", frame);
    }
    if input.key_held(VirtualKeyCode::Left) {
        state.mobiles[0].move_pos(-2, 0);
        state.mobiles[0]
            .sprite
            .animation_sm
            .input("stop_moving", frame);
    }
    if input.key_held(VirtualKeyCode::Up) {
        state.mobiles[0].move_pos(0, -2);
        state.mobiles[0].sprite.animation_sm.input("move_up", frame);
    }
    if input.key_held(VirtualKeyCode::Down) {
        state.mobiles[0].move_pos(0, 2);
        state.mobiles[0]
            .sprite
            .animation_sm
            .input("stop_moving", frame);
    }
    // Update player position

    // Detect collisions: Generate contacts
    let mut contacts: Vec<Contact> = vec![];
    // collision::gather_contacts(&state.terrains, &state.mobiles, &state.projs, &mut contacts);
    collision::gather_contacts(
        &state.terrains,
        // &state
        //     .mobiles
        //     .into_iter()
        //     .map(|x| x.collider)
        //     .collect::<Vec<_>>(),
        &state.mobiles,
        &state.projs,
        &mut contacts,
    );

    // Handle collisions: Apply restitution impulses.
    collision::handle_contact(
        &mut state.terrains,
        &mut state.mobiles,
        &mut state.projs,
        &mut contacts,
    );

    if state.frame_count == 15 {
        // state.frame_count = 0;
        // state.projs.push(Projectile::new(&state.mobiles[0]));
        /*
        if let ColliderType::Mobile(ref mobile) = state.entities[0].collider {
            state.projs.push(Projectile::new(&mobile));
        }
        */
        state
            .projs
            .push(Projectile::new(&state.mobiles[0].collider));
        println!("pushed a proj.");
    }

    // Update game rules: What happens when the player touches things?
}
