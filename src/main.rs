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
use collision::{Collider, Contact, Mobile, Projectile, Terrain, Wall};

mod entity;
use entity::Entity;

mod texture;
use texture::Texture;

mod tiles;
use tiles::{Tile, Tilemap, Tileset, TILE_SZ};

mod animation;

mod sprite;
use sprite::*;

mod types;
use types::*;

mod assets;
use assets::*;

// Now this main module is just for the run-loop and rules processing.
struct GameState {
    terrains: Vec<Entity<Terrain>>,
    tilemaps: Vec<Tilemap>,
    mobiles: Vec<Entity<Mobile>>,
    walls: Vec<Wall>,
    projs: Vec<Projectile>,
    stage: GameStage,
    frame_count: usize,
    scroll: Vec2i,
    score: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameStage {
    Rocks(bool, usize),
    Boulders(usize),
    GameOver(usize),
}

// seconds per frame
const DT: f64 = 1.0 / 60.0;

const WIDTH: usize = 320;
const HEIGHT: usize = 576;
const DEPTH: usize = 4;
const TILEMAP_HT: usize = 256;

const WALL_SZ: usize = 32;
const ROCK_SZ: usize = 16;

// player shoots every PROJ_DT frames
const PROJ_DT: usize = 6;

fn init(tileset: &Rc<Tileset>, sprite_sheet: &Rc<Texture>) -> GameState {
    let mut tilemaps: Vec<Tilemap> = vec![];
    for i in 0..(HEIGHT / TILEMAP_HT + 1) {
        tilemaps.push(Tilemap::new(
            Vec2i(0, HEIGHT as i32 - (i * TILEMAP_HT) as i32),
            (WIDTH / TILE_SZ, TILEMAP_HT / TILE_SZ),
            tileset,
            vec![3169; (WIDTH / TILE_SZ) * (TILEMAP_HT / TILE_SZ)],
        ));
    }

    // Player sprite
    let player_sprite = assets::player_anim(sprite_sheet, 0);

    // Player entity
    let player = Entity {
        collider: Mobile::player(180, 500),
        position: Vec2i(180, 500),
        sprite: player_sprite,
    };

    // Initial game state
    GameState {
        tilemaps,
        terrains: vec![],
        mobiles: vec![player],
        walls: walls_vec(WIDTH as u16, HEIGHT as u16),
        projs: vec![],
        stage: GameStage::Rocks(true, 1),
        frame_count: 0,
        scroll: Vec2i(0, 0),
        score: 0,
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Space Shooter")
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

    let sprite_sheet = Rc::new(Texture::with_file(Path::new(
        "content/spaceshooter/Spritesheet/sheet.png",
    )));
    let font_sheet = Rc::new(Texture::with_file(Path::new("content/monospace_font.png")));
    let mut terrain_tile_ids = HashMap::new();
    terrain_tile_ids.insert(
        String::from("ground"),
        vec![3169, 2905, 1, 356, 268, 312, 61, 144],
    );
    let tile_sheet = Rc::new(Texture::with_file(Path::new("content/tilesheet.png")));
    let tileset = Rc::new(Tileset::new(
        vec![Tile { solid: false }; 88 * 69],
        &tile_sheet,
        terrain_tile_ids,
    ));

    let mut state = init(&tileset, &sprite_sheet);

    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    // Track end of the last frame
    let mut since = Instant::now(); //TODO: This seems to be similar?
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, state.scroll);

            // Load and unload tilemaps if necessary
            update_tilemaps(&mut state);

            // Draw current game
            draw_game(&mut state, &mut screen, &font_sheet);

            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            available_time += since.elapsed().as_secs_f64();
        }

        // Game over event
        if let GameStage::GameOver(death_frame) = state.stage {
            if state.frame_count - death_frame >= 150 {
                state = init(&tileset, &sprite_sheet);
            }
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
            update_game(&mut state, &input, &sprite_sheet, &tile_sheet);

            // Increment the frame counter
            state.frame_count += 1;
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

        let new_map = Tilemap::new(
            Vec2i(
                state.scroll.0,
                state.scroll.1 - TILEMAP_HT as i32 + TILE_SZ as i32,
            ),
            (WIDTH / TILE_SZ, TILEMAP_HT / TILE_SZ),
            &state.tilemaps[0].tileset,
            vec![tile_id; WIDTH * TILEMAP_HT / TILE_SZ / TILE_SZ],
        );
        state.tilemaps.push(new_map);
    }
}

fn draw_game(state: &mut GameState, screen: &mut Screen, font_sheet: &Rc<Texture>) {
    // Call screen's drawing methods to render the game state
    screen.clear(Rgba(255, 197, 255, 255));

    // Remove Terrain objects that have left screen
    cleanup_terrain(state, screen);

    for map in state.tilemaps.iter() {
        map.draw(screen);
    }

    for proj in state.projs.iter() {
        screen.rect(proj.rect, Rgba(0, 128, 0, 255));
    }

    for e in state.mobiles.iter_mut() {
        screen.draw_sprite(&mut e.sprite, state.frame_count);
    }

    for e in state.terrains.iter_mut() {
        screen.draw_sprite(&mut e.sprite, state.frame_count);
    }

    // Draw HP bar
    draw_string("HP", screen, font_sheet, Vec2i(20, 520), state.scroll);
    let hp = state.mobiles[0].collider.hp;
    screen.rect(
        Rect {
            x: 70,
            y: state.scroll.1 + 520,
            w: hp as u16 * 2,
            h: 18,
        },
        Rgba(0, 128, 0, 255),
    );
    screen.rect(
        Rect {
            x: 70 + (hp as i32 * 2),
            y: state.scroll.1 + 520,
            w: (100 - hp as u16) * 2,
            h: 18,
        },
        Rgba(128, 0, 0, 255),
    );
    screen.line(
        Vec2i(70, state.scroll.1 + 520),
        Vec2i(270, state.scroll.1 + 520),
        Rgba(0, 0, 0, 255),
    );
    screen.line(
        Vec2i(270, state.scroll.1 + 520),
        Vec2i(270, state.scroll.1 + 538),
        Rgba(0, 0, 0, 255),
    );
    screen.line(
        Vec2i(70, state.scroll.1 + 520),
        Vec2i(70, state.scroll.1 + 538),
        Rgba(0, 0, 0, 255),
    );
    screen.line(
        Vec2i(70, state.scroll.1 + 538),
        Vec2i(270, state.scroll.1 + 538),
        Rgba(0, 0, 0, 255),
    );
    screen.line(
        Vec2i(70 + (hp as i32 * 2), state.scroll.1 + 520),
        Vec2i(70 + (hp as i32 * 2), state.scroll.1 + 538),
        Rgba(0, 0, 0, 255),
    );

    // Draw score
    let mut score_msg = "Score ".to_string();
    score_msg.push_str(&state.score.to_string());
    draw_string(&score_msg, screen, font_sheet, Vec2i(20, 20), state.scroll);

    // Draw game over message if game is over
    if let GameStage::GameOver(_) = state.stage {
        draw_string(
            "Game over",
            screen,
            font_sheet,
            Vec2i(80, 200),
            state.scroll,
        );
        draw_string(
            "Restarting",
            screen,
            font_sheet,
            Vec2i(80, 250),
            state.scroll,
        );
    }
}

fn update_game(
    state: &mut GameState,
    input: &WinitInputHelper,
    sprite_sheet: &Rc<Texture>,
    tile_sheet: &Rc<Texture>,
) {
    state.scroll.1 -= 1;

    match state.stage {
        GameStage::Rocks(spawning_enemies, num_waves) => {
            // spawn rocks every 360 frames
            if state.frame_count % 360 == 120 {
                generate_terrain(state, tile_sheet, 0);
            }

            // bool in Rocks keeps track of whether we are still spawning enemies
            // to start the stage
            if spawning_enemies {
                if state.frame_count % 30 == 0 {
                    state.mobiles.push(enemy_entity(
                        sprite_sheet,
                        state.frame_count,
                        Vec2i(100, state.scroll.1 - 30),
                    ));
                }

                // once 4 are spawned (5 including player), stop spawning
                if state.mobiles.len() == 5 {
                    state.stage = GameStage::Rocks(false, num_waves);
                }
            }
            // once all enemies are dead, start spawning again
            else if state.mobiles.len() == 1 {
                let mut rng = rand::thread_rng();
                // starts being possible to move on to next stage after wave 2
                // guaranteed to move on after wave 5
                if rng.gen_range(0..4) + num_waves >= 5 {
                    state.stage = GameStage::Boulders(1);
                } else {
                    state.stage = GameStage::Rocks(true, num_waves + 1);
                }
            }
        }

        GameStage::Boulders(num_waves) => {
            // Spawn a boulder wall every n frames, number goes down as waves go up
            if state.frame_count % (300 - num_waves * 8) == 0 {
                generate_terrain(state, tile_sheet, 1);
                // starts being possible to move on to next stage after wave 4
                // guaranteed to move on after wave 7
                let mut rng = rand::thread_rng();
                if rng.gen_range(0..4) + num_waves >= 7 {
                    state.stage = GameStage::Rocks(true, 1);
                } else {
                    state.stage = GameStage::Boulders(num_waves + 1);
                }
            }
        }

        GameStage::GameOver(_) => {}
    }

    // Update player position
    // Player control goes here
    match state.stage {
        GameStage::Rocks(_, _) | GameStage::Boulders(_) => {
            if input.key_held(VirtualKeyCode::Right) {
                state.mobiles[0].collider.vx = 3.0;
            } else if input.key_held(VirtualKeyCode::Left) {
                state.mobiles[0].collider.vx = -3.0;
            } else {
                state.mobiles[0].collider.vx = 0.0;
            }
            if input.key_held(VirtualKeyCode::Up) {
                state.mobiles[0].collider.vy = -4.0;
            } else if input.key_held(VirtualKeyCode::Down) {
                state.mobiles[0].collider.vy = 2.0;
            } else {
                state.mobiles[0].collider.vy = -1.0;
            }

            if input.key_held(VirtualKeyCode::O) {
                state.stage = GameStage::Rocks(true, 1);
            } else if input.key_held(VirtualKeyCode::P) {
                state.stage = GameStage::Boulders(1);
            }
        }

        GameStage::GameOver(_) => {}
    }

    // Update enemy AI movements
    update_enemies(state);

    // Update position of mobiles
    for m in state.mobiles.iter_mut() {
        m.move_pos(m.collider.vx as i32, m.collider.vy as i32);
    }

    // Update proj position
    for proj in state.projs.iter_mut() {
        proj.move_pos(proj.get_velocity().0 as i32, proj.get_velocity().1 as i32);
    }

    // Update wall position (scroll with camera)
    for wall in state.walls.iter_mut() {
        wall.move_pos(0, -1);
    }

    // Detect collisions: Generate contacts
    let mut contacts: Vec<Contact> = vec![];
    collision::gather_contacts(
        &state.terrains,
        &state.mobiles,
        &state.walls,
        &state.projs,
        &mut contacts,
    );

    // Handle collisions
    let (player_is_alive, scores_gained) = collision::handle_contact(
        &mut state.terrains,
        &mut state.mobiles,
        &mut state.projs,
        &mut contacts,
    );

    if let GameStage::Rocks(_, _) | GameStage::Boulders(_) = state.stage {
        // Set GameOver stage if player is not alive
        if !player_is_alive {
            state.mobiles[0]
                .sprite
                .animation_sm
                .input("die", state.frame_count);
            state.mobiles[0].collider.vx = 0.0;
            state.mobiles[0].collider.vy = -1.0;
            state.stage = GameStage::GameOver(state.frame_count);
        } else {
            state.score += scores_gained;
        }

        // Fire projectile
        if state.frame_count % PROJ_DT == 0 {
            state
                .projs
                .push(Projectile::new(&state.mobiles[0].collider));
        }
    }
}

/**
 * Randomly picks hexadecimal string of length 4 and uses it to generate terrain objects.
 *
 * terrain_type: 0 = random rocks, 1 = wall with some rocks
 */
fn generate_terrain(state: &mut GameState, tile_sheet: &Rc<Texture>, terrain_type: usize) {
    let mut rng = rand::thread_rng();

    if terrain_type == 0 {
        for i in 0..(WIDTH / ROCK_SZ) {
            for j in 0..6 {
                if rng.gen_range(0..6) == 0 {
                    let pos = Vec2i(
                        (i * ROCK_SZ) as i32,
                        state.scroll.1 - (ROCK_SZ * (j + 1)) as i32,
                    );
                    state
                        .terrains
                        .push(rock_entity(tile_sheet, state.frame_count, pos));
                }
            }
        }
    } else if terrain_type == 1 {
        let seed = rng.gen_range(0..256);
        for i in 0..(WIDTH / WALL_SZ) {
            // ~1/3 chance of adding rocks instead of walls for 3 slots
            if ((seed + i) / 3) % 3 == 0 {
                // let pos1 = Vec2i((i * WALL_SZ) as i32, state.scroll.1 - WALL_SZ as i32);
                // let pos2 = Vec2i(
                //     (i * WALL_SZ + ROCK_SZ) as i32,
                //     state.scroll.1 - WALL_SZ as i32,
                // );
                let pos3 = Vec2i(
                    (i * WALL_SZ) as i32,
                    state.scroll.1 - WALL_SZ as i32 + ROCK_SZ as i32,
                );
                let pos4 = Vec2i(
                    (i * WALL_SZ + ROCK_SZ) as i32,
                    state.scroll.1 - WALL_SZ as i32 + ROCK_SZ as i32,
                );

                // state
                //     .terrains
                //     .push(rock_entity(tile_sheet, state.frame_count, pos1));
                // state
                //     .terrains
                //     .push(rock_entity(tile_sheet, state.frame_count, pos2));
                state
                    .terrains
                    .push(rock_entity(tile_sheet, state.frame_count, pos3));
                state
                    .terrains
                    .push(rock_entity(tile_sheet, state.frame_count, pos4));
            } else {
                let pos = Vec2i((i * WALL_SZ) as i32, state.scroll.1 - WALL_SZ as i32);
                state
                    .terrains
                    .push(boulder_entity(tile_sheet, state.frame_count, pos));
            }
        }
    }
}

fn cleanup_terrain(state: &mut GameState, screen: &Screen) {
    let frame_count = state.frame_count;
    state.terrains.retain(|t| {
        screen.is_visible(t.collider.rect) || frame_count - t.collider.created_at < 300
    });
}

fn update_enemies(state: &mut GameState) {
    let player_pos = state.mobiles[0].position.clone();

    for enemy in state.mobiles.iter_mut().skip(1) {
        // Accelerate away from nearby terrain
        for terrain in state.terrains.iter() {
            let dx = (terrain.position.0 - enemy.position.0) as f32;
            let dy = (terrain.position.1 - enemy.position.1) as f32;

            if dx.abs() < 50.0 && dy.abs() < 50.0 {
                if dx.abs() > dy.abs() {
                    enemy.collider.vx -= 5.0 / dx;
                } else {
                    enemy.collider.vy -= 5.0 / dy;
                }
            }
        }

        // Accelerate x towards player
        let mut dx = ((player_pos.0 - enemy.position.0) as f32) / 50.0;
        let max_vx = 0.07;
        if dx < -max_vx {
            dx = -max_vx;
        } else if dx > max_vx {
            dx = max_vx;
        }
        enemy.collider.vx += dx;

        // Accelerate y upward if enemy is below player
        let dy = player_pos.1 - enemy.position.1;
        let max_vy = 5.0;
        if dy < 0 {
            // enemy.collider.vy -= 0.03;
            enemy.collider.vy = (enemy.collider.vy - 0.03).max(-max_vy);
        }

        // Accelerate y downward if enemy is above player
        if dy > 0 {
            // enemy.collider.vy += 0.03;
            enemy.collider.vy = (enemy.collider.vy + 0.03).min(max_vy);
        }

        // Accelerate y downward if enemy is less than 50 away from top of screen
        // let dy = enemy.position.1 - state.scroll.1;
        // if dy < 75 {
        //     enemy.collider.vy += 0.03;
        // }

        // Decelerate naturally (due to friction or something)
        // Note that base speed = (0.0, -1.0) due to camera scrolling upward

        if enemy.collider.vx > 0.0 {
            enemy.collider.vx = (enemy.collider.vx - 0.01).max(0.0);
        } else if enemy.collider.vx < 0.0 {
            enemy.collider.vx = (enemy.collider.vx + 0.01).min(0.0);
        }
        if enemy.collider.vy > -1.0 {
            enemy.collider.vy = (enemy.collider.vy - 0.01).max(-1.0);
        } else if enemy.collider.vy < -1.0 {
            enemy.collider.vy = (enemy.collider.vy + 0.01).min(-1.0);
        }
    }
}
