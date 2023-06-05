use crate::common::RefCommon;
use crate::entity_rarity::EntityRarity;
use crate::{DataEntry, DataMap, FileError, ParseError};
use pk2::Pk2;
use std::str::FromStr;

pub fn load_character_map(pk2: &Pk2) -> Result<DataMap<RefCharacterData>, FileError> {
    DataMap::from(pk2, "/server_dep/silkroad/textdata/CharacterData.txt")
}

#[derive(Clone)]
pub struct RefCharacterData {
    pub common: RefCommon,
    // column 16
    pub rarity: EntityRarity,
    // column 57
    pub level: u8,
    // column 79
    pub exp: u32,
    // column 59
    pub hp: u32,
    // column 50
    pub base_range: u16,
    // column 46
    pub walk_speed: u32,
    // column 47
    pub run_speed: u32,
    // column 48
    pub berserk_speed: u32,
    // column 93
    pub aggressive: bool,
    // column 83-92
    pub skills: Vec<u32>,
}

impl DataEntry for RefCharacterData {
    fn ref_id(&self) -> u32 {
        self.common.ref_id
    }

    fn code(&self) -> &str {
        &self.common.id
    }
}

impl FromStr for RefCharacterData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let common = RefCommon::from_columns(&elements)?;
        let aggressive: u8 = elements.get(93).ok_or(ParseError::MissingColumn(94))?.parse()?;
        let mut skills: Vec<u32> = Vec::new();
        for i in 83..=92 {
            let skill_id: u32 = elements.get(i).ok_or(ParseError::MissingColumn(i as u8))?.parse()?;
            if skill_id != 0 {
                skills.push(skill_id);
            }
        }
        Ok(Self {
            common,
            rarity: elements.get(16).ok_or(ParseError::MissingColumn(16))?.parse()?,
            level: elements.get(57).ok_or(ParseError::MissingColumn(57))?.parse()?,
            exp: elements.get(79).ok_or(ParseError::MissingColumn(79))?.parse()?,
            hp: elements.get(59).ok_or(ParseError::MissingColumn(59))?.parse()?,
            walk_speed: elements.get(46).ok_or(ParseError::MissingColumn(46))?.parse()?,
            run_speed: elements.get(47).ok_or(ParseError::MissingColumn(47))?.parse()?,
            berserk_speed: elements.get(48).ok_or(ParseError::MissingColumn(48))?.parse()?,
            base_range: elements.get(50).ok_or(ParseError::MissingColumn(50))?.parse()?,
            aggressive: aggressive == 1,
            skills,
        })
    }
}
