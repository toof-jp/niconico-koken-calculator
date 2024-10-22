use std::collections::HashMap;
use std::env;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Ok, Result};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let user_session: SecretString = env::var("USER_SESSION")
        .expect("USER_SESSION is not set")
        .into();

    let mut sum_point = 0;
    let mut offset_add_id = None;
    let mut owener_count = HashMap::new();

    loop {
        let history = get_history(&user_session, offset_add_id).await.unwrap();

        for histories in history.data.histories {
            println!("{}: {}", histories.itemName, histories.point);
            sum_point += histories.point;
            *owener_count.entry(histories.ownerName).or_insert(0) += histories.point;
            offset_add_id = Some(histories.id);
        }

        if history.data.nextCount == 0 {
            break;
        }

        sleep(Duration::from_secs(1));
    }

    println!();
    println!("sum_point: {}", sum_point);
    println!();
    for (owner, point) in owener_count {
        println!("{}: {}", owner, point);
    }
}

async fn get_history(user_session: &SecretString, offset_add_id: Option<i64>) -> Result<History> {
    let mut url = "https://api.koken.nicovideo.jp/v2/my/histories".to_string();
    let offset_add_id = match offset_add_id {
        Some(offset_add_id) => offset_add_id.to_string(),
        None => "".to_string(),
    };
    url.push_str(&format!(
        "?contentType=nage_agv&offsetAdId={}&limit=30",
        offset_add_id
    ));

    let history = reqwest::Client::new()
        .get(&url)
        .header(reqwest::header::COOKIE, user_session.expose_secret())
        .send()
        .await?
        .json::<History>()
        .await?;

    Ok(history)
}

#[derive(Debug, Serialize, Deserialize)]
struct Histories {
    contentId: String,
    contentType: String,
    contribution: i64,
    endedAt: i64,
    id: i64,
    itemName: String,
    itemThumbnailUrl: String,
    #[serde(default)]
    ownerIcon: String,
    #[serde(default)]
    ownerId: i64,
    #[serde(default)]
    ownerName: String,
    point: i64,
    startedAt: i64,
    supporterName: String,
    tags: Vec<String>,
    targetUrl: String,
    thumbnailUrl: String,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    histories: Vec<Histories>,
    nextCount: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Meta {
    status: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct History {
    data: Data,
    meta: Meta,
}
