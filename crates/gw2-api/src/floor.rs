//! Per-map floor data (`/v2/continents/<c>/floors/<f>/regions/<r>/maps/<m>`):
//! points of interest, waypoints, renown hearts, hero challenges and sectors, in
//! **continent coordinates**. The [`MapFrame`] converts them to the map's own
//! coordinate space.
//!
//! How API map coordinates map to the client's in-world coordinates is
//! **unknown** until REA or a capture shows it. Map units are commonly said to be
//! inches, but that is unverified.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Map, Rect};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapFloor {
    pub id: u32,
    pub name: String,
    pub min_level: u32,
    pub max_level: u32,
    pub default_floor: i32,
    pub map_rect: Option<Rect>,
    pub continent_rect: Option<Rect>,
    pub points_of_interest: BTreeMap<String, Poi>,
    pub tasks: BTreeMap<String, Task>,
    pub skill_challenges: Vec<SkillChallenge>,
    pub sectors: BTreeMap<String, Sector>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Poi {
    pub id: u32,
    pub name: Option<String>,
    /// `landmark`, `waypoint`, `vista`, `unlock`.
    #[serde(rename = "type")]
    pub kind: String,
    pub floor: i32,
    pub coord: [f64; 2],
}

/// Renown heart.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Task {
    pub id: u32,
    pub objective: String,
    pub level: u32,
    pub coord: [f64; 2],
    pub bounds: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillChallenge {
    pub id: Option<String>,
    pub coord: [f64; 2],
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sector {
    pub id: u32,
    pub name: Option<String>,
    pub level: u32,
    pub coord: [f64; 2],
    pub bounds: Vec<[f64; 2]>,
}

/// Linear mapping between continent coordinates (y grows downward) and map
/// coordinates (y grows upward), from a map's two rectangles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapFrame {
    pub map_rect: Rect,
    pub continent_rect: Rect,
}

impl MapFrame {
    pub fn of(map: &Map) -> Option<Self> {
        Some(Self { map_rect: map.map_rect?, continent_rect: map.continent_rect? })
    }

    pub fn of_floor(m: &MapFloor) -> Option<Self> {
        Some(Self { map_rect: m.map_rect?, continent_rect: m.continent_rect? })
    }

    pub fn continent_to_map(&self, c: [f64; 2]) -> [f64; 2] {
        let [[mx0, my0], [mx1, my1]] = self.map_rect;
        let [[cx0, cy0], [cx1, cy1]] = self.continent_rect;
        let x = mx0 + (c[0] - cx0) / (cx1 - cx0) * (mx1 - mx0);
        let y = my1 - (c[1] - cy0) / (cy1 - cy0) * (my1 - my0);
        [x, y]
    }

    pub fn map_to_continent(&self, m: [f64; 2]) -> [f64; 2] {
        let [[mx0, my0], [mx1, my1]] = self.map_rect;
        let [[cx0, cy0], [cx1, cy1]] = self.continent_rect;
        let x = cx0 + (m[0] - mx0) / (mx1 - mx0) * (cx1 - cx0);
        let y = cy0 + (my1 - m[1]) / (my1 - my0) * (cy1 - cy0);
        [x, y]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queensdale() -> MapFrame {
        MapFrame {
            map_rect: [[-43008.0, -27648.0], [43008.0, 30720.0]],
            continent_rect: [[42624.0, 28032.0], [46208.0, 30464.0]],
        }
    }

    #[test]
    fn corners_map_to_corners() {
        let f = queensdale();
        assert_eq!(f.continent_to_map([42624.0, 28032.0]), [-43008.0, 30720.0]);
        assert_eq!(f.continent_to_map([46208.0, 30464.0]), [43008.0, -27648.0]);
    }

    #[test]
    fn round_trip() {
        let f = queensdale();
        let c = [44063.9, 29886.6]; // Claypool renown heart (task 211)
        let m = f.continent_to_map(c);
        let back = f.map_to_continent(m);
        assert!((back[0] - c[0]).abs() < 1e-6 && (back[1] - c[1]).abs() < 1e-6);
        // Continent x 44063.9 < centre 44416, y 29886.6 > centre 29248: south-west.
        assert!(m[0] < 0.0 && m[1] < 0.0, "Claypool is in the south-west quadrant: {m:?}");
    }

    #[test]
    fn parses_floor_sample() {
        let j = r#"{"id":15,"name":"Queensdale","map_rect":[[-43008,-27648],[43008,30720]],
            "continent_rect":[[42624,28032],[46208,30464]],
            "points_of_interest":{"127":{"name":"Eda's Orchard","type":"landmark","floor":1,"coord":[42830.7,28406.7],"id":127,"chat_link":"x"}},
            "tasks":{"211":{"objective":"Help the Seraph protect Claypool from centaurs.","level":9,"coord":[44063.9,29886.6],"bounds":[[1,2],[3,4]],"id":211}},
            "skill_challenges":[{"coord":[1.0,2.0],"id":"0-5"}],
            "sectors":{}}"#;
        let m: MapFloor = serde_json::from_str(j).unwrap();
        assert_eq!(m.points_of_interest["127"].kind, "landmark");
        assert_eq!(m.tasks["211"].level, 9);
        assert!(MapFrame::of_floor(&m).is_some());
    }
}
