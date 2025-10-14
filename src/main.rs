use std::io::Write;

use anyhow::Context;

mod warapi_schema;

const WARAPI_REPO_PATH: &'static str = r"H:\trash\trash\warapi";

enum Shard {
    Able,
}
impl Shard {
    fn root_endpoint(&self) -> &'static str {
        match self {
            Shard::Able => "https://war-service-live.foxholeservices.com/api",
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Shard::Able => "able",
        }
    }
}

struct WarapiClient {
    agent: ureq::Agent,
    shard: Shard,
    // this being unused is GOOD.
    // war: warapi_schema::War,
    war_name: String,
}
impl WarapiClient {
    pub fn new(agent: ureq::Agent, shard: Shard) -> Self {
        let response_raw = agent
            .get(format!("{}/worldconquest/war", shard.root_endpoint()))
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();
        let war: warapi_schema::War = serde_json::from_str(&response_raw).unwrap();
        let war_name: String;
        match war.resistance_start_time {
            None => war_name = format!("{}-{}", shard.name(), war.war_number),
            Some(res_time) => {
                let res_start_at = chrono::DateTime::from_timestamp_millis(res_time).unwrap();
                let now = chrono::Utc::now();
                if now.signed_duration_since(res_start_at) < chrono::TimeDelta::hours(12) {
                    panic!(
                        "Too soon since resistance start, we are gonna mess things up. Aborting."
                    )
                }
                war_name = format!("{}-{}-resistance", shard.name(), war.war_number);
            }
        };
        let out_f = &std::path::PathBuf::from(format!("cache/warapi/{}/war.json", war_name));

        if out_f.exists() {
            let old_war: warapi_schema::War =
                serde_json::from_str(&std::fs::read_to_string(out_f).unwrap()).unwrap();
            if war.war_id != old_war.war_id {
                panic!(
                    "The cached /worldconquest/war response at {:?} has the same war number ({}) but differen war id from the one we get from API right now ({} vs {}). This is highly sus!",
                    out_f, war.war_number, war.war_id, old_war.war_id
                );
            }
        } else {
            std::fs::create_dir_all(out_f.parent().unwrap()).unwrap();
            std::fs::File::create(out_f)
                .unwrap()
                .write_all(response_raw.as_bytes())
                .unwrap();
        };
        let out = WarapiClient {
            agent,
            shard,
            // war,
            war_name,
        };
        out
    }

    fn _read_cached<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: impl AsRef<str>,
        file: impl AsRef<str>,
    ) -> T {
        let endpoint = endpoint.as_ref();
        let file = file.as_ref();
        let uri = format!(
            "{}/{}",
            self.shard.root_endpoint(),
            endpoint.trim_start_matches("/")
        );
        let cache_f =
            std::path::PathBuf::from(format!("cache/warapi/{}/{}.json", self.war_name, file,));
        let data = match cache_f.exists() {
            true => std::fs::read_to_string(cache_f).unwrap(),
            false => {
                log::info!("fetching {}", uri);
                let d = self
                    .agent
                    .get(uri)
                    .call()
                    .unwrap()
                    .body_mut()
                    .read_to_string()
                    .unwrap();
                std::fs::create_dir_all(cache_f.parent().unwrap()).unwrap();
                std::fs::File::create(cache_f)
                    .unwrap()
                    .write_all(d.as_bytes())
                    .unwrap();
                d
            }
        };
        let parsed: T = serde_json::from_str(&data).unwrap();
        parsed
    }

    pub fn maps(&self) -> Vec<String> {
        let maps = self._read_cached("/worldconquest/maps", "maps");
        maps
    }
    pub fn map_static(&self, map: &str) -> warapi_schema::Map {
        let map = self._read_cached(
            &format!("/worldconquest/maps/{}/static", map),
            &format!("maps/{}-static.json", map),
        );
        map
    }
    pub fn map_dynamic(&self, map: &str) -> warapi_schema::Map {
        let map = self._read_cached(
            &format!("/worldconquest/maps/{}/dynamic/public", map),
            &format!("maps/{}-dynamic.json", map),
        );
        map
    }
    pub fn get_combined_map(&self, map: &str) -> warapi_schema::Map {
        let map_st = self.map_static(map);
        let map_dy = self.map_dynamic(map);

        let mut out = map_st;
        assert!(out.map_items.is_empty());
        out.map_items = map_dy.map_items;
        out
    }
}

fn get_icon_file_name(icon_id: i32) -> &'static str {
    // source: https://github.com/clapfoot/warapi?tab=readme-ov-file#map-icons
    match icon_id {
        8 => "Forward Base 1", // Forward Base 1,

        11 => "Medical",                // Hospital,
        12 => "Vehicle",                // Vehicle Factory,
        17 => "Manufacturing",          // Refinery,
        18 => "Shipyard",               // Shipyard,
        19 => "TechCenter",             // Tech Center,
        20 => "Salvage",                // Salvage Field,
        21 => "Components",             // Component Field,
        22 => "FuelField",              // Fuel Field,
        23 => "Sulfur",                 // Sulfur Field,
        24 => "WorldMapTent",           // World Map Tent,
        25 => "TravelTent",             // Travel Tent,
        26 => "TrainingArea",           // Training Area,
        27 => "Keep",                   // Special Base (Keep),
        28 => "ObservationTower",       // Observation Tower,
        29 => "Fort",                   // Fort,
        30 => "Troop Ship",             // Troop Ship,
        32 => "SulfurMine",             // Sulfur Mine,
        33 => "StorageFacility",        // Storage Facility,
        34 => "Factory",                // Factory,
        35 => "Safehouse",              // Garrison Station,
        37 => "RocketSite",             // Rocket Site,
        38 => "SalvageMine",            // Salvage Mine,
        39 => "ConstructionYard",       // Construction Yard,
        40 => "ComponentMine",          // Component Mine,
        45 => "RelicBase",              // Relic Base 1,
        51 => "MassProductionFactory",  // Mass Production Factory,
        52 => "Seaport",                // Seaport,
        53 => "CoastalGun",             // Coastal Gun,
        54 => "SoulFactory",            // Soul Factory,
        56 => "TownBaseTier1",          // Town Base 1,
        57 => "TownBaseTier2",          // Town Base 2,
        58 => "TownBaseTier3",          // Town Base 3,
        59 => "StormCannon",            // Storm Cannon,
        60 => "IntelCenter",            // Intel Center,
        61 => "Coal",                   // Coal Field,
        62 => "OilWell",                // Oil Field,
        70 => "RocketTarget",           // Rocket Target,
        71 => "RocketGround Zero",      // Rocket Ground Zero,
        72 => "RocketSite With Rocket", // Rocket Site With Rocket,
        75 => "FacilityMineOilRig",     // Facility Mine Oil Rig,
        83 => "WeatherStation",         // Weather Station,
        84 => "MortarHouse",            // Mortar House,
        other => unimplemented!("unknown icon type {:?}", other),
    }
}

fn get_hex_coords(name: &str) -> (u32, u32) {
    // source: eyes
    // One could also grab these from the game files at War/Content/Blueprints/Data/BMapList.uasset
    // tho the coords there would be different, and placement calc needs adjusted
    match name {
        "BasinSionnachHex" => (0, 0),
        "HowlCountyHex" => (0, 1),
        "ClansheadValleyHex" => (0, 2),
        "MorgensCrossingHex" => (0, 3),
        "GodcroftsHex" => (0, 4),

        "SpeakingWoodsHex" => (1, 0),
        "ReachingTrailHex" => (1, 1),
        "ViperPitHex" => (1, 2),
        "WeatheredExpanseHex" => (1, 3),
        "StlicanShelfHex" => (1, 4),
        "TempestIslandHex" => (1, 5),

        "CallumsCapeHex" => (2, 0),
        "MooringCountyHex" => (2, 1),
        "CallahansPassageHex" => (2, 2),
        "MarbanHollow" => (2, 3),
        "ClahstraHex" => (2, 4),
        "EndlessShoreHex" => (2, 5),
        "TheFingersHex" => (2, 6),

        "NevishLineHex" => (3, 0),
        "StonecradleHex" => (3, 1),
        "LinnMercyHex" => (3, 2),
        "DeadLandsHex" => (3, 3),
        "DrownedValeHex" => (3, 4),
        "AllodsBightHex" => (3, 5),
        "ReaversPassHex" => (3, 6),

        "OarbreakerHex" => (4, 0),
        "FarranacCoastHex" => (4, 1),
        "KingsCageHex" => (4, 2),
        "LochMorHex" => (4, 3),
        "UmbralWildwoodHex" => (4, 4),
        "ShackledChasmHex" => (4, 5),
        "TerminusHex" => (4, 6),

        "FishermansRowHex" => (5, 1),
        "WestgateHex" => (5, 2),
        "SableportHex" => (5, 3),
        "HeartlandsHex" => (5, 4),
        "GreatMarchHex" => (5, 5),
        "AcrithiaHex" => (5, 6),

        "StemaLandingHex" => (6, 2),
        "OriginHex" => (6, 3),
        "AshFieldsHex" => (6, 4),
        "RedRiverHex" => (6, 5),
        "KalokaiHex" => (6, 6),

        other => unimplemented!("unknown coordinates of hex {:?}", other),
    }
}

fn make_map_icon_id(map_item: &warapi_schema::MapItem) -> String {
    let icon_file_name = get_icon_file_name(map_item.icon_type);
    let faction_suffix: &'static str = match map_item.team_id {
        warapi_schema::TeamId::Colonials => "cl",
        warapi_schema::TeamId::Wardens => "wd",
        warapi_schema::TeamId::Nobody => "nt",
    };
    let icon_id = format!("icon-{}-{}", icon_file_name, faction_suffix);
    icon_id
}
fn make_map_icon_base_id(map_item: &warapi_schema::MapItem) -> String {
    let icon_file_name = get_icon_file_name(map_item.icon_type);
    let icon_id = format!("icon-{}-base", icon_file_name);
    icon_id
}

fn draw_all_hexes(warapi_repo_path: &std::path::Path, maps: Vec<(String, warapi_schema::Map)>) {
    let mut canvas = svg::Document::new();
    let mut defs = svg::node::element::Definitions::new();
    let mut defs_terrain = svg::node::element::Group::new().set("id", "terrain-group");
    let mut defs_icons = svg::node::element::Group::new().set("id", "icons-group");
    let mut defs_icons_base = svg::node::element::Group::new().set("id", "icons-base-group");
    let mut def_composed_hexes = svg::node::element::Group::new();

    let cos_30 = (std::f32::consts::PI / 180.0 * 30.0).cos();
    let sin_30 = (std::f32::consts::PI / 180.0 * 30.0).sin();

    let mut known_icon_dims = std::collections::HashMap::new();
    let (mut terrain_width, mut terrain_height) = (0, 0); // uhh...
    // the scaling of just the terrain. Making them smaller in pixels also makes them smaller in filesize
    // which is valuable.
    let terrain_resize_factor = 1.0 / 3.0;
    let icon_scale_factor = 1.0 / 6.0; // scaling of icons. No effect on file size or quality, pure svg
    let global_scale_factor = 1.0; // the scaling of the overall image. No effect on file size, just presentation
    let mut composed_dims = (0, 0); // dimensions of the individual hexes, with everything on them
    let mut eventual_bounds_px = (0, 0); // dimensions of the entire image

    {
        // svg filters for coloring base (black-and-white) icons, either to
        // represent the faction (collie/warden) or just for readability (resources).
        // Colors taken from foxholestats
        let colors = vec![
            ("Collie", [101, 135, 94]),
            ("Warden", [72, 125, 169]),
            ("Salvage", [154, 122, 85]),
            ("Sulfur", [199, 199, 87]),
            ("Coal", [75, 75, 75]),
            ("Oil", [205, 107, 35]),
            ("Components", [200, 200, 200]),
        ];
        for (name, values) in colors {
            let matrix = format!(
                "{} 0 0 0 0\n0 {} 0 0 0\n0 0 {} 0 0\n0 0 0 1 0",
                values[0] as f32 / 255.0,
                values[1] as f32 / 255.0,
                values[2] as f32 / 255.0
            );
            let filter = svg::node::element::Filter::new()
                .set("color-interpolation-filters", "sRGB")
                .add(
                    svg::node::element::FilterEffectColorMatrix::new()
                        .set("in", "SourceGraphic")
                        .set("type", "matrix")
                        .set("values", matrix),
                )
                .set("id", format!("color{}", name));
            defs_icons = defs_icons.add(filter);
        }
    }

    // iterate the maps one by one.
    // Load (and add) the terrain, then load any (missing) icons and add them too.
    for (map_name, map) in maps {
        // debug
        if !["BasinSionnachHex", "HowlCountyHex", "SpeakingWoodsHex"].contains(&map_name.as_str()) {
            // continue;
        }
        log::info!("hex {}", map_name);

        let composed_hex_id = &format!("composed-hex-{}", map_name);
        let tn_id = &format!("terrain-{}", map_name);
        let mut composed = svg::node::element::Group::new().set("id", composed_hex_id.clone());

        // load the terrain
        {
            // fixup clahstra naming discrepancy
            let filename = match map_name.as_str() {
                "ClahstraHex" => format!("Map{}Map.TGA", map_name),
                other => format!("Map{}.TGA", other),
            };

            let map_base_image = &warapi_repo_path.join("Images").join("maps").join(filename);
            let terrain = image::ImageReader::open(map_base_image)
                .with_context(|| anyhow::format_err!("reading {:?}", map_base_image))
                .unwrap()
                .decode()
                .unwrap();
            let terrain = terrain.resize(
                (terrain.width() as f32 * terrain_resize_factor).round() as u32,
                (terrain.height() as f32 * terrain_resize_factor).round() as u32,
                image::imageops::FilterType::Lanczos3,
            );
            if terrain_width != 0 {
                assert_eq!(terrain_width, terrain.width());
                assert_eq!(terrain_height, terrain.height());
            } else {
                terrain_width = terrain.width();
                terrain_height = terrain.height();
            }

            let mut tn_png = std::io::Cursor::new(vec![]);
            terrain
                .write_to(&mut tn_png, image::ImageFormat::Png)
                .unwrap();

            let tn = svg::node::element::Image::new()
                .set("id", tn_id.clone())
                .set("width", terrain.width())
                .set("height", terrain.height())
                .set(
                    "href",
                    format!("data:image/png;base64,{}", base64::encode(tn_png.get_ref())),
                );
            defs_terrain = defs_terrain.add(tn);
            let u = svg::node::element::Use::new().set("href", format!("#{}", tn_id));
            composed = composed.add(u);
        }

        // Now do the icons
        // Sort them top-to-bottom for prettier layering. Works well for mines
        // We could probably also yeet safehouses and other boring things to the background, but that's tricky
        let mut map_items = map.map_items;
        map_items.sort_by_key(|it| ordered_float::OrderedFloat(it.y));

        for mi in &map_items {
            let icon_id_for_map = &make_map_icon_id(mi);
            if !known_icon_dims.contains_key(icon_id_for_map) {
                log::info!("adding {} and variants", icon_id_for_map);
                let base_icon_id = &make_map_icon_base_id(mi);

                // if we don't have the pixels - get the pixels
                if !known_icon_dims.contains_key(base_icon_id) {
                    let icon_path = &warapi_repo_path
                        .join("Images")
                        .join("MapIcons")
                        .join(format!("MapIcon{}.TGA", get_icon_file_name(mi.icon_type)));
                    let icon = image::ImageReader::open(icon_path)
                        .with_context(|| {
                            format!(
                                "trying to open {:?} - icon type {}",
                                icon_path, mi.icon_type
                            )
                        })
                        .unwrap()
                        .decode()
                        .unwrap();
                    let (icon_width, icon_height) = (
                        (icon.width() as f32 * icon_scale_factor).round(),
                        (icon.height() as f32 * icon_scale_factor).round(),
                    );

                    let mut base_icon_png = std::io::Cursor::new(vec![]);
                    icon.write_to(&mut base_icon_png, image::ImageFormat::Png)
                        .unwrap();
                    let icon_elem = svg::node::element::Image::new()
                        .set("id", base_icon_id.clone())
                        .set("width", icon_width)
                        .set("height", icon_height)
                        .set(
                            "href",
                            format!(
                                "data:image/png;base64,{}",
                                base64::encode(base_icon_png.get_ref())
                            ),
                        );
                    defs_icons_base = defs_icons_base.add(icon_elem.clone());
                    known_icon_dims.insert(base_icon_id.clone(), (icon_width, icon_height));
                }

                // we do have the pixels, but might not have the correct colors yet.
                let base_icon_dims = known_icon_dims.get(base_icon_id).unwrap().clone();
                let factions = match mi.team_id {
                    // not making warden-colored fields - that's silly
                    warapi_schema::TeamId::Nobody => vec![warapi_schema::TeamId::Nobody],
                    // but if it is already warden/collie-owned then it makes sense that
                    // the other team can have it as well, so make all three of them
                    _ => vec![
                        warapi_schema::TeamId::Nobody,
                        warapi_schema::TeamId::Wardens,
                        warapi_schema::TeamId::Colonials,
                    ],
                };
                for faction in factions {
                    let new_mi = &warapi_schema::MapItem {
                        team_id: faction,
                        ..mi.clone()
                    };
                    let icon_id_here = make_map_icon_id(new_mi);
                    let mut icon_here = svg::node::element::Use::new()
                        .set("id", icon_id_here.clone())
                        .set("href", format!("#{}", base_icon_id));

                    use warapi_schema::TeamId;
                    if let Some(filter) = match faction {
                        TeamId::Colonials => Some("colorCollie"),
                        TeamId::Wardens => Some("colorWarden"),
                        TeamId::Nobody => match get_icon_file_name(new_mi.icon_type) {
                            "Salvage" | "SalvageMine" => Some("colorSalvage"),
                            "Sulfur" | "SulfurMine" => Some("colorSulfur"),
                            "Coal" => Some("colorCoal"),
                            "OilWell" => Some("colorOil"),
                            "Components" | "ComponentMine" => Some("colorComponents"),
                            _ => None,
                        },
                    } {
                        icon_here = icon_here.set("filter", format!("url(#{})", filter));
                    }

                    defs_icons = defs_icons.add(icon_here);
                    known_icon_dims.insert(icon_id_here.clone(), base_icon_dims.clone());
                }
            }

            let (icon_width, icon_height) = known_icon_dims.get(icon_id_for_map).unwrap();

            let tlx = (terrain_width as f32 * mi.x - icon_width / 2.0) as u32;
            let tly = (terrain_height as f32 * mi.y - icon_height / 2.0) as u32;
            let u = svg::node::element::Use::new()
                .set("href", format!("#{}", icon_id_for_map))
                .set("x", tlx)
                .set("y", tly);
            composed = composed.add(u);

            composed = composed.set("transform", format!("scale({})", global_scale_factor));
            composed_dims = (
                // the +2/+1 are for pretty white "borders" between hexes
                // (explicitly drawing borders on top did not work out well at all -
                //  drawing pixels looks bad, and drawing svg segments/polygons is VERY heavy
                //  for the browser, somehow)
                (terrain_width as f32 * global_scale_factor) as i32 + 2,
                (terrain_height as f32 * global_scale_factor) as i32 + 1,
            );
        }
        def_composed_hexes = def_composed_hexes.add(composed);

        // now position the hex on the global grid
        {
            let hex_diameter_short = composed_dims.1;
            let hex_coords = get_hex_coords(&map_name);

            let global_offset_px = ((cos_30 * 4.0) * hex_diameter_short as f32, 0.0);
            let offset_hexes = (
                hex_coords.0 as f32 * -1.0 * cos_30 + hex_coords.1 as f32 * cos_30,
                hex_coords.0 as f32 * sin_30 + hex_coords.1 as f32 * sin_30,
            );

            let offset = (
                global_offset_px.0 + offset_hexes.0 * hex_diameter_short as f32, // heigh
                global_offset_px.1 + offset_hexes.1 * hex_diameter_short as f32,
            );

            let u = svg::node::element::Use::new()
                .set("href", format!("#{}", composed_hex_id))
                .set("x", offset.0 as i32)
                .set("y", offset.1 as i32);
            canvas = canvas.add(u);
            eventual_bounds_px = (
                eventual_bounds_px.0.max(offset.0 as i32 + composed_dims.0),
                eventual_bounds_px.1.max(offset.1 as i32 + composed_dims.1),
            );
        }
    }

    // combine the svg parts together, and write the file out
    defs = defs.add(defs_icons);
    defs = defs.add(def_composed_hexes);
    defs = defs.add(defs_icons_base);
    defs = defs.add(defs_terrain);

    canvas = canvas.add(defs);
    canvas = canvas
        .set("width", eventual_bounds_px.0)
        .set("height", eventual_bounds_px.1);

    let out_f = &std::path::PathBuf::from("tmp/out.svg");
    std::fs::create_dir_all(out_f.parent().unwrap()).unwrap();
    svg::save(out_f, &canvas).unwrap();
    log::info!("Written to {}", out_f.display());
}

fn do_stuff(cfg: &Config) {
    let agent = ureq::Agent::new_with_defaults();
    let shard = Shard::Able;
    let client = WarapiClient::new(agent, shard);

    let maps = client
        .maps()
        .iter()
        .map(|mapname| (mapname.clone(), client.get_combined_map(mapname)))
        .collect::<Vec<_>>();
    // let warapi_repo_path = std::path::Path::new(WARAPI_REPO_PATH);
    draw_all_hexes(&cfg.warapi_repo_path, maps);
}

#[derive(serde::Deserialize)]
struct Config {
    warapi_repo_path: std::path::PathBuf,
}
fn read_config() -> Config {
    use std::path::Path;
    let config_path = Path::new("config.json5");
    let config_example_path = Path::new("config.example.json5");
    if !config_path.exists() {
        log::warn!(
            "{} does not exist! Copying over from {} - you might want to tweak it tho",
            config_path.display(),
            config_example_path.display()
        );
        std::fs::copy(config_example_path, config_path).unwrap();
    }
    let cfg: Config = json5::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();

    // you know what, let's verify the config while at it...
    {
        if !cfg.warapi_repo_path.exists() {
            log::error!(
                "Your config at {} specifies {} as the path for warapi repo, but that doesn't exist. Download it from https://github.com/clapfoot/warapi or tweak the path, idk.",
                config_path.display(),
                cfg.warapi_repo_path.display(),
            );
            std::process::exit(1);
        }
    }
    cfg
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cfg = &read_config();
    do_stuff(cfg)
}
