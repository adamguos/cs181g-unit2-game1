use crate::types::Rect;

pub struct Animation {
    // Do this for the exercise today!
// You'll want to know the frames involved and the timing for each frame
// But then there's also dynamic data, which might live in this struct or might live somewhere else
// An Animation/AnimationState split could be fine, if AnimationState holds the start time and the present frame (or just the start time) and possibly a reference to the Animation
// but there are lots of designs that will work!
}

impl Animation {
    // Should hold some data...
    // Be used to decide what frame to use...
    // And sprites can be updated based on that information.
    // Or we could give sprites an =animation= field instead of a =frame=!
    // Could have a query function like current_frame(&self, start_time:usize, now:usize, speedup_factor:usize)
    // Or could be ticked in-place
}
