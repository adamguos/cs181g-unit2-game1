use crate::types::Rect;

use std::rc::Rc;

pub struct Animation {
    /**
     * Struct representing an animation sequence
     * frame_rects represent areas of a sprite sheet (handled by Sprite) to draw per tick
     */
    frame_rects: Vec<Rect>,
    frame_times: Vec<usize>,
    total_time: usize,
    loops: bool,
}

impl Animation {
    pub fn new(frame_rects: Vec<Rect>, frame_times: Vec<usize>, loops: bool) -> Self {
        assert!(frame_rects.len() == frame_times.len());

        Animation {
            frame_rects: frame_rects,
            frame_times: frame_times.clone(),
            total_time: frame_times.iter().sum(),
            loops: loops,
        }
    }

    pub fn current_frame(&self, start_time: usize, now: usize) -> Rect {
        // Calculate current frame to display using the current frame number
        let mut frame_index: usize = 0;
        let mut tot = 0;
        let rem = if self.loops {
            (now - start_time) % self.total_time
        } else {
            now
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
}
