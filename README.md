CSCI 181G unit 2 game 1

Adam Guo, Ziang Xue, Gleb Tebaykin

- Collision
There are 3 struct types: `Terrian`, `Mobile` and `Projectile`. In my vision projectiles won't collide with other projectiles but every other possible pairs do collides. The `gather_contacts` is the same as the anim2d lab. -- Ziang

- Main
I think each objects (whether it's a terrain / mobile / projectile) has collsion structs, sprites and other attributes, so I'm wondering what ways should we do to organize all these. Should we have another struct for each of these "objects"? (I'm really not sure about this becuase I'm trying not to think in terms of OOP). We can also simply have arrays of sprites, anims, colliders and stuff and use indices for everything (One drawback I see in this is that for collision there will be seperate arrays for the 3 structs but for everything else we don't need separate arrays). We should definitely find a way to work around this. -- Ziang