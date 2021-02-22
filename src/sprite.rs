use crate::animation::Animation;
use crate::texture::Texture;
use crate::types::Vec2i;
use std::rc::Rc;

pub struct Sprite {
    image: Rc<Texture>,
    pub animation: Rc<Animation>,
    pub position: Vec2i,
    start_frame: usize,
}

impl Sprite {
    pub fn new(
        image: &Rc<Texture>,
        animation: &Rc<Animation>,
        position: Vec2i,
        start_frame: usize,
    ) -> Self {
        Self {
            image: Rc::clone(image),
            animation: Rc::clone(animation),
            position,
            start_frame,
        }
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite, cur_frame: usize);
}

use crate::screen::Screen;
impl<'fb> DrawSpriteExt for Screen<'fb> {
    fn draw_sprite(&mut self, s: &Sprite, cur_frame: usize) {
        let frame = s.animation.current_frame(s.start_frame, cur_frame);
        self.bitblt(&s.image, frame.clone(), s.position);
    }
}
