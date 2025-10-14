use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct War{
    pub war_id: String,
    pub war_number: i32,
    pub resistance_start_time: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(unused)]
pub struct Map{
    pub region_id: i32,
    // pub scorched_victory_towns: i32,
    pub map_items: Vec<MapItem>,
    // pub map_items_c: Vec<MapItem>,
    // pub map_items_w: Vec<MapItem>,
    #[allow(unused)]
    pub map_text_items: Vec<MapTextItem>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct MapItem{
    pub team_id: TeamId,
    pub icon_type: i32,
    pub x: f32,
    pub y: f32,
    // pub flags: i32,
    // view_direction: i32,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum TeamId{
    #[serde(rename="NONE")]
    Nobody,
    #[serde(rename="WARDENS")]
    Wardens,
    #[serde(rename="COLONIALS")]
    Colonials,
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[allow(unused)]
pub struct MapTextItem{
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub map_marker_type: String,
}