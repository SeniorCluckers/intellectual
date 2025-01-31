use actix_web::{get, web, Responder};
use askama::Template;
use futures::future;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::genius;
use crate::genius::{GeniusSong, GeniusSongRequest};
use crate::templates::template;
use crate::utils;

struct Verse {
    title: String,
    lyrics: Vec<String>,
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate {
    verses: Vec<Verse>,
    query: LyricsQuery,
    song: GeniusSong,
}

#[derive(Debug, Deserialize)]
pub struct LyricsQuery {
    path: String,
    api_path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<LyricsQuery>) -> impl Responder {
    let trimmed_api_path = info.api_path.trim_start_matches('/');
    let responses = future::join3(
        genius::text(genius::SubDomain::Api, trimmed_api_path),
        genius::text(genius::SubDomain::Root, info.path.trim_start_matches('/')),
        count_view(
            trimmed_api_path
                .trim_start_matches(|c: char| !c.is_ascii_digit())
                .parse::<u32>()
                .unwrap(),
        ),
    )
    .await;
    let api: GeniusSongRequest = serde_json::from_str(&responses.0).unwrap();
    let verses = scrape_lyrics(&responses.1);
    template(LyricsTemplate {
        verses,
        query: info.into_inner(),
        song: api.response.song,
    })
}

fn scrape_lyrics(doc: &str) -> Vec<Verse> {
    let document = Html::parse_document(doc);
    let parser = &Selector::parse("div[data-lyrics-container=true]").unwrap();

    let mut verses: Vec<Verse> = Vec::new();

    for x in document.select(parser).flat_map(|x| x.text()) {
        if x.starts_with('[') && x.ends_with(']') {
            verses.push(Verse {
                title: x.to_string(),
                lyrics: Vec::new(),
            })
        } else {
            if verses.is_empty() {
                verses.push(Verse {
                    title: "".to_string(),
                    lyrics: Vec::new(),
                })
            }
            let mut x1 = verses.remove(verses.len() - 1);
            x1.lyrics.push(x.to_string());
            verses.push(x1);
        }
    }

    if verses.is_empty() {
        verses.push(Verse {
            title: "This song has no lyrics".to_string(),
            lyrics: Vec::new(),
        })
    }

    verses
}

async fn count_view(id: u32) {
    let _ = Client::new()
        .post(format!("https://genius.com/api/songs/{}/count_view", id))
        .send()
        .await;
}
