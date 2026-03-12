use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An item returned by /maps/detailedList.
#[derive(Debug, Clone, Deserialize)]
pub struct Map {
    pub id: i32,
    pub name: String,
    pub zone_counts: HashMap<String, i32>,
    pub authors: Vec<Author>,
    pub tier_info: TierInfo,
    pub rating_info: RatingInfo,
    pub videos: Videos,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub id: i32,
    pub map_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TierInfo {
    #[serde(rename = "3")]
    pub soldier: i8,
    #[serde(rename = "4")]
    pub demoman: i8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingInfo {
    #[serde(rename = "3")]
    pub soldier: i8,
    #[serde(rename = "4")]
    pub demoman: i8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Videos {
    pub soldier: Option<String>,
    pub demoman: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapRecordsList {
    pub zone_info: ZoneInfo,
    pub tier_info: TierInfo,
    pub rating_info: RatingInfo,
    pub completion_info: CompletionInfo,
    #[serde(rename = "results")]
    pub records: Records,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZoneInfo {
    pub id: i32,
    pub map_id: i32,
    pub zoneindex: i32,
    pub custom_name: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionInfo {
    pub soldier: i32,
    pub demoman: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Records {
    pub soldier: Vec<Record>,
    pub demoman: Vec<Record>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub id: i32,
    pub zone_id: i32,
    pub duration: f64,
    pub class: i8,
    pub date: f64,
    pub demo_info: DemoInfo,
    pub user_id: i32,
    pub name: String,
    pub steamid: String,
    pub rank: i32,
    pub placement: i32,
    pub player_info: PlayerInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DemoInfo {
    pub id: i32,
    pub start_tick: i32,
    pub end_tick: i32,
    pub url: Option<String>,
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerInfo {
    pub id: i32,
    pub steamid: String,
    pub name: String,
}

#[cfg(test)]
mod tests {

    mod maps {
        use crate::models::Map;

        const DATA: &str = include_str!("../test_data/maps.json");

        #[test]
        fn deserializes_full_response() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            assert_eq!(maps.len(), 775);
        }

        #[test]
        fn basic_fields() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 750).unwrap();
            assert_eq!(map.name, "jump_a_b3");
        }

        #[test]
        fn tier_info_renames() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 750).unwrap();
            assert_eq!(map.tier_info.soldier, 6);
            assert_eq!(map.tier_info.demoman, 3);
        }

        #[test]
        fn rating_info_renames() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 481).unwrap();
            assert_eq!(map.rating_info.soldier, 1);
            assert_eq!(map.rating_info.demoman, 2);
        }

        #[test]
        fn zone_counts_variable_keys() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();

            // jump_a_b3 has bonus zones
            let map = maps.iter().find(|m| m.id == 750).unwrap();
            assert_eq!(map.zone_counts.get("checkpoint"), Some(&4));
            assert_eq!(map.zone_counts.get("bonus"), Some(&4));
            assert_eq!(map.zone_counts.get("bonus_end"), Some(&4));

            // jump_abandon has no bonus zones
            let map = maps.iter().find(|m| m.id == 481).unwrap();
            assert_eq!(map.zone_counts.get("bonus"), None);
            assert_eq!(map.zone_counts.get("checkpoint"), Some(&6));

            // jump_ablation has misc and trick zones
            let map = maps.iter().find(|m| m.id == 376).unwrap();
            assert_eq!(map.zone_counts.get("trick"), Some(&2));
            assert_eq!(map.zone_counts.get("misc"), Some(&4));

            // jump_above has special zones
            let map = maps.iter().find(|m| m.id == 207).unwrap();
            assert_eq!(map.zone_counts.get("special"), Some(&2));
        }

        #[test]
        fn single_author() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 750).unwrap();
            assert_eq!(map.authors.len(), 1);
            assert_eq!(map.authors[0].name, "Waldo");
            assert_eq!(map.authors[0].id, 1619776);
            assert_eq!(map.authors[0].map_id, 750);
        }

        #[test]
        fn multiple_authors() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 759).unwrap();
            assert_eq!(map.authors.len(), 3);
            let names: Vec<&str> = map.authors.iter().map(|a| a.name.as_str()).collect();
            assert_eq!(names, vec!["riotbz", "Adam-g1", "Zike1017"]);
        }

        #[test]
        fn videos_both_present() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 262).unwrap(); // jump_aigis
            assert_eq!(map.videos.soldier.as_deref(), Some("ZGzteRbGXTI"));
            assert_eq!(map.videos.demoman.as_deref(), Some("ZGzteRbGXTI"));
        }

        #[test]
        fn videos_both_null() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 811).unwrap(); // jump_academy3
            assert!(map.videos.soldier.is_none());
            assert!(map.videos.demoman.is_none());
        }

        #[test]
        fn videos_only_demoman() {
            let maps: Vec<Map> = serde_json::from_str(DATA).unwrap();
            let map = maps.iter().find(|m| m.id == 752).unwrap(); // jump_achlys
            assert!(map.videos.soldier.is_none());
            assert_eq!(map.videos.demoman.as_deref(), Some("_rXKkSELjHo"));
        }
    }

    mod records {
        use crate::models::MapRecordsList;

        const DATA: &str = include_str!("../test_data/map_records_136.json");
        const DATA2: &str = include_str!("../test_data/map_records_4.json");
        const DATA3: &str = include_str!("../test_data/map_records_412_full.json");

        #[test]
        fn deserializes_full_response() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            assert_eq!(resp.records.soldier.len(), 50);
            assert_eq!(resp.records.demoman.len(), 50);

            let resp: MapRecordsList = serde_json::from_str(DATA2).unwrap();
            assert_eq!(resp.records.soldier.len(), 50);
            assert_eq!(resp.records.demoman.len(), 50);

            let _: MapRecordsList = serde_json::from_str(DATA3).unwrap();
        }

        #[test]
        fn zone_info() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            assert_eq!(resp.zone_info.id, 1681);
            assert_eq!(resp.zone_info.map_id, 136);
            assert_eq!(resp.zone_info.zoneindex, 1);
            assert!(resp.zone_info.custom_name.is_none());
            assert_eq!(resp.zone_info.kind, "map");
        }

        #[test]
        fn tier_info_renames() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            assert_eq!(resp.tier_info.soldier, 5);
            assert_eq!(resp.tier_info.demoman, 3);
        }

        #[test]
        fn rating_info_renames() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            assert_eq!(resp.rating_info.soldier, 1);
            assert_eq!(resp.rating_info.demoman, 4);
        }

        #[test]
        fn completion_info() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            assert_eq!(resp.completion_info.soldier, 1331);
            assert_eq!(resp.completion_info.demoman, 720);
        }

        #[test]
        fn record_basic_fields() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            let rec = &resp.records.soldier[0];
            assert_eq!(rec.id, 7466797);
            assert_eq!(rec.zone_id, 1681);
            assert_eq!(rec.class, 3);
            assert_eq!(rec.user_id, 39902);
            assert_eq!(rec.rank, 1);
            assert_eq!(rec.placement, 1);
            assert_eq!(rec.name, "On Little Cat Feet");
            assert_eq!(rec.steamid, "STEAM_0:0:43167835");
        }

        #[test]
        fn record_ordering_by_rank() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            let ranks: Vec<i32> = resp.records.soldier.iter().map(|r| r.rank).collect();
            let mut sorted = ranks.clone();
            sorted.sort();
            assert_eq!(ranks, sorted);
        }

        #[test]
        fn record_durations_ascending() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            let durations: Vec<f64> = resp.records.soldier.iter().map(|r| r.duration).collect();
            for w in durations.windows(2) {
                assert!(
                    w[0] <= w[1],
                    "expected ascending durations: {} <= {}",
                    w[0],
                    w[1]
                );
            }
        }

        #[test]
        fn demo_info() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            let demo = &resp.records.soldier[0].demo_info;
            assert_eq!(demo.id, 3043671);
            assert_eq!(demo.start_tick, 398767);
            assert_eq!(demo.end_tick, 414029);

            if let Some(demo_url) = &demo.url {
                assert!(demo_url.starts_with("https://"));
                assert!(demo_url.ends_with(".zip"));
            }
        }

        #[test]
        fn server_info() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            let server = &resp.records.soldier[0].demo_info.server_info;
            assert_eq!(server.id, 66);
            assert_eq!(
                server.name,
                Some("jump.tf (Frankfurt) Rank 100 Only".to_string())
            );
        }

        #[test]
        fn player_info_matches_top_level() {
            let resp: MapRecordsList = serde_json::from_str(DATA).unwrap();
            for rec in &resp.records.soldier {
                assert_eq!(rec.player_info.id, rec.user_id);
                assert_eq!(rec.player_info.steamid, rec.steamid);
                assert_eq!(rec.player_info.name, rec.name);
            }
        }
    }
}
