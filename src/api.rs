use std::{cmp::min, num::NonZeroU32};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, FromInto, TryFromInto};
use shakmaty::{
    fen::Fen,
    uci::{IllegalUciError, Uci},
    variant::{Variant, VariantPosition},
    CastlingMode, EnPassantMode, Position as _, PositionError,
};
use thiserror::Error;

use crate::model::{ClientSecret, Engine, JobId, MultiPv, ProviderSecret, SessionId, UciVariant};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Search {
    Movetime(u32),
    Depth(u32),
    Nodes(u64)
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    session_id: SessionId,
    threads: NonZeroU32,
    hash: NonZeroU32,
    #[serde(flatten)]
    search: Search,
    #[serde_as(as = "TryFromInto<u32>")]
    multi_pv: MultiPv,
    #[serde_as(as = "FromInto<UciVariant>")]
    variant: Variant,
    #[serde_as(as = "DisplayFromStr")]
    initial_fen: Fen,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    moves: Vec<Uci>,
}

#[derive(Error, Debug)]
pub enum InvalidWorkError {
    #[error("illegal initial position: {0}")]
    Position(#[from] PositionError<VariantPosition>),
    #[error("illegal uci: {0}")]
    IllegalUci(#[from] IllegalUciError),
    #[error("too many moves")]
    TooManyMoves,
    #[error("unsupported variant")]
    UnsupportedVariant,
}

impl Work {
    pub fn sanitize(self, engine: &Engine) -> Result<(Work, VariantPosition), InvalidWorkError> {
        if !engine
            .config
            .variants
            .iter()
            .copied()
            .any(|v| v == self.variant)
        {
            return Err(InvalidWorkError::UnsupportedVariant);
        }

        let mut pos = VariantPosition::from_setup(
            self.variant,
            self.initial_fen.into_setup(),
            CastlingMode::Chess960,
        )?;
        let initial_fen = Fen(pos.clone().into_setup(EnPassantMode::Legal));

        if self.moves.len() > 600 {
            return Err(InvalidWorkError::TooManyMoves);
        }
        let mut moves = Vec::with_capacity(self.moves.len());
        for uci in self.moves {
            let m = uci.to_move(&pos)?;
            moves.push(m.to_uci(CastlingMode::Chess960));
            pos.play_unchecked(&m);
        }

        Ok((
            Work {
                session_id: self.session_id,
                threads: min(self.threads, engine.config.max_threads),
                hash: min(self.hash, engine.config.max_hash),
                search: self.search,
                multi_pv: self.multi_pv,
                variant: self.variant,
                initial_fen,
                moves,
            },
            pos,
        ))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnalyseRequest {
    pub client_secret: ClientSecret,
    pub work: Work,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcquireRequest {
    pub provider_secret: ProviderSecret,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcquireResponse {
    pub id: JobId,
    pub work: Work,
    pub engine: Engine,
}
