use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters, Clone)]
#[getset(get = "pub")]
pub struct TargetIndex {
    pub index_name: String,
    pub duration_days: u32,
}

#[derive(Debug, Deserialize, Serialize, Getters, Clone)]
#[getset(get = "pub")]
pub struct TargetIndexList {
    pub index: Vec<TargetIndex>,
}
