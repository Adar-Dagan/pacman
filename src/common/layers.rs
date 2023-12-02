#[derive(Copy, Clone)]
pub enum Layers {
    Map,
    Pellets,
    BonusSymbols,
    OnMapText,
    Player,
    Ghosts,
    GhostsEyes,
    Mask,
    Text,
}

impl Layers {
    pub fn as_f32(&self) -> f32 {
        *self as usize as f32
    }
}
