use super::zoo_message::ZooVersion;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for ZooVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let version = match *self {
            ZooVersion::V1_0 => "V1_0",
            ZooVersion::Unsupported => "Unsupported",
        };
        serializer.serialize_str(version)
    }
}

impl<'de> Deserialize<'de> for ZooVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = String::deserialize(deserializer)?;
        Ok(match version.as_str() {
            "V1_0" => ZooVersion::V1_0,
            _ => ZooVersion::Unsupported,
        })
    }
}
