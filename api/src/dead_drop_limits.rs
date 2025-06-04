use std::num::NonZeroU32;

#[derive(Clone)]
pub struct DeadDropLimits {
    pub j2u_dead_drops_per_request_limit: NonZeroU32,
    pub u2j_dead_drops_per_request_limit: NonZeroU32,
}

impl DeadDropLimits {
    pub fn new(
        j2u_dead_drops_per_request_limit: NonZeroU32,
        u2j_dead_drops_per_request_limit: NonZeroU32,
    ) -> Self {
        Self {
            j2u_dead_drops_per_request_limit,
            u2j_dead_drops_per_request_limit,
        }
    }
}
