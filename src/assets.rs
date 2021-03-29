use rand::{thread_rng, Rng};
use std::rc::Rc;

use crate::animation::*;
use crate::collision::*;
use crate::entity::*;
use crate::screen::*;
use crate::sprite::*;
use crate::texture::*;
use crate::types::*;

pub fn player_anim(sprite_sheet: &Rc<Texture>, frame_count: usize) -> Sprite {
    Sprite::new(
        &sprite_sheet,
        AnimationSM::new(
            vec![Animation::new(
                vec![Rect {
                    x: 502,
                    y: 991,
                    w: 36,
                    h: 25,
                }],
                vec![60],
                frame_count,
                true,
            )],
            vec![],
            frame_count,
            0,
        ),
        Vec2i(180, 500),
    )
}

pub fn enemy_entity(sprite_sheet: &Rc<Texture>, frame_count: usize, pos: Vec2i) -> Entity<Mobile> {
    let sprite_rects = vec![
        Rect {
            x: 535,
            y: 150,
            w: 32,
            h: 25,
        },
        Rect {
            x: 775,
            y: 301,
            w: 32,
            h: 25,
        },
        Rect {
            x: 777,
            y: 327,
            w: 32,
            h: 25,
        },
        Rect {
            x: 482,
            y: 358,
            w: 32,
            h: 25,
        },
    ];

    let mut rng = rand::thread_rng();
    let sprite_i = rng.gen_range(0..sprite_rects.len());

    Entity::new(
        Sprite::new(
            &sprite_sheet,
            AnimationSM::new(
                vec![Animation::new(
                    vec![sprite_rects[sprite_i]],
                    vec![60],
                    frame_count,
                    true,
                )],
                vec![],
                frame_count,
                0,
            ),
            pos,
        ),
        pos,
        Mobile::enemy(
            Rect {
                x: pos.0,
                y: pos.1,
                w: 32,
                h: 25,
            },
            0.0,
            3.0,
            60,
        ),
    )
}

pub fn walls_vec(screen_w: u16, screen_h: u16) -> Vec<Wall> {
    vec![
        Wall::new(Rect {
            x: -64,
            y: -64,
            w: 64,
            h: screen_h + 128,
        }),
        Wall::new(Rect {
            x: screen_w as i32,
            y: -64,
            w: 64,
            h: screen_h + 128,
        }),
        /*
        Wall::new(Rect {
            x: 0,
            y: -64,
            w: screen_w,
            h: 64,
        }),
        */
        Wall::new(Rect {
            x: 0,
            y: screen_h as i32,
            w: screen_w,
            h: 64,
        }),
    ]
}

pub fn boulder_entity(
    sprite_sheet: &Rc<Texture>,
    frame_count: usize,
    pos: Vec2i,
) -> Entity<Terrain> {
    Entity::new(
        Sprite::new(
            &sprite_sheet,
            AnimationSM::new(
                vec![Animation::new(
                    vec![Rect {
                        x: 48,
                        y: 320,
                        w: 32,
                        h: 32,
                    }],
                    vec![60],
                    frame_count,
                    true,
                )],
                vec![],
                frame_count,
                0,
            ),
            pos,
        ),
        pos,
        Terrain::new(
            Rect {
                x: pos.0,
                y: pos.1,
                w: 32,
                h: 32,
            },
            frame_count,
            false,
            1,
        ),
    )
}

pub fn rock_entity(sprite_sheet: &Rc<Texture>, frame_count: usize, pos: Vec2i) -> Entity<Terrain> {
    Entity::new(
        Sprite::new(
            &sprite_sheet,
            AnimationSM::new(
                vec![
                    Animation::new(
                        vec![Rect {
                            x: 368,
                            y: 128,
                            w: 16,
                            h: 16,
                        }],
                        vec![60],
                        frame_count,
                        true,
                    ),
                    Animation::new(
                        vec![Rect {
                            x: 368,
                            y: 144,
                            w: 16,
                            h: 16,
                        }],
                        vec![60],
                        frame_count,
                        true,
                    ),
                    Animation::new(
                        vec![Rect {
                            x: 368,
                            y: 160,
                            w: 16,
                            h: 16,
                        }],
                        vec![60],
                        frame_count,
                        true,
                    ),
                    Animation::new(
                        vec![Rect {
                            x: 368,
                            y: 176,
                            w: 16,
                            h: 16,
                        }],
                        vec![60],
                        frame_count,
                        true,
                    ),
                ],
                vec![
                    (0, 1, String::from("hit")),
                    (1, 2, String::from("hit")),
                    (2, 3, String::from("hit")),
                ],
                frame_count,
                0,
            ),
            pos,
        ),
        pos,
        Terrain::new(
            Rect {
                x: pos.0,
                y: pos.1,
                w: 16,
                h: 16,
            },
            frame_count,
            true,
            40,
        ),
    )
}

pub fn get_font_letter(c: char) -> Option<Rect> {
    if c.is_lowercase() {
        Some(Rect {
            x: (c as i32 - 'a' as i32) * 18 + 9,
            y: 5,
            w: 18,
            h: 18,
        })
    } else if c.is_uppercase() {
        Some(Rect {
            x: (c as i32 - 'A' as i32) * 18 + 9,
            y: 23,
            w: 18,
            h: 18,
        })
    } else if c.is_numeric() {
        Some(Rect {
            x: (c as i32 - '0' as i32) * 18 + 9,
            y: 41,
            w: 18,
            h: 18,
        })
    } else {
        None
    }
}

pub fn draw_string(
    string: &str,
    screen: &mut Screen,
    font_sheet: &Rc<Texture>,
    pos: Vec2i,
    scroll: Vec2i,
) {
    for (i, c) in string.chars().enumerate() {
        match get_font_letter(c) {
            None => {}
            Some(rect) => {
                screen.bitblt(
                    font_sheet,
                    rect,
                    Vec2i(pos.0 + 18 * i as i32, scroll.1 + pos.1),
                );
            }
        }
    }
}
