use std::cmp::max;

use crate::types::Rect;

// seconds per frame
const DT: f64 = 1.0 / 60.0;

const DEPTH: usize = 4;
const WIDTH: usize = 512;
const HEIGHT: usize = 480;
const PITCH: usize = WIDTH * DEPTH;

// We'll make our Color type an RGBA8888 pixel.
type Color = [u8; DEPTH];

const CLEAR_COL: Color = [32, 32, 64, 255];
const WALL_COL: Color = [200, 200, 200, 255];
const PLAYER_COL: Color = [255, 128, 128, 255];

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ColliderID {
    Terrain(usize),
    Mobile(usize),
    Projectile(usize),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct Contact {
    a: ColliderID,
    b: ColliderID,
    mtv: (i32, i32),
}

/*  I think we will be doing level generations, so "mobile" will have the
    ability to move both vertically and horizontally (but for now I assume that
    the player can move horizontally and the enemy can move vetically and
    horizontally.) The terrain will move horizontally as they are generated and
    as the player "advances"
*/

/*
   We will mostly be treating terrain as blocks, possibly in rectangle shapes to simplify. It does not need a speed. If with generations it has to move we can constantly change its position based on frame changes.
*/
pub(crate) struct Terrain {
    rect: Rect,
    hp: usize,
}
impl Terrain {
    fn new(x: i32, y: i32) -> Self {
        Self {
            rect: Rect {
                x: x,
                y: y,
                w: 20,
                h: 20,
            },
            hp: 300,
        }
    }
}

/*
   Mobiles would need to be able to move freely. We would require its hitbox to be rect.
*/
pub(crate) struct Mobile {
    rect: Rect,
    vx: i32,
    vy: i32,
    hp: usize,
}
impl Mobile {
    pub fn enemy(x: i32, y: i32, hp: usize) -> Self {
        Self {
            rect: Rect {
                x: x,
                y: y,
                w: 30,
                h: 30,
            },
            vx: 0,
            vy: 4,
            hp: 50,
        }
    }

    pub fn player(x: i32, y: i32) -> Self {
        Self {
            rect: Rect {
                x: x,
                y: y,
                w: 40,
                h: 40,
            },
            vx: 0,
            vy: 0,
            hp: 100,
        }
    }
}

/*
    Projectiles can cross each others and they will only collide with terrains and mobiles. Since we might need it to point clearly the speed should be floats. (subject to change.)
*/
pub(crate) struct Projectile {
    pub(crate) rect: Rect,
    vx: f64,
    vy: f64,
    hp: usize,
}

impl Projectile {
    pub(crate) fn new(from: &Mobile) -> Self {
        Self {
            rect: Rect {
                x: from.rect.x,
                y: from.rect.y - 10,
                w: 5,
                h: 5,
            },
            vx: 0.0,
            vy: -10.0,
            hp: 10,
        }
    }
}

// pixels gives us an rgba8888 framebuffer
fn clear(fb: &mut [u8], c: Color) {
    // Four bytes per pixel; chunks_exact_mut gives an iterator over 4-element slices.
    // So this way we can use copy_from_slice to copy our color slice into px very quickly.
    for px in fb.chunks_exact_mut(4) {
        px.copy_from_slice(&c);
    }
}

#[allow(dead_code)]
fn rect(fb: &mut [u8], r: Rect, c: Color) {
    assert!(r.x < WIDTH as i32);
    assert!(r.y < HEIGHT as i32);
    // NOTE, very fragile! will break for out of bounds rects!  See next week for the fix.
    let x1 = (r.x + r.w as i32).min(WIDTH as i32) as usize;
    let y1 = (r.y + r.h as i32).min(HEIGHT as i32) as usize;
    for row in fb[(r.y as usize * PITCH)..(y1 * PITCH)].chunks_exact_mut(PITCH) {
        for p in row[(r.x as usize * DEPTH)..(x1 * DEPTH)].chunks_exact_mut(DEPTH) {
            p.copy_from_slice(&c);
        }
    }
}

// Here we will be using push() on into, so it can't be a slice
pub(crate) fn gather_contacts(
    terrains: &[Terrain],
    mobiles: &[Mobile],
    projs: &[Projectile],
    into: &mut Vec<Contact>,
) {
    // collide mobiles against mobiles
    for (ai, a) in mobiles.iter().enumerate() {
        for (bi, b) in mobiles.iter().enumerate().skip(ai + 1) {
            if !separating_axis(
                a.rect.x,
                a.rect.x + a.rect.w as i32,
                b.rect.x,
                b.rect.x + b.rect.w as i32,
            ) && !separating_axis(
                a.rect.y,
                a.rect.y + a.rect.h as i32,
                b.rect.y,
                b.rect.y + b.rect.h as i32,
            ) {
                let contact = Contact {
                    a: ColliderID::Mobile(ai),
                    b: ColliderID::Mobile(bi),
                    mtv: (0, 0),
                };

                into.push(contact);
            }
        }
    }
    // collide mobiles against terrains
    for (ai, a) in mobiles.iter().enumerate() {
        for (bi, b) in terrains.iter().enumerate() {
            if !separating_axis(
                a.rect.x,
                a.rect.x + a.rect.w as i32,
                b.rect.x,
                b.rect.x + b.rect.w as i32,
            ) && !separating_axis(
                a.rect.y,
                a.rect.y + a.rect.h as i32,
                b.rect.y,
                b.rect.y + b.rect.h as i32,
            ) {
                let contact = Contact {
                    a: ColliderID::Mobile(ai),
                    b: ColliderID::Terrain(bi),
                    mtv: (0, 0),
                };

                into.push(contact);
            }
        }
    }
    // collide projs against mobiles
    for (ai, a) in projs.iter().enumerate() {
        for (bi, b) in mobiles.iter().enumerate() {
            if !separating_axis(
                a.rect.x,
                a.rect.x + a.rect.w as i32,
                b.rect.x,
                b.rect.x + b.rect.w as i32,
            ) && !separating_axis(
                a.rect.y,
                a.rect.y + a.rect.h as i32,
                b.rect.y,
                b.rect.y + b.rect.h as i32,
            ) {
                let contact = Contact {
                    a: ColliderID::Projectile(ai),
                    b: ColliderID::Mobile(bi),
                    mtv: (0, 0),
                };

                into.push(contact);
            }
        }
    }
    // collide projs against terrains
    for (ai, a) in projs.iter().enumerate() {
        for (bi, b) in terrains.iter().enumerate() {
            if !separating_axis(
                a.rect.x,
                a.rect.x + a.rect.w as i32,
                b.rect.x,
                b.rect.x + b.rect.w as i32,
            ) && !separating_axis(
                a.rect.y,
                a.rect.y + a.rect.h as i32,
                b.rect.y,
                b.rect.y + b.rect.h as i32,
            ) {
                let contact = Contact {
                    a: ColliderID::Projectile(ai),
                    b: ColliderID::Terrain(bi),
                    mtv: (0, 0),
                };

                into.push(contact);
            }
        }
    }
}

/*
Modify the hp of the objects and remove unnecessary objects.
*/
pub(crate) fn handle_contact(
    terrains: &mut Vec<Terrain>,
    mobiles: &mut Vec<Mobile>,
    projs: &mut Vec<Projectile>,
    contacts: &mut Vec<Contact>,
) -> bool {
    // We first modify the hp of the collision objects.
    for contact in contacts.iter() {
        match (contact.a, contact.b) {
            // By design a contact will always be MM MT PM PT
            // MT collide will kill the mobile
            // MM collide will destroy the lower hp mobile and cause 30 pt damage to the higher hp mobile
            (ColliderID::Mobile(a), ColliderID::Terrain(b)) => {
                mobiles[a].hp = 0;
            }
            (ColliderID::Mobile(a), ColliderID::Mobile(b)) => {
                if mobiles[a].hp > mobiles[b].hp {
                    mobiles[b].hp = 0;
                    mobiles[a].hp = if mobiles[a].hp >= 30 {
                        mobiles[a].hp - 30
                    } else {
                        0
                    };
                } else {
                    mobiles[a].hp = 0;
                    mobiles[b].hp = if mobiles[b].hp >= 30 {
                        mobiles[b].hp - 30
                    } else {
                        0
                    };
                }
            }
            (ColliderID::Projectile(a), ColliderID::Terrain(b)) => {
                if terrains[b].hp >= projs[a].hp {
                    terrains[b].hp -= projs[a].hp;
                } else {
                    terrains[b].hp = 0;
                }
                projs[a].hp = 0;
            }
            (ColliderID::Projectile(a), ColliderID::Mobile(b)) => {
                if mobiles[b].hp >= projs[a].hp {
                    mobiles[b].hp -= projs[a].hp;
                } else {
                    mobiles[b].hp = 0;
                }
                projs[a].hp = 0;
            }
            _ => {}
        }
    }
    terrains.retain(|terrain| terrain.hp > 0);
    mobiles.retain(|mobile| mobile.hp > 0);
    projs.retain(|proj| proj.hp > 0);
    return true;
}

fn restitute(statics: &[Terrain], dynamics: &mut [Mobile], contacts: &mut [Contact]) {
    // handle restitution of dynamics against dynamics and dynamics against statics wrt contacts.
    // You could instead make contacts `Vec<Contact>` if you think you might remove contacts.
    // You could also add an additional parameter, a slice or vec representing how far we've displaced each dynamic, to avoid allocations if you track a vec of how far things have been moved.
    // You might also want to pass in another &mut Vec<Contact> to be filled in with "real" touches that actually happened.
    contacts.sort_unstable_by_key(|c| -(c.mtv.0 * c.mtv.0 + c.mtv.1 * c.mtv.1));
    // Keep going!  Note that you can assume every contact has a dynamic object in .a.
    // You might decide to tweak the interface of this function to separately take dynamic-static and dynamic-dynamic contacts, to avoid a branch inside of the response calculation.
    // Or, you might decide to calculate signed mtvs taking direction into account instead of the unsigned displacements from rect_displacement up above.  Or calculate one MTV per involved entity, then apply displacements to both objects during restitution (sorting by the max or the sum of their magnitudes)
}

fn separating_axis(ax1: i32, ax2: i32, bx1: i32, bx2: i32) -> bool {
    assert!(ax1 <= ax2 && bx1 <= bx2);
    ax2 < bx1 || bx2 < ax1
}
