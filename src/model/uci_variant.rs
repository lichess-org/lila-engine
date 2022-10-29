use serde::{Deserialize, Serialize};
use shakmaty::variant::Variant;

#[derive(Deserialize, Serialize)]
pub enum UciVariant {
    #[serde(
        rename = "chess",
        alias = "standard",
        alias = "chess960",
        alias = "fromPosition"
    )]
    Chess,
    #[serde(rename = "antichess")]
    Antichess,
    #[serde(rename = "atomic")]
    Atomic,
    #[serde(rename = "crazyhouse")]
    Crazyhouse,
    #[serde(rename = "horde")]
    Horde,
    #[serde(rename = "kingofthehill", alias = "kingOfTheHill")]
    KingOfTheHill,
    #[serde(rename = "racingkings", alias = "racingKings")]
    RacingKings,
    #[serde(rename = "3check", alias = "threeCheck")]
    ThreeCheck,
}

impl From<UciVariant> for Variant {
    fn from(value: UciVariant) -> Variant {
        match value {
            UciVariant::Chess => Variant::Chess,
            UciVariant::Antichess => Variant::Antichess,
            UciVariant::Atomic => Variant::Atomic,
            UciVariant::Crazyhouse => Variant::Crazyhouse,
            UciVariant::Horde => Variant::Horde,
            UciVariant::KingOfTheHill => Variant::KingOfTheHill,
            UciVariant::RacingKings => Variant::RacingKings,
            UciVariant::ThreeCheck => Variant::ThreeCheck,
        }
    }
}

impl From<Variant> for UciVariant {
    fn from(value: Variant) -> UciVariant {
        match value {
            Variant::Chess => UciVariant::Chess,
            Variant::Antichess => UciVariant::Antichess,
            Variant::Atomic => UciVariant::Atomic,
            Variant::Crazyhouse => UciVariant::Crazyhouse,
            Variant::Horde => UciVariant::Horde,
            Variant::KingOfTheHill => UciVariant::KingOfTheHill,
            Variant::RacingKings => UciVariant::RacingKings,
            Variant::ThreeCheck => UciVariant::ThreeCheck,
        }
    }
}
