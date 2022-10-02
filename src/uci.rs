use std::{collections::HashMap, fmt, num::ParseIntError, time::Duration};

use memchr::{memchr2, memchr2_iter};
use shakmaty::uci::{ParseUciError, Uci};
use thiserror::Error;

use crate::api::{InvalidMultiPvError, MultiPv};

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("unexpected token")]
    UnexpectedToken,
    #[error("unexpected line break in uci command")]
    UnexpectedLineBreak,
    #[error("unexpected end of line")]
    UnexpectedEndOfLine,
    #[error("invalid move: {0}")]
    InvalidMove(#[from] ParseUciError),
    #[error("invalid integer: {0}")]
    InvalidInteger(#[from] ParseIntError),
    #[error("invalid multipv: {0}")]
    InvalidMultipv(#[from] InvalidMultiPvError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    eval: Eval,
    lowerbound: bool,
    upperbound: bool,
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.eval.fmt(f)?;
        if self.lowerbound {
            f.write_str(" lowerbound")?;
        }
        if self.upperbound {
            f.write_str(" upperbound")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eval {
    Cp(i64),
    Mate(i32),
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eval::Cp(cp) => write!(f, "cp {cp}"),
            Eval::Mate(mate) => write!(f, "mate {mate}"),
        }
    }
}

#[derive(Debug)]
pub enum UciOut {
    Bestmove {
        m: Option<Uci>,
        ponder: Option<Uci>,
    },
    Info {
        multipv: Option<MultiPv>,
        depth: Option<u32>,
        seldepth: Option<u32>,
        time: Option<Duration>,
        nodes: Option<u64>,
        score: Option<Score>,
        currmove: Option<Uci>,
        currmovenumber: Option<u32>,
        hashfull: Option<u32>,
        nps: Option<u64>,
        tbhits: Option<u64>,
        sbhits: Option<u64>,
        cpuload: Option<u32>,
        refutation: HashMap<Uci, Vec<Uci>>,
        currline: HashMap<u32, Vec<Uci>>,
        pv: Option<Vec<Uci>>,
        string: Option<String>,
    },
}

impl UciOut {
    pub fn from_line(s: &str) -> Result<Option<UciOut>, ProtocolError> {
        Parser::new(s)?.parse_out()
    }
}

impl fmt::Display for UciOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UciOut::Bestmove { m, ponder } => {
                match m {
                    Some(m) => write!(f, "bestmove {m}")?,
                    None => f.write_str("bestmove (none)")?,
                }
                if let Some(ponder) = ponder {
                    write!(f, " ponder {ponder}")?;
                }
                Ok(())
            }
            UciOut::Info {
                multipv,
                depth,
                seldepth,
                time,
                nodes,
                score,
                currmove,
                currmovenumber,
                hashfull,
                nps,
                tbhits,
                sbhits,
                cpuload,
                refutation,
                currline,
                pv,
                string,
            } => {
                f.write_str("info")?;
                if let Some(multipv) = multipv {
                    write!(f, " multipv {multipv}")?;
                }
                if let Some(depth) = depth {
                    write!(f, " depth {depth}")?;
                }
                if let Some(seldepth) = seldepth {
                    write!(f, " seldepth {seldepth}")?;
                }
                if let Some(time) = time {
                    write!(f, " time {}", time.as_millis())?;
                }
                if let Some(nodes) = nodes {
                    write!(f, " nodes {nodes}")?;
                }
                if let Some(score) = score {
                    write!(f, " score {score}")?;
                }
                if let Some(currmove) = currmove {
                    write!(f, " currmove {currmove}")?;
                }
                if let Some(currmovenumber) = currmovenumber {
                    write!(f, " currmovenumber {currmovenumber}")?;
                }
                if let Some(hashfull) = hashfull {
                    write!(f, " hashfull {hashfull}")?;
                }
                if let Some(nps) = nps {
                    write!(f, " nps {nps}")?;
                }
                if let Some(tbhits) = tbhits {
                    write!(f, " tbhits {tbhits}")?;
                }
                if let Some(sbhits) = sbhits {
                    write!(f, " sbhits {sbhits}")?;
                }
                if let Some(cpuload) = cpuload {
                    write!(f, " cpuload {cpuload}")?;
                }
                for (refuted, refuted_by) in refutation {
                    write!(f, " refutation {refuted}")?;
                    for m in refuted_by {
                        write!(f, " {m}")?;
                    }
                }
                for (cpunr, currline) in currline {
                    write!(f, " currline {cpunr}")?;
                    for m in currline {
                        write!(f, " {m}")?;
                    }
                }
                if let Some(pv) = pv {
                    f.write_str(" pv")?;
                    for m in pv {
                        write!(f, " {m}")?;
                    }
                }
                if let Some(string) = string {
                    write!(f, " string {string}")?;
                }
                Ok(())
            }
        }
    }
}

struct Parser<'a> {
    s: &'a str,
}

impl<'a> Iterator for Parser<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let (head, tail) = read(self.s);
        self.s = tail;
        head
    }
}

impl<'a> Parser<'a> {
    pub fn new(s: &str) -> Result<Parser<'_>, ProtocolError> {
        match memchr2(b'\r', b'\n', s.as_bytes()) {
            Some(_) => Err(ProtocolError::UnexpectedLineBreak),
            None => Ok(Parser { s }),
        }
    }

    fn peek(&self) -> Option<&str> {
        let (head, _) = read(self.s);
        head
    }

    fn until<P>(&mut self, pred: P) -> Option<&str>
    where
        P: FnMut(&'a str) -> bool,
    {
        let (head, tail) = read_until(self.s, pred);
        self.s = tail;
        head
    }

    fn parse_moves(&mut self) -> Vec<Uci> {
        let mut moves = Vec::new();
        while let Some(m) = self.peek() {
            match m.parse() {
                Ok(uci) => {
                    self.next();
                    moves.push(uci);
                }
                Err(_) => break,
            }
        }
        moves
    }

    fn parse_bestmove(&mut self) -> Result<UciOut, ProtocolError> {
        Ok(UciOut::Bestmove {
            m: match self.next() {
                Some("(none)") | None => None,
                Some(m) => Some(m.parse()?),
            },
            ponder: match self.next() {
                Some("ponder") => match self.next() {
                    Some("(none)") | None => None,
                    Some(m) => Some(m.parse()?),
                },
                Some(_) => return Err(ProtocolError::UnexpectedToken),
                None => None,
            },
        })
    }

    fn parse_score(&mut self) -> Result<Score, ProtocolError> {
        let eval = match self.next() {
            Some("cp") => Eval::Cp(
                self.next()
                    .ok_or(ProtocolError::UnexpectedEndOfLine)?
                    .parse()?,
            ),
            Some("mate") => Eval::Mate(
                self.next()
                    .ok_or(ProtocolError::UnexpectedEndOfLine)?
                    .parse()?,
            ),
            Some(_) => return Err(ProtocolError::UnexpectedToken),
            None => return Err(ProtocolError::UnexpectedEndOfLine),
        };
        let mut lowerbound = false;
        let mut upperbound = false;
        while let Some(token) = self.peek() {
            match token {
                "lowerbound" => {
                    self.next();
                    lowerbound = true;
                }
                "upperbound" => {
                    self.next();
                    upperbound = true;
                }
                _ => break,
            }
        }
        Ok(Score {
            eval,
            lowerbound,
            upperbound,
        })
    }

    fn parse_info(&mut self) -> Result<UciOut, ProtocolError> {
        let mut multipv = None;
        let mut depth = None;
        let mut seldepth = None;
        let mut time = None;
        let mut nodes = None;
        let mut score = None;
        let mut currmove = None;
        let mut currmovenumber = None;
        let mut hashfull = None;
        let mut nps = None;
        let mut tbhits = None;
        let mut sbhits = None;
        let mut cpuload = None;
        let mut refutation = HashMap::new();
        let mut currline = HashMap::new();
        let mut pv = None;
        let mut string = None;
        loop {
            match self.next() {
                Some("multipv") => {
                    multipv = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse::<u32>()?
                            .try_into()?,
                    )
                }
                Some("depth") => {
                    depth = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("seldepth") => {
                    seldepth = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("time") => {
                    time = Some(Duration::from_millis(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    ))
                }
                Some("nodes") => {
                    nodes = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("score") => score = Some(self.parse_score()?),
                Some("currmove") => {
                    currmove = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("currmovenumber") => {
                    currmovenumber = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("hashfull") => {
                    hashfull = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("nps") => {
                    nps = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("tbhits") => {
                    tbhits = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("sbhits") => {
                    sbhits = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("cpuload") => {
                    cpuload = Some(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                    )
                }
                Some("refutation") => {
                    refutation.insert(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                        self.parse_moves(),
                    );
                }
                Some("currline") => {
                    currline.insert(
                        self.next()
                            .ok_or(ProtocolError::UnexpectedEndOfLine)?
                            .parse()?,
                        self.parse_moves(),
                    );
                }
                Some("pv") => pv = Some(self.parse_moves()),
                Some("string") => {
                    string = Some(self.until(|_| false).unwrap_or_default().to_owned())
                }
                Some(_) => return Err(ProtocolError::UnexpectedToken),
                None => break,
            }
        }
        Ok(UciOut::Info {
            multipv,
            depth,
            seldepth,
            time,
            nodes,
            score,
            currmove,
            currmovenumber,
            hashfull,
            nps,
            tbhits,
            sbhits,
            cpuload,
            refutation,
            currline,
            pv,
            string,
        })
    }

    fn parse_out(&mut self) -> Result<Option<UciOut>, ProtocolError> {
        Ok(Some(match self.next() {
            Some("bestmove") => self.parse_bestmove()?,
            Some("info") => self.parse_info()?,
            Some(_) | None => return Ok(None),
        }))
    }
}

fn is_separator(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn read(s: &str) -> (Option<&str>, &str) {
    let s = s.trim_start_matches(is_separator);
    if s.is_empty() {
        (None, s)
    } else {
        let (head, tail) = s.split_at(memchr2(b' ', b'\t', s.as_bytes()).unwrap_or(s.len()));
        (Some(head), tail)
    }
}

fn read_until<'a, P>(s: &'a str, mut pred: P) -> (Option<&'a str>, &'a str)
where
    P: FnMut(&'a str) -> bool,
{
    let s = s.trim_start_matches(is_separator);
    if s.is_empty() {
        (None, "")
    } else {
        for end in memchr2_iter(b' ', b'\t', s.as_bytes()) {
            let (head, tail) = s.split_at(end);
            if let (Some(next_token), _) = read(tail) {
                if pred(next_token) {
                    return (Some(head.trim_end_matches(is_separator)), tail);
                }
            }
        }
        (Some(s.trim_end_matches(is_separator)), "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        assert_eq!(read(""), (None, ""));
        assert_eq!(read(" abc\t def g"), (Some("abc"), "\t def g"));
        assert_eq!(read("  end"), (Some("end"), ""));
    }

    #[test]
    fn test_read_until() {
        assert_eq!(
            read_until("abc def value foo", |t| t == "value"),
            (Some("abc def"), " value foo")
        );
        assert_eq!(
            read_until("abc def valuefoo", |t| t == "value"),
            (Some("abc def valuefoo"), "")
        );
        assert_eq!(
            read_until("value abc", |t| t == "value"),
            (Some("value abc"), "")
        );
    }
}
