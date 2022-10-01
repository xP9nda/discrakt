use serde::Deserialize;
use std::{collections::HashMap, time::Duration};
use ureq::{Agent, AgentBuilder};

#[derive(Deserialize)]
pub struct TraktMovie {
    pub title: String,
    pub year: u16,
    pub ids: TraktIds,
}

#[derive(Deserialize)]
pub struct TraktShow {
    pub title: String,
    pub year: u16,
    pub ids: TraktIds,
}

#[derive(Deserialize)]
pub struct TraktEpisode {
    pub season: u8,
    pub number: u8,
    pub title: String,
    pub ids: TraktIds,
}

#[derive(Deserialize)]
pub struct TraktIds {
    pub trakt: u32,
    pub slug: Option<String>,
    pub tvdb: Option<u32>,
    pub imdb: Option<String>,
    pub tmdb: Option<u32>,
    pub tvrage: Option<u32>,
}

#[derive(Deserialize)]
pub struct TraktWatchingResponse {
    pub expires_at: String,
    pub started_at: String,
    pub action: String,
    pub r#type: String,
    pub movie: Option<TraktMovie>,
    pub show: Option<TraktShow>,
    pub episode: Option<TraktEpisode>,
}

#[derive(Deserialize)]
pub struct TraktRatingsResponse {
    pub rating: f64,
    pub votes: u32,
    pub distribution: HashMap<String, u16>,
}

pub struct Trakt {
    cache: HashMap<String, f64>,
    agent: Agent,
    client_id: String,
    username: String,
}

impl Trakt {
    pub fn new(client_id: String, username: String) -> Trakt {
        Trakt {
            cache: HashMap::default(),
            agent: AgentBuilder::new()
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
            client_id,
            username,
        }
    }

    pub fn get_watching(&self) -> Option<TraktWatchingResponse> {
        let endpoint = format!("https://api.trakt.tv/users/{}/watching", self.username);

        let response = match self
            .agent
            .get(&endpoint)
            .set("Content-Type", "application/json")
            .set("trakt-api-version", "2")
            .set("trakt-api-key", &self.client_id)
            .call()
        {
            Ok(response) => response,
            Err(_) => return None,
        };

        match response.into_json() {
            Ok(body) => body,
            Err(_) => None,
        }
    }

    pub fn get_movie_rating(&mut self, movie_slug: String) -> Option<f64> {
        match self.cache.get(&movie_slug) {
            Some(rating) => Some(*rating),
            None => {
                let endpoint = format!("https://api.trakt.tv/movies/{}/ratings", movie_slug);

                let response = match self
                    .agent
                    .get(&endpoint)
                    .set("Content-Type", "application/json")
                    .set("trakt-api-version", "2")
                    .set("trakt-api-key", &self.client_id)
                    .call()
                {
                    Ok(response) => response,
                    Err(_) => return Some(0.0),
                };

                match response.into_json() {
                    Ok(body) => {
                        let body: TraktRatingsResponse = body;
                        self.cache.insert(movie_slug.to_string(), body.rating);
                        Some(body.rating)
                    }
                    Err(_) => Some(0.0),
                }
            }
        }
    }

    pub fn get_episode_rating(&mut self, show_slug: String, season_num: String, episode_num: String) -> Option<f64> {
        match self.cache.get(&show_slug) {
            Some(rating) => Some(*rating),
            None => {
                let endpoint = format!("https://api.trakt.tv/shows/{}/seasons/{}/episodes/{}/ratings", show_slug, season_num,episode_num);

                let response = match self
                    .agent
                    .get(&endpoint)
                    .set("Content-Type", "application/json")
                    .set("trakt-api-version", "2")
                    .set("trakt-api-key", &self.client_id)
                    .call()
                {
                    Ok(response) => response,
                    Err(_) => return Some(0.0),
                };

                match response.into_json() {
                    Ok(body) => {
                        let body: TraktRatingsResponse = body;
                        self.cache.insert(show_slug.to_string(), body.rating);
                        Some(body.rating)
                    }
                    Err(_) => Some(0.0),
                }
            }
        }
    }
}
