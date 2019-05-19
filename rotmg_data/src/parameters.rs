use serde::{Deserialize, Serialize};

/// ROTMG client parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    /// Game build version
    pub version: String,

    /// Port to use when connecting to servers
    pub port: u16,

    /// The game ID for the tutorial
    pub tutorial_gameid: i32,

    /// The game ID for the nexus
    pub nexus_gameid: i32,

    /// The game ID for a random realm
    pub random_gameid: i32,
}
