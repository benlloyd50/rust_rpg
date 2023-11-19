use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeightedDrop {
    pub item: String,
    pub chance: u32,
}

