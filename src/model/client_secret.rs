use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Eq, Clone)]
pub struct ClientSecret(String);

impl PartialEq for ClientSecret {
    fn eq(&self, other: &ClientSecret) -> bool {
        // Best effort constant time equality
        self.0.len() == other.0.len()
            && self
                .0
                .bytes()
                .zip(other.0.bytes())
                .fold(0, |acc, (left, right)| acc | (left ^ right))
                == 0
    }
}
