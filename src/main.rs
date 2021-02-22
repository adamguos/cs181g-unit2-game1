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
use collision::{Contact, Mobile, Projectile, Terrain};
// Texture has our image loading and processing stuff
mod texture;
use texture::Texture;
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
    animations: Vec<Animation>,
    textures: Vec<Rc<Texture>>,
    sprites: Vec<Sprite>,
    terrains: Vec<Terrain>,
    mobiles: Vec<Mobile>,
    projs: Vec<Projectile>,
}
// seconds per frame
const DT: f64 = 1.0 / 60.0;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const DEPTH: usize = 4;

fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Anim2D")
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
    let tex = Rc::new(Texture::with_file(Path::new("content/sample_sprites.jpg")));

    // How many frames have we simulated?
    let mut frame_count: usize = 0;

    let mut state = GameState {
        // initial game state...
        animations: vec![],
        sprites: vec![Sprite::new(
            &tex,
            animation::AnimationSM::new(
                vec![
                    animation::Animation::new(
                        vec![
                            Rect {
                                x: 8,
                                y: 168,
                                w: 48,
                                h: 48,
                            },
                            Rect {
                                x: 72,
                                y: 168,
                                w: 48,
                                h: 48,
                            },
                        ],
                        vec![30, 30],
                        frame_count,
                        true,
                    ),
                    animation::Animation::new(
                        vec![
                            Rect {
                                x: 8,
                                y: 328,
                                w: 32,
                                h: 32,
                            },
                            Rect {
                                x: 48,
                                y: 328,
                                w: 32,
                                h: 32,
                            },
                            Rect {
                                x: 89,
                                y: 333,
                                w: 23,
                                h: 25,
                            },
                            Rect {
                                x: 120,
                                y: 336,
                                w: 16,
                                h: 16,
                            },
                        ],
                        vec![15, 15, 15, 15],
                        frame_count,
                        false,
                    ),
                ],
                vec![(0, 1, String::from("jump")), (1, 0, String::from(""))],
                frame_count,
                0,
            ),
            Vec2i(90, 100),
        )],
        textures: vec![tex],
        terrains: vec![],
        mobiles: vec![],
        projs: vec![],
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
            let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH);
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
    screen.rect(
        Rect {
            x: 100,
            y: 100,
            w: 32,
            h: 64,
        },
        Rgba(128, 0, 0, 255),
    );
    screen.line(Vec2i(0, 150), Vec2i(300, 200), Rgba(0, 128, 0, 255));
    for s in state.sprites.iter_mut() {
        screen.draw_sprite(s, frame_count);
    }
}

fn update_game(state: &mut GameState, input: &WinitInputHelper, frame: usize) {
    // Player control goes here
    if input.key_held(VirtualKeyCode::Right) {
        state.sprites[0].position.0 += 2;
    }
    if input.key_held(VirtualKeyCode::Left) {
        state.sprites[0].position.0 -= 2;
    }
    if input.key_held(VirtualKeyCode::Up) {
        state.sprites[0].position.1 -= 2;
    }
    if input.key_held(VirtualKeyCode::Down) {
        state.sprites[0].position.1 += 2;
    }
    if input.key_pressed(VirtualKeyCode::Space) {
        state.sprites[0].animation_sm.input("jump", frame);
    }
    // Update player position

    // Detect collisions: Generate contacts
    let mut contacts: Vec<Contact> = vec![];
    collision::gather_contacts(&state.terrains, &state.mobiles, &state.projs, &mut contacts);

    // Handle collisions: Apply restitution impulses.
    for contact in contacts.iter() {
        // let colliders = (contact.b, contact.a);
    }

    // Update game rules: What happens when the player touches things?
}
