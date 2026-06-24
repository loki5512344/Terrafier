//! Minecraft data version constants and version detection.

use crate::model::types::Platform;

pub fn data_version() -> i32 {
    3954
}

pub fn version_from_data_version(dv: i32) -> Option<Platform> {
    match dv {
        2860..=2865 => Some(Platform {
            id: "java_1_18".into(),
            display_name: "Minecraft Java 1.18".into(),
            min_height: -64,
            max_height: 320,
        }),
        2866..=2974 => {
            log::warn!(
                "DataVersion {} is between 1.18 and 1.19, falling back to 1.18",
                dv
            );
            Some(Platform {
                id: "java_1_18".into(),
                display_name: "Minecraft Java 1.18".into(),
                min_height: -64,
                max_height: 320,
            })
        }
        2975..=3117 => Some(Platform {
            id: "java_1_19".into(),
            display_name: "Minecraft Java 1.19".into(),
            min_height: -64,
            max_height: 320,
        }),
        3118..=3336 => {
            log::warn!(
                "DataVersion {} is between 1.19 and 1.20, falling back to 1.19",
                dv
            );
            Some(Platform {
                id: "java_1_19".into(),
                display_name: "Minecraft Java 1.19".into(),
                min_height: -64,
                max_height: 320,
            })
        }
        3337..=3460 => Some(Platform {
            id: "java_1_20".into(),
            display_name: "Minecraft Java 1.20".into(),
            min_height: -64,
            max_height: 320,
        }),
        3461..=3577 => {
            log::warn!(
                "DataVersion {} is between 1.20 and 1.20.5, falling back to 1.20",
                dv
            );
            Some(Platform {
                id: "java_1_20".into(),
                display_name: "Minecraft Java 1.20 (fallback)".into(),
                min_height: -64,
                max_height: 320,
            })
        }
        3578..=3700 => Some(Platform {
            id: "java_1_20_5".into(),
            display_name: "Minecraft Java 1.20.5+".into(),
            min_height: -64,
            max_height: 320,
        }),
        3701..=3818 => {
            log::warn!(
                "DataVersion {} is between 1.20.5 and 1.21, falling back to 1.20.5",
                dv
            );
            Some(Platform {
                id: "java_1_20_5".into(),
                display_name: "Minecraft Java 1.20.5 (fallback)".into(),
                min_height: -64,
                max_height: 320,
            })
        }
        3819..=3953 => Some(Platform {
            id: "java_1_21".into(),
            display_name: "Minecraft Java 1.21".into(),
            min_height: -64,
            max_height: 320,
        }),
        3954..=4100 => Some(Platform {
            id: "java_1_21_2".into(),
            display_name: "Minecraft Java 1.21.2+".into(),
            min_height: -64,
            max_height: 320,
        }),
        _ => None,
    }
}
