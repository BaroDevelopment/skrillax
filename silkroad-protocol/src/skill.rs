use silkroad_serde::*;

#[derive(Deserialize, Copy, Clone)]
pub struct LevelUpMastery {
    pub mastery: u32,
    pub amount: u8,
}

#[derive(Serialize, ByteSize, Copy, Clone)]
#[silkroad(size = 2)]
pub enum LevelUpMasteryError {
    #[silkroad(value = 0x3802)]
    InsufficientSP,
    #[silkroad(value = 0x3804)]
    MasteryMaxLevel,
    #[silkroad(value = 0x3805)]
    ReachedTotalLimit,
}

#[derive(Serialize, ByteSize, Copy, Clone)]
pub enum LevelUpMasteryResponse {
    #[silkroad(value = 1)]
    Success { mastery: u32, new_level: u8 },
    #[silkroad(value = 2)]
    Error(LevelUpMasteryError),
}

#[derive(Deserialize, Copy, Clone)]
pub struct LearnSkill(pub u32);

#[derive(Serialize, ByteSize, Copy, Clone)]
pub enum LearnSkillResponse {
    #[silkroad(value = 1)]
    Success(u32),
    #[silkroad(value = 2)]
    Error(LevelUpMasteryError), // TODO
}
