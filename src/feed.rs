use crate::location::Coords;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use std::time::Duration;

pub const DEFAULT_RADIUS_MI: u32 = 10;
/// The feeds cap a query at 250 nautical miles.
pub const MAX_RADIUS_MI: u32 = 250;
pub const MENU_AIRCRAFT_LIMIT: usize = 15;

const MILES_PER_NM: f64 = 1.150_78;
const EARTH_RADIUS_MI: f64 = 3958.8;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Aircraft {
    pub hex: String,
    #[serde(rename = "flight")]
    pub callsign: Option<String>,
    #[serde(rename = "r")]
    pub registration: Option<String>,
    #[serde(rename = "t")]
    pub type_code: Option<String>,
    #[serde(rename = "desc")]
    pub description: Option<String>,
    #[serde(rename = "ownOp")]
    pub operator: Option<String>,
    #[serde(alias = "destination", alias = "dest", alias = "to")]
    pub destination: Option<String>,
    #[serde(alias = "origin", alias = "from")]
    pub origin: Option<String>,
    #[serde(skip)]
    pub origin_name: Option<String>,
    #[serde(skip)]
    pub destination_name: Option<String>,
    #[serde(skip)]
    pub photo_url: Option<String>,
    #[serde(skip)]
    pub photo_page: Option<String>,
    #[serde(skip)]
    pub photo_photographer: Option<String>,
    pub alt_baro: Option<Altitude>,
    #[serde(rename = "gs")]
    pub ground_speed: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Altitude {
    Feet(f64),
    /// The feeds send the string "ground" instead of a number.
    Ground(#[allow(dead_code)] String),
}

/// Barometric altitude bounds in feet. A missing bound is open.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AltitudeRange {
    pub min_ft: Option<u32>,
    pub max_ft: Option<u32>,
}

/// The highest altitude the settings accept.
pub const MAX_ALTITUDE_FT: u32 = 60_000;

impl AltitudeRange {
    /// Puts the bounds in order.
    pub fn sorted(self) -> Self {
        match (self.min_ft, self.max_ft) {
            (Some(min), Some(max)) if min > max => Self {
                min_ft: Some(max),
                max_ft: Some(min),
            },
            _ => self,
        }
    }

    /// Aircraft on the ground count as 0 ft. Ones that don't report altitude always pass.
    pub fn contains(&self, altitude: Option<&Altitude>) -> bool {
        let ft = match altitude {
            Some(Altitude::Feet(ft)) => *ft,
            Some(Altitude::Ground(_)) => 0.0,
            None => return true,
        };
        self.min_ft.is_none_or(|min| ft >= min as f64)
            && self.max_ft.is_none_or(|max| ft <= max as f64)
    }
}

impl Aircraft {
    pub fn callsign(&self) -> Option<&str> {
        self.callsign
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    pub fn name(&self) -> String {
        self.callsign()
            .or(self.registration.as_deref())
            .unwrap_or(&self.hex)
            .to_uppercase()
    }

    pub fn model(&self) -> Option<&str> {
        self.description.as_deref().or(self.type_code.as_deref())
    }

    pub fn flightradar_url(&self) -> String {
        match (self.callsign(), self.registration.as_deref()) {
            (Some(callsign), _) => format!("https://www.flightradar24.com/{callsign}"),
            (None, Some(reg)) => format!("https://www.flightradar24.com/data/aircraft/{reg}"),
            (None, None) => format!("https://www.flightradar24.com/{}", self.hex),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Contact {
    pub aircraft: Aircraft,
    pub distance_mi: f64,
    pub direction: &'static str,
}

#[derive(Deserialize)]
struct Response {
    #[serde(alias = "aircraft")]
    ac: Vec<Aircraft>,
}

struct Source {
    name: &'static str,
    url: fn(f64, f64, u32) -> String,
}

const SOURCES: [Source; 2] = [
    Source {
        name: "adsb.fi",
        url: |lat, lon, nm| {
            format!("https://opendata.adsb.fi/api/v2/lat/{lat}/lon/{lon}/dist/{nm}")
        },
    },
    Source {
        name: "adsb.lol",
        url: |lat, lon, nm| format!("https://api.adsb.lol/v2/point/{lat}/{lon}/{nm}"),
    },
];

pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(concat!(
            "Overhead/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/y-u-s-u-f/overhead)"
        ))
        .build()
        .into()
}

/// Aircraft within the radius, closest first. Tries each source in order.
///
/// With `approximate`, the feeds only see home rounded to two decimals (about 1 km).
/// The query grows by a mile to cover the difference, and distances still use exact home.
pub fn fetch(
    agent: &ureq::Agent,
    home: Coords,
    radius_mi: u32,
    approximate: bool,
) -> Result<(&'static str, Vec<Contact>), String> {
    let mut radius_nm = (radius_mi as f64 / MILES_PER_NM).ceil() as u32;
    let (mut lat, mut lon) = home;
    if approximate {
        let round = |x: f64| (x * 100.0).round() / 100.0;
        (lat, lon) = (round(lat), round(lon));
        radius_nm += 1;
    }
    let mut errors = Vec::new();

    for source in &SOURCES {
        let url = (source.url)(lat, lon, radius_nm);
        match agent
            .get(&url)
            .call()
            .and_then(|mut r| r.body_mut().read_json::<Response>())
        {
            Ok(response) => {
                let mut contacts = nearby_aircraft(response.ac, home, radius_mi);
                enrich_routes(agent, &mut contacts);
                enrich_photos(agent, &mut contacts);
                return Ok((source.name, contacts));
            }
            Err(err) => errors.push(format!("{}: {err}", source.name)),
        }
    }
    Err(errors.join("; "))
}

fn enrich_routes(agent: &ureq::Agent, contacts: &mut [Contact]) {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    static ROUTES: OnceLock<Mutex<HashMap<String, (Instant, Option<FlightRoute>)>>> = OnceLock::new();
    let routes = ROUTES.get_or_init(|| Mutex::new(HashMap::new()));
    let missing: Vec<_> = contacts
        .iter()
        .take(MENU_AIRCRAFT_LIMIT)
        .filter_map(|contact| {
            let aircraft = &contact.aircraft;
            let callsign = aircraft.callsign()?.trim().to_uppercase();
            let has_route_names = aircraft.origin_name.as_deref().is_some_and(|s| !s.trim().is_empty())
                && aircraft.destination_name.as_deref().is_some_and(|s| !s.trim().is_empty());
            let stale = routes
                .lock()
                .ok()?
                .get(&callsign)
                .is_none_or(|(checked, _)| checked.elapsed() > Duration::from_secs(60));
            (!has_route_names && stale).then_some((callsign, aircraft.lat?, aircraft.lon?))
        })
        .collect();
    if !missing.is_empty() {
        let planes: Vec<_> = missing
            .iter()
            .map(|(callsign, lat, lng)| serde_json::json!({"callsign": callsign, "lat": lat, "lng": lng}))
            .collect();
        let lookup = agent
            .post("https://api.adsb.lol/api/0/routeset")
            .header("Accept", "application/json")
            .send_json(serde_json::json!({"planes": planes}))
            .and_then(|mut response| response.body_mut().read_json::<serde_json::Value>());
        let mut found = HashMap::new();
        if let Ok(value) = lookup {
            if let Some(items) = value.as_array() {
                for item in items {
                    let Some(callsign) = item.get("callsign").and_then(|v| v.as_str()) else { continue };
                    // Routeset uses the aircraft's current position as well as its callsign.
                    // Ignore routes it explicitly marks implausible: callsigns can be reused
                    // and a plausible-looking callsign-only route may belong to another flight.
                    let airports = item.get("_airports").and_then(|v| v.as_array());
                    let route = airports.and_then(|airports| {
                        if airports.len() < 2 { return None; }
                        let route = route_from_airports(&airports[0], &airports[1])?;
                        let plausible = item.get("plausible").and_then(|v| v.as_bool()) == Some(true);
                        let near_path = missing.iter().find(|(candidate, _, _)| {
                            candidate.eq_ignore_ascii_case(callsign)
                        }).is_some_and(|(_, lat, lon)| route_is_near_position(&airports[0], &airports[1], *lat, *lon));
                        (plausible || near_path).then_some(route)
                    });
                    found.insert(callsign.trim().to_uppercase(), route);
                }
            }
        }
        if let Ok(mut cache) = routes.lock() {
            for (callsign, _, _) in &missing {
                cache.insert(
                    callsign.clone(),
                    (Instant::now(), found.get(callsign).cloned().flatten()),
                );
            }
        }
    }
    if let Ok(cache) = routes.lock() {
        for contact in contacts {
            let Some(callsign) = contact.aircraft.callsign().map(|s| s.trim().to_uppercase()) else { continue };
            let Some((_, Some(route))) = cache.get(&callsign) else { continue };
            if contact.aircraft.origin.as_deref().is_none_or(|s| s.trim().is_empty()) {
                contact.aircraft.origin = Some(route.origin_code.clone());
            }
            if contact.aircraft.destination.as_deref().is_none_or(|s| s.trim().is_empty()) {
                contact.aircraft.destination = Some(route.destination_code.clone());
            }
            contact.aircraft.origin_name = route.origin_name.clone();
            contact.aircraft.destination_name = route.destination_name.clone();
        }
    }
}

#[derive(Clone, Debug)]
struct FlightRoute {
    origin_code: String,
    destination_code: String,
    origin_name: Option<String>,
    destination_name: Option<String>,
}

fn route_from_airports(origin: &serde_json::Value, destination: &serde_json::Value) -> Option<FlightRoute> {
    Some(FlightRoute {
        origin_code: airport_name(origin)?,
        destination_code: airport_name(destination)?,
        origin_name: airport_full_name(origin),
        destination_name: airport_full_name(destination),
    })
}

fn airport_name(airport: &serde_json::Value) -> Option<String> {
    airport.get("iata").and_then(|v| v.as_str())
        .or_else(|| airport.get("iata_code").and_then(|v| v.as_str()))
        .or_else(|| airport.get("icao").and_then(|v| v.as_str()))
        .or_else(|| airport.get("icao_code").and_then(|v| v.as_str()))
        .map(str::to_owned)
}

fn airport_full_name(airport: &serde_json::Value) -> Option<String> {
    airport.get("name").and_then(|v| v.as_str())
        .or_else(|| airport.get("location").and_then(|v| v.as_str()))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

/// Accept a routeset result it labels uncertain only when the live aircraft position
/// is within 250 miles of the airport-to-airport segment.
fn route_is_near_position(origin: &serde_json::Value, destination: &serde_json::Value, lat: f64, lon: f64) -> bool {
    let coord = |airport: &serde_json::Value, key: &str| airport.get(key).and_then(|v| v.as_f64());
    let (Some(a_lat), Some(a_lon), Some(b_lat), Some(b_lon)) = (
        coord(origin, "lat"), coord(origin, "lon"), coord(destination, "lat"), coord(destination, "lon"),
    ) else { return false; };

    // Local equirectangular projection is accurate enough for this broad rejection radius.
    let mean_lat = ((a_lat + b_lat + lat) / 3.0).to_radians();
    let project = |p_lat: f64, p_lon: f64| (p_lon.to_radians() * mean_lat.cos() * EARTH_RADIUS_MI, p_lat.to_radians() * EARTH_RADIUS_MI);
    let (ax, ay) = project(a_lat, a_lon);
    let (bx, by) = project(b_lat, b_lon);
    let (px, py) = project(lat, lon);
    let (dx, dy) = (bx - ax, by - ay);
    let length_sq = dx * dx + dy * dy;
    if length_sq == 0.0 { return false; }
    let t = (((px - ax) * dx + (py - ay) * dy) / length_sq).clamp(0.0, 1.0);
    ((px - (ax + t * dx)).powi(2) + (py - (ay + t * dy)).powi(2)).sqrt() <= 250.0
}

#[derive(Clone, Debug)]
struct RemotePhoto {
    image: String,
    page: String,
    photographer: Option<String>,
}

#[derive(Deserialize)]
struct JetApiResponse {
    #[serde(rename = "Images", default)]
    images: Vec<JetApiPhoto>,
}

#[derive(Deserialize)]
struct JetApiPhoto {
    #[serde(rename = "Image")]
    image: String,
    #[serde(rename = "Link")]
    link: String,
    #[serde(rename = "Photographer")]
    photographer: Option<String>,
}

fn enrich_photos(agent: &ureq::Agent, contacts: &mut [Contact]) {
    static PHOTOS: OnceLock<Mutex<HashMap<String, (Instant, Option<RemotePhoto>)>>> = OnceLock::new();
    let cache = PHOTOS.get_or_init(|| Mutex::new(HashMap::new()));
    let missing: Vec<String> = contacts
        .iter()
        .take(MENU_AIRCRAFT_LIMIT)
        .filter_map(|contact| contact.aircraft.registration.as_deref())
        .map(|registration| registration.trim().to_uppercase())
        .filter(|registration| !registration.is_empty())
        .filter(|registration| {
            cache.lock().ok().is_some_and(|entries| {
                entries.get(registration).is_none_or(|(updated, _)| updated.elapsed() > Duration::from_secs(21_600))
            })
        })
        .collect();

    if !missing.is_empty() {
        let results = std::thread::scope(|scope| {
            let tasks: Vec<_> = missing
                .iter()
                .map(|registration| {
                    let client = agent.clone();
                    scope.spawn(move || (registration.clone(), fetch_jetphoto(&client, registration)))
                })
                .collect();
            tasks.into_iter().filter_map(|task| task.join().ok()).collect::<Vec<_>>()
        });
        if let Ok(mut entries) = cache.lock() {
            for (registration, photo) in results {
                entries.insert(registration, (Instant::now(), photo));
            }
        }
    }

    if let Ok(entries) = cache.lock() {
        for contact in contacts.iter_mut().take(MENU_AIRCRAFT_LIMIT) {
            let Some(registration) = contact.aircraft.registration.as_deref() else { continue };
            let Some((_, Some(photo))) = entries.get(&registration.trim().to_uppercase()) else { continue };
            contact.aircraft.photo_url = Some(photo.image.clone());
            contact.aircraft.photo_page = Some(photo.page.clone());
            contact.aircraft.photo_photographer = photo.photographer.clone();
        }
    }
}

fn fetch_jetphoto(agent: &ureq::Agent, registration: &str) -> Option<RemotePhoto> {
    let encoded = registration
        .bytes()
        .map(|byte| if byte.is_ascii_alphanumeric() || byte == b'-' { (byte as char).to_string() } else { format!("%{byte:02X}") })
        .collect::<String>();
    let url = format!("https://www.jetapi.dev/api?reg={encoded}&photos=1&flights=0&only_jp=true");
    let response = agent.get(&url).call().ok()?.body_mut().read_json::<JetApiResponse>().ok()?;
    let photo = response.images.into_iter().next()?;
    Some(RemotePhoto {
        image: photo.image,
        page: photo.link,
        photographer: photo.photographer.filter(|name| !name.trim().is_empty()),
    })
}

fn nearby_aircraft(aircraft: Vec<Aircraft>, home: Coords, radius_mi: u32) -> Vec<Contact> {
    let mut contacts: Vec<Contact> = aircraft
        .into_iter()
        .filter_map(|aircraft| {
            let (lat, lon) = (aircraft.lat?, aircraft.lon?);
            let distance_mi = distance_mi(home, (lat, lon));
            (distance_mi <= radius_mi as f64).then(|| Contact {
                direction: compass(bearing(home, (lat, lon))),
                aircraft,
                distance_mi,
            })
        })
        .collect();
    contacts.sort_by(|a, b| a.distance_mi.total_cmp(&b.distance_mi));
    contacts
}

fn distance_mi((lat1, lon1): (f64, f64), (lat2, lon2): (f64, f64)) -> f64 {
    let (lat1, lat2) = (lat1.to_radians(), lat2.to_radians());
    let dlat = lat2 - lat1;
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_MI * a.sqrt().asin()
}

fn bearing((lat1, lon1): (f64, f64), (lat2, lon2): (f64, f64)) -> f64 {
    let (lat1, lat2) = (lat1.to_radians(), lat2.to_radians());
    let dlon = (lon2 - lon1).to_radians();
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    y.atan2(x).to_degrees().rem_euclid(360.0)
}

fn compass(degrees: f64) -> &'static str {
    const POINTS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    POINTS[((degrees + 22.5) / 45.0) as usize % 8]
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: Coords = (40.0, -100.0);

    #[test]
    fn distance_and_direction() {
        // Roughly 10 miles due north of home.
        let north = (HOME.0 + 0.1447, HOME.1);
        assert!((distance_mi(HOME, north) - 10.0).abs() < 0.05);
        assert_eq!(compass(bearing(HOME, north)), "N");
        assert_eq!(compass(bearing(HOME, (HOME.0, HOME.1 - 0.1))), "W");
    }

    #[test]
    fn keeps_all_aircraft_inside_radius() {
        let plane = |hex: &str, lat| Aircraft {
            hex: hex.into(),
            lat: Some(lat),
            lon: Some(HOME.1),
            ..Default::default()
        };
        let contacts = nearby_aircraft(
            vec![
                plane("far", HOME.0 + 0.5),
                plane("civil", HOME.0),
                plane("near", HOME.0 + 0.01),
            ],
            HOME,
            10,
        );
        let hexes: Vec<_> = contacts.iter().map(|c| c.aircraft.hex.as_str()).collect();
        assert_eq!(hexes, ["near"]);
    }

    #[test]
    fn altitude_range() {
        let feet = Altitude::Feet(5000.0);
        let ground = Altitude::Ground("ground".into());
        let below = AltitudeRange {
            min_ft: None,
            max_ft: Some(10_000),
        };
        let above = AltitudeRange {
            min_ft: Some(1000),
            max_ft: None,
        };
        assert!(AltitudeRange::default().contains(Some(&ground)));
        assert!(below.contains(Some(&feet)) && below.contains(Some(&ground)));
        assert!(above.contains(Some(&feet)) && !above.contains(Some(&ground)));
        assert!(!below.contains(Some(&Altitude::Feet(35_000.0))));
        assert!(above.contains(None));
        assert_eq!(
            AltitudeRange {
                min_ft: Some(9000),
                max_ft: Some(2000)
            }
            .sorted(),
            AltitudeRange {
                min_ft: Some(2000),
                max_ft: Some(9000)
            }
        );
    }

    #[test]
    fn parses_ground_altitude() {
        let json =
            r#"{"aircraft":[{"hex":"ae1","alt_baro":"ground"},{"hex":"ae2","alt_baro":1200}]}"#;
        let response: Response = serde_json::from_str(json).unwrap();
        assert!(matches!(response.ac[0].alt_baro, Some(Altitude::Ground(_))));
        assert!(matches!(response.ac[1].alt_baro, Some(Altitude::Feet(f)) if f == 1200.0));
    }
}
