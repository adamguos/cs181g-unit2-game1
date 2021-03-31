use crate::animation::AnimationSM;
use crate::texture::Texture;
use crate::types::Vec2i;
use std::rc::Rc;

pub struct Sprite {
    image: Rc<Texture>,
    // pub animation: Rc<Animation>,
    pub animation_sm: AnimationSM,
    pub position: Vec2i,
}

impl Sprite {
    pub fn new(image: &Rc<Texture>, animation_sm: AnimationSM, position: Vec2i) -> Self {
        Self {
            image: Rc::clone(image),
            animation_sm: animation_sm,
            position,
        }
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &mut Sprite, cur_frame: usize);
}

use crate::screen::Screen;
impl<'fb> DrawSpriteExt for Screen<'fb> {
    fn draw_sprite(&mut self, s: &mut Sprite, cur_frame: usize) {
        let frame = s
            .animation_sm
            .current_anim(cur_frame)
            .current_frame(cur_frame);

        self.bitblt(&s.image, frame.clone(), s.position);
    }
}
