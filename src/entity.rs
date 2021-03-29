use crate::collision::Collider;
use crate::sprite::Sprite;
use crate::types::Vec2i;

pub struct Entity<T: Collider> {
    pub sprite: Sprite,
    pub position: Vec2i,
    pub collider: T,
}

/*
One issue is that sprite, entity and collider all have its own position info, and if these positions are out of alignment, it is almost certain that something would go wrong.

Hence, when we initialize the entity, we must align the position informations.
*/

impl<T: Collider> Entity<T> {
    pub fn new(sprite: Sprite, position: Vec2i, collider: T) -> Self {
        let mut this_entity = Entity {
            sprite,
            position,
            collider,
        };
        this_entity.align();
        this_entity
    }

    pub fn move_pos(&mut self, dx: i32, dy: i32) {
        self.sprite.position.0 += dx;
        self.sprite.position.1 += dy;

        self.collider.move_pos(dx, dy);

        self.position.0 += dx;
        self.position.1 += dy;
    }

    fn align(&mut self) {
        if self.sprite.position.0 != self.position.0 {
            self.sprite.position.0 = self.position.0;
        }
        if self.sprite.position.1 != self.position.1 {
            self.sprite.position.1 = self.position.1;
        }
        self.collider.set_pos(self.position.0, self.position.1);
    }
}
