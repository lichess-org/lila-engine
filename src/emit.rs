use std::{cmp::min, time::Duration};

use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr, DurationMilliSeconds};
use shakmaty::{uci::Uci, variant::VariantPosition, CastlingMode, Position};

use crate::{
    model::MultiPv,
    uci::{Eval, UciOut},
};

#[serde_as]
#[derive(Clone, Debug, Serialize)]
struct EmitPv {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    moves: Vec<Uci>,
    #[serde(flatten)]
    eval: Eval,
    depth: u32,
}

impl EmitPv {
    fn extract(uci: &UciOut, pos: &VariantPosition) -> (MultiPv, Option<EmitPv>) {
        let multi_pv = match *uci {
            UciOut::Info {
                multipv: Some(multipv),
                ..
            } => multipv,
            _ => MultiPv::default(),
        };

        (
            multi_pv,
            match *uci {
                UciOut::Info {
                    depth: Some(depth),
                    score: Some(ref score),
                    pv: Some(ref pv),
                    ..
                } => (multi_pv > MultiPv::default() || (!score.lowerbound && !score.lowerbound))
                    .then(|| EmitPv {
                        moves: normalize_pv(pv, pos.clone()),
                        eval: pos.turn().fold_wb(score.eval, -score.eval),
                        depth,
                    }),
                _ => None,
            },
        )
    }
}

fn normalize_pv(pv: &[Uci], mut pos: VariantPosition) -> Vec<Uci> {
    let mut moves = Vec::new();
    for uci in pv.iter().take(30) {
        let m = match uci.to_move(&pos) {
            Ok(m) => m,
            Err(_) => break,
        };
        moves.push(m.to_uci(CastlingMode::Chess960));
        pos.play_unchecked(&m);
    }
    moves
}

#[serde_as]
#[derive(Clone, Debug, Default, Serialize)]
pub struct Emit {
    #[serde_as(as = "DurationMilliSeconds")]
    time: Duration,
    depth: u32,
    nodes: u64,
    pvs: Vec<Option<EmitPv>>,
}

impl Emit {
    pub fn update(&mut self, uci: &UciOut, pos: &VariantPosition) {
        let (multi_pv, emit_pv) = EmitPv::extract(&uci, pos);
        if multi_pv <= MultiPv::default() {
            if let UciOut::Info {
                time: Some(time), ..
            } = *uci
            {
                self.time = time;
            }
            if let UciOut::Info {
                depth: Some(depth), ..
            } = *uci
            {
                self.depth = depth;
            }
            if let UciOut::Info {
                nodes: Some(nodes), ..
            } = *uci
            {
                self.nodes = nodes;
            }
            for pv in &mut self.pvs {
                *pv = None;
            }
        } else {
            if let UciOut::Info {
                depth: Some(depth), ..
            } = *uci
            {
                self.depth = min(self.depth, depth);
            }
        }

        let num_pv = usize::from(multi_pv);
        if self.pvs.len() < num_pv {
            self.pvs.resize(num_pv, None);
        }

        if emit_pv.is_some() {
            self.pvs[num_pv - 1] = emit_pv;
        }
    }

    pub fn should_emit(&self) -> bool {
        !self.pvs.is_empty() && self.pvs.iter().all(|pv| pv.is_some())
    }
}
