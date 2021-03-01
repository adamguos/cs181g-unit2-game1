use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod screen;
use screen::Screen;

mod collision;
use collision::{Collider, Contact, Mobile, Projectile, Terrain};

mod entity;
use entity::Entity;

mod texture;
use texture::Texture;

mod tiles;
use tiles::{Tile, Tilemap, Tileset, TILE_SZ};

mod animation;
use animation::{Animation, AnimationSM};

mod sprite;
use sprite::*;

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
    scroll: Vec2i,
}

// seconds per frame
const DT: f64 = 1.0 / 60.0;

const WIDTH: usize = 480;
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
    let mut terrain_tile_ids = HashMap::new();
    terrain_tile_ids.insert(
        String::from("ground"),
        vec![3169, 2905, 1, 356, 268, 312, 61, 144],
    );
    let tileset = Rc::new(Tileset::new(
        vec![Tile { solid: false }; 88 * 69],
        &Rc::new(Texture::with_file(Path::new("content/tilesheet.png"))),
        terrain_tile_ids,
    ));

    let tilemaps = vec![
        Tilemap::new(Vec2i(0, 0), (30, 50), &tileset, vec![3169; 1500]),
        Tilemap::new(Vec2i(0, 800), (30, 50), &tileset, vec![2095; 1500]),
    ];

    // Player sprite
    let player_sprite = Sprite::new(
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
        tilemaps: tilemaps,
        terrains: vec![],
        mobiles: vec![player],
        projs: vec![],
        frame_count: 0,
        scroll: Vec2i(0, 0),
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
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, state.scroll);
            screen.clear(Rgba(0, 0, 0, 0));

            // Load and unload tilemaps if necessary
            update_tilemaps(&mut state);

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

fn update_tilemaps(state: &mut GameState) {
    // Unload tilemaps that are off screen, and check if new tilemap needs to be loaded
    let mut visible = vec![];
    let mut no_need_load = false;
    for map in state.tilemaps.iter() {
        visible.push(map.is_visible(state.scroll, Vec2i(WIDTH as i32, HEIGHT as i32)));
        no_need_load = no_need_load || ((map.position.1 + TILE_SZ as i32) < state.scroll.1);
    }
    let mut i = 0;
    state.tilemaps.retain(|_| (visible[i], i += 1).0);

    // Load new tilemap if need
    if !no_need_load {
        let mut rng = rand::thread_rng();
        let tile_idx = rng.gen_range(0..state.tilemaps[0].tileset.tile_ids["ground"].len());
        let tile_id = state.tilemaps[0].tileset.tile_ids["ground"][tile_idx];
        println!("tile_id {}", tile_id);

        let new_map = Tilemap::new(
            Vec2i(
                state.scroll.0,
                state.scroll.1 - HEIGHT as i32 + TILE_SZ as i32,
            ),
            (WIDTH / TILE_SZ, HEIGHT / TILE_SZ),
            &state.tilemaps[0].tileset,
            vec![tile_id; WIDTH * HEIGHT / TILE_SZ / TILE_SZ],
        );
        state.tilemaps.push(new_map);
    }
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
    state.scroll.1 -= 1;

    // Player control goes here
    if input.key_pressed(VirtualKeyCode::Right) {
        state.mobiles[0].move_pos(2, 0);
        state.mobiles[0]
            .sprite
            .animation_sm
            .input("stop_moving", frame);
    }
    if input.key_pressed(VirtualKeyCode::Left) {
        state.mobiles[0].move_pos(-2, 0);
        state.mobiles[0]
            .sprite
            .animation_sm
            .input("stop_moving", frame);
    }
    if input.key_pressed(VirtualKeyCode::Up) {
        state.mobiles[0].move_pos(0, -2);
        state.mobiles[0].sprite.animation_sm.input("move_up", frame);
    }
    if input.key_pressed(VirtualKeyCode::Down) {
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
