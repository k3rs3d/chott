use crate::error::AppError;
use crate::pages::PageId;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::trace;

#[derive(Debug)]
pub struct WorldTime {
    pub hour: u8,
    pub _minute: u8,
}
impl WorldTime {
    /// Returns true if time is daytime (6:00 <= hour < 18:00)
    pub fn is_daytime(&self) -> bool {
        self.hour >= 6 && self.hour < 18
    }

    /// Returns true if time is nighttime (18:00 <= hour or hour < 6:00)
    pub fn is_night(&self) -> bool {
        !self.is_daytime()
    }

    /// is it dusk/dawn (+/-1 hour from boundary)?
    pub fn _is_twilight(&self) -> bool {
        (self.hour >= 5 && self.hour < 7) || (self.hour >= 17 && self.hour < 19)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Environment {
    season: String,
    weather: String,
    timestamp: SystemTime,
}

#[derive(Clone)]
pub struct EnvironmentManager {
    pub cache: Arc<Mutex<HashMap<PageId, Environment>>>,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        EnvironmentManager {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_environment_for_page(
        &self,
        page_id: &PageId,
    ) -> Result<Environment, AppError> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|e| AppError::MutexError(format!("Failed to lock cache: {e}")))?;
        if let Some(env) = cache.get(page_id) {
            //let elapsed = env.timestamp.elapsed()?;
            {
                trace!("Env cache hit for {page_id}");
                return Ok(env.clone());
            }
        }
        trace!("Env cache miss for {page_id}");
        // Generate new environment if missing or expired
        let new_env = self.generate_environment(page_id).map_err(|e| {
            AppError::EnvironmentError(format!("Failed to generate environment: {e}"))
        })?;
        cache.insert(page_id.to_owned(), new_env.clone());
        Ok(new_env)
    }

    fn generate_environment(&self, _page_id: &PageId) -> Result<Environment, AppError> {
        // Use system time, rng, etc.
        let now = SystemTime::now();
        let season = compute_season(now);
        let weather = random_weather();
        Ok(Environment {
            season,
            weather,
            timestamp: now,
        })
    }
}

fn compute_season(now: SystemTime) -> String {
    // Use month for season
    let datetime = chrono::DateTime::<chrono::Utc>::from(now);
    let month = datetime.month();
    match month {
        12 | 1 | 2 => "Winter",
        3 | 4 | 5 => "Spring",
        6 | 7 | 8 => "Summer",
        9 | 10 | 11 => "Autumn",
        _ => "Unknown",
    }
    .to_string()
}

fn random_weather() -> String {
    // use system time as pseudorandom source for weather, rotates every minute
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let idx = (now / 60) % 4;
    match idx {
        0 => "Clear",
        1 => "Rainy",
        2 => "Cloudy",
        3 => "Windy",
        _ => "Foggy",
    }
    .to_string()
}
