use std::rc::Rc;

use crate::animation::*;
use crate::collision::*;
use crate::entity::*;
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

pub fn wall_entity(sprite_sheet: &Rc<Texture>, frame_count: usize, pos: Vec2i) -> Entity<Terrain> {
    Entity {
        sprite: Sprite::new(
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
        position: pos,
        collider: Terrain {
            rect: Rect {
                x: pos.0,
                y: pos.1,
                w: 32,
                h: 32,
            },
            destructible: false,
            hp: 1,
        },
    }
}

pub fn rock_entity(sprite_sheet: &Rc<Texture>, frame_count: usize, pos: Vec2i) -> Entity<Terrain> {
    Entity {
        sprite: Sprite::new(
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
        position: pos,
        collider: Terrain {
            rect: Rect {
                x: pos.0,
                y: pos.1,
                w: 16,
                h: 16,
            },
            destructible: true,
            hp: 40,
        },
    }
}
