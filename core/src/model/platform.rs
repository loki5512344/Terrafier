use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: String,
    pub display_name: String,
    pub min_height: i16,
    pub max_height: i16,
}

impl Platform {
    pub fn java_1_18() -> Self {
        Self {
            id: "java_anvil_1_18".to_string(),
            display_name: "Minecraft Java 1.18+".to_string(),
            min_height: -64,
            max_height: 320,
        }
    }
}
