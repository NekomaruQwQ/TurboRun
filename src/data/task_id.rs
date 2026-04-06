/// Stable ID of 32-bit integers, serialized as 8-char lowercase hex.
///
/// Two IDs collide with probability ~1 / 2^32 per pair — negligible
/// for a desktop app with ~dozens of tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u32);

impl TaskId {
    /// Generates a random ID using [`fastrand`] under the hood.
    pub fn random() -> Self {
        Self(fastrand::u32(..))
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.0)
    }
}

impl std::str::FromStr for TaskId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u32::from_str_radix(s, 16).map(Self)
    }
}

impl serde::Serialize for TaskId {
    fn serialize<S: serde::Serializer>(&self, serializer: S)
     -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TaskId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D)
     -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}