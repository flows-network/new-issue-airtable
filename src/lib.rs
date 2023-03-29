use airtable_flows::create_record;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use http_req::{request::Request, response::Headers, uri::Uri};
use schedule_flows::schedule_cron_job;
use serde::Deserialize;
use serde_json::Value;
use slack_flows::send_message_to_channel;
use std::convert::TryFrom;
use std::env;
use store_flows::{global_get, global_set};

#[no_mangle]
pub fn run() {
    schedule_cron_job(
        String::from("35 * * * *"),
        String::from("cron_job_evoked"),
        callback,
    );
}

fn callback(_body: Vec<u8>) {
    dotenv().ok();

    let github_token: String =
        env::var("github_token").unwrap_or("GitHub token not found".to_string());
    let account: &str = "jaykchen";
    let base_id: &str = "apptJFYvsGrrywvWh";
    let table_name: &str = "users";

    let search_key_word = "GitHub WASMEDGE";

    let url_str = format!(
        "https://api.github.com/search/issues?q={}&sort=created&order=desc",
        search_key_word
    );
    let bearer_token = format!("bearer {}", github_token);
    let url = Uri::try_from(url_str.as_str()).unwrap();

    let mut writer = Vec::<u8>::new();

    let mut headers = Headers::new();
    headers.insert("User-Agent", "flows-network connector");
    headers.insert("Authorization", &bearer_token);
    headers.insert("Accept", "application/vnd.github+json");

    Request::new(&url)
        .headers(headers)
        .send(&mut writer)
        .unwrap();

    let res: Result<SearchResult, _> = serde_json::from_slice(&writer);

    match res {
        Err(_e) => {}

        Ok(search_result) => {
            let time_entries_last_saved: Option<DateTime<Utc>> =
                match global_get("time_entries_last_saved")
                    .unwrap()
                    .to_string()
                    .parse()
                {
                    Ok(t) => Some(t),
                    Err(_) => None,
                };
            for item in search_result.items {
                let name = item.user.login;
                let title = item.title;
                send_message_to_channel("ik8", "general", title.to_string());
                let html_url = item.html_url;
                let time = item.created_at;

                if time_entries_last_saved.is_none()
                    || time_entries_last_saved.is_some() && time > time_entries_last_saved.unwrap()
                {
                    let text =
                        format!("{name} mentioned WASMEDGE in issue: {title}  @{html_url}\n{time}");
                    send_message_to_channel("ik8", "general", text);

                    let data = serde_json::json!({
                        "Name": name,
                        "Repo": html_url,
                        "Created": time,
                    });
                    create_record(account, base_id, table_name, data.clone());
                    let time_serialized: Value =
                        serde_json::to_value(&time).expect("Failed to serialize date-time");
                    global_set("time_entries_last_saved", time_serialized);
                    send_message_to_channel("ik8", "ch_out", data.to_string());
                }
                send_message_to_channel("ik8", "ch_mid", time.to_string());
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    items: Vec<Issue>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    html_url: String,
    title: String,
    user: User,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}
