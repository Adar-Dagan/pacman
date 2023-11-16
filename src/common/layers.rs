#[derive(Copy, Clone)]
pub enum Layers {
    Map,
    Pellets,
    Player,
    Ghosts,
    GhostsEyes,
    Mask = 900,
}

impl Layers {
    pub fn as_f32(&self) -> f32 {
        *self as usize as f32
    }
}
