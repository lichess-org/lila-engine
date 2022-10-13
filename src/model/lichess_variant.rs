use serde::{Deserialize, Serialize};
use shakmaty::variant::Variant;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum LichessVariant {
    #[serde(alias = "antichess")]
    Antichess,
    #[serde(alias = "atomic")]
    Atomic,
    #[serde(alias = "chess960")]
    Chess960,
    #[serde(alias = "crazyhouse")]
    Crazyhouse,
    #[serde(alias = "fromPosition", alias = "From Position")]
    FromPosition,
    #[serde(alias = "horde")]
    Horde,
    #[serde(alias = "kingOfTheHill", alias = "King of the Hill")]
    KingOfTheHill,
    #[serde(alias = "racingKings", alias = "Racing Kings")]
    RacingKings,
    #[serde(alias = "chess", alias = "standard")]
    Standard,
    #[serde(alias = "threeCheck", alias = "Three-check")]
    ThreeCheck,
}

impl From<LichessVariant> for Variant {
    fn from(variant: LichessVariant) -> Variant {
        match variant {
            LichessVariant::Antichess => Variant::Antichess,
            LichessVariant::Atomic => Variant::Atomic,
            LichessVariant::Chess960 | LichessVariant::FromPosition | LichessVariant::Standard => {
                Variant::Chess
            }
            LichessVariant::Crazyhouse => Variant::Crazyhouse,
            LichessVariant::Horde => Variant::Horde,
            LichessVariant::KingOfTheHill => Variant::KingOfTheHill,
            LichessVariant::RacingKings => Variant::RacingKings,
            LichessVariant::ThreeCheck => Variant::ThreeCheck,
        }
    }
}

impl From<Variant> for LichessVariant {
    fn from(variant: Variant) -> LichessVariant {
        match variant {
            Variant::Chess => LichessVariant::Standard,
            Variant::Antichess => LichessVariant::Antichess,
            Variant::Atomic => LichessVariant::Atomic,
            Variant::Crazyhouse => LichessVariant::Crazyhouse,
            Variant::Horde => LichessVariant::Horde,
            Variant::KingOfTheHill => LichessVariant::KingOfTheHill,
            Variant::RacingKings => LichessVariant::RacingKings,
            Variant::ThreeCheck => LichessVariant::ThreeCheck,
        }
    }
}
