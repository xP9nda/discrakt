use chrono::{DateTime, Utc};
use discord_rich_presence::{
    activity::{Activity, Assets, Button, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use std::{thread::sleep, time::Duration};

use crate::{
    trakt::{Trakt, TraktWatchingResponse},
    utils::log,
};

pub struct Discord {
    client: DiscordIpcClient,
}

impl Discord {
    pub fn new(discord_client_id: String) -> Discord {
        Discord {
            client: match DiscordIpcClient::new(&discord_client_id) {
                Ok(client) => client,
                Err(e) => {
                    log(&format!("Couldn't connect to Discord: {e}"));
                    panic!("Couldn't connect to Discord");
                }
            },
        }
    }

    pub fn connect(&mut self) {
        loop {
            if self.client.connect().is_ok() {
                break;
            } else {
                log("Failed to connect to Discord, retrying in 15 seconds");
                sleep(Duration::from_secs(15));
            }
        }
    }

    pub fn close(&mut self) {
        self.client.close().unwrap();
    }

    pub fn set_activity(&mut self, trakt_response: &TraktWatchingResponse, trakt: &mut Trakt) {
        let details;
        let state;
        let media;
        let link_imdb;
        let link_trakt;
        let text_imdb;
        let text_trakt;
        let rating_text;
        let start_date = DateTime::parse_from_rfc3339(&trakt_response.started_at).unwrap();
        let end_date = DateTime::parse_from_rfc3339(&trakt_response.expires_at).unwrap();
        let now = Utc::now();
        let percentage = now.signed_duration_since(start_date).num_seconds() as f32
            / end_date.signed_duration_since(start_date).num_seconds() as f32;
        let watch_percentage = format!("{:.2}%", percentage * 100.0);
        let watch_percentage_remaining = format!("{:.2}%", 100.0 - (percentage * 100.0));
        let watch_text = format!("{} watched | {} remaining", watch_percentage, watch_percentage_remaining);

        match trakt_response.r#type.as_str() {
            "movie" => {
                let movie = trakt_response.movie.as_ref().unwrap();
                details = format!("{}", movie.title);
                state = format!("📅 {}", movie.year);

                rating_text = format!(
                    "⭐️ {:.1}/10",
                    Trakt::get_movie_rating(trakt, movie.ids.slug.as_ref().unwrap().to_string())
                        .as_ref()
                        .unwrap()
                );

                media = "movies";
                link_imdb = format!(
                    "https://www.imdb.com/title/{}",
                    movie.ids.imdb.as_ref().unwrap()
                );
                text_imdb = "View movie on IMDB";
                link_trakt = format!(
                    "https://trakt.tv/{}/{}",
                    media,
                    movie.ids.slug.as_ref().unwrap()
                );
                text_trakt = "View movie on Trakt";
            }
            "episode" if trakt_response.episode.is_some() => {
                let episode = trakt_response.episode.as_ref().unwrap();
                let show = trakt_response.show.as_ref().unwrap();
                details = show.title.to_string();

                // add a 0 infront of the episode number if it's less than 10
                let epsstring = if episode.number < 10 {
                    format!("0{}", episode.number)
                } else {
                    format!("{}", episode.number)
                }.to_string();

                state = format!("{}x{} \"{}\"", episode.season, &epsstring, episode.title);

                rating_text = format!(
                    "⭐️ {:.1}/10",
                    Trakt::get_episode_rating(trakt, show.ids.slug.as_ref().unwrap().to_string(), episode.season.to_string(), episode.number.to_string())
                        .as_ref()
                        .unwrap()
                );

                media = "shows";
                link_imdb = format!(
                    "https://www.imdb.com/title/{}",
                    show.ids.imdb.as_ref().unwrap()
                );
                text_imdb = "View show on IMDB";
                link_trakt = format!(
                    "https://trakt.tv/{}/{}",
                    media,
                    show.ids.slug.as_ref().unwrap()
                );
                text_trakt = "View show on Trakt";
            }
            _ => {
                log(&format!("Unknown media type: {}", trakt_response.r#type));
                return;
            }
        }

        log(&format!("{details} - {state} | {watch_percentage}"));

        let payload = Activity::new()
            .details(&details)
            .state(&state)
            .assets(
                Assets::new()
                    .large_image(media)
                    .large_text(&watch_text)
                    .small_image("rating")
                    .small_text(&rating_text),
            )
            .timestamps(
                Timestamps::new()
                    .start(start_date.timestamp())
                    .end(end_date.timestamp()),
            )
            .buttons(vec![
                Button::new(&text_imdb, &link_imdb),
                Button::new(&text_trakt, &link_trakt),
            ]);

        if self.client.set_activity(payload).is_err() {
            self.connect();
        }
    }
}
