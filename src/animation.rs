use crate::types::Rect;

pub struct Animation {
    /**
     * Struct representing an animation sequence
     * frame_rects represent areas of a sprite sheet (handled by Sprite) to draw per tick
     */
    frame_rects: Vec<Rect>,
    frame_times: Vec<usize>,
    start_time: usize,
    total_time: usize,
    loops: bool,
}

impl Animation {
    pub fn new(
        frame_rects: Vec<Rect>,
        frame_times: Vec<usize>,
        start_time: usize,
        loops: bool,
    ) -> Self {
        assert!(frame_rects.len() == frame_times.len());

        Animation {
            frame_rects,
            frame_times: frame_times.clone(),
            start_time,
            total_time: frame_times.iter().sum(),
            loops,
        }
    }

    pub fn current_frame(&self, now: usize) -> Rect {
        // Calculate current frame to display using the current frame number
        let mut frame_index: usize = 0;
        let mut tot = 0;
        let rem = if self.loops {
            (now - self.start_time) % self.total_time
        } else {
            now - self.start_time
        };

        for (i, ft) in self.frame_times.iter().enumerate() {
            if rem <= tot {
                frame_index = i;
                break;
            }
            tot += ft;
        }

        self.frame_rects[frame_index]
    }

    pub fn done(&self, now: usize) -> bool {
        !self.loops && (now - self.start_time >= self.total_time)
    }
}

pub struct AnimationSM {
    /**
     * Struct representing animation state machine.
     *
     * transitions: vector of (src, dest, read), from/to are indices of animation vec
     */
    animations: Vec<Animation>,
    transitions: Vec<(usize, usize, String)>,
    // update_time: usize,
    start_index: usize,
    current_anim: usize,
}

impl AnimationSM {
    pub fn new(
        animations: Vec<Animation>,
        transitions: Vec<(usize, usize, String)>,
        // update_time: usize,
        start_index: usize,
    ) -> Self {
        AnimationSM {
            animations,
            transitions,
            // update_time,
            start_index,
            current_anim: start_index,
        }
    }

    pub fn current_anim(&mut self, now: usize) -> &Animation {
        self.update_anim(now);

        &self.animations[self.current_anim]
    }

    pub fn input(&mut self, input: &str, now: usize) {
        for (src, dest, read) in self.transitions.iter() {
            if *src == self.current_anim && *read == input {
                self.current_anim = *dest;
                self.animations[self.current_anim].start_time = now;
                break;
            }
        }

        // Do nothing if transition is not found
    }

    fn update_anim(&mut self, now: usize) {
        // Update if animation is finished
        // Moves according to first matching transition found, given no input
        // Ignores later ones if there are multiple
        if self.animations[self.current_anim].done(now) {
            let mut transition_found = false;

            for (src, dest, read) in self.transitions.iter() {
                if *src == self.current_anim && *read == "" {
                    self.current_anim = *dest;
                    transition_found = true;
                    break;
                }
            }

            // If no transitions are found, reset to start_index
            if !transition_found {
                self.current_anim = self.start_index;
            }
        }
    }
}
