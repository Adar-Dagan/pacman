pub enum Layers {
    Map,
    Pellets,
    Player,
    Ghosts,
    Mask,
}

impl Layers {
    pub fn as_f32(&self) -> f32 {
        match self {
            Layers::Map => 0.0,
            Layers::Pellets => 10.0,
            Layers::Player => 20.0,
            Layers::Ghosts => 30.0,
            Layers::Mask => 900.0,
        }
    }
}
