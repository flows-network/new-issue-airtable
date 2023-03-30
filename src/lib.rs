use airtable_flows::create_record;
use chrono::{DateTime, Datelike, Duration, Utc};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use schedule_flows::schedule_cron_job;
use serde::Deserialize;
use serde_json::Value;
use slack_flows::send_message_to_channel;

#[no_mangle]
pub fn run() {
    schedule_cron_job(
        String::from("36 * * * *"),
        String::from("cron_job_evoked"),
        callback,
    );
}

fn callback(_body: Vec<u8>) {
    let account: &str = "jaykchen";
    let base_id: &str = "apptJFYvsGrrywvWh";
    let table_name: &str = "users";

    let search_key_word = "GitHub WASMEDGE";
    let mut writer = Vec::new();

    let query_params: Value = serde_json::json!({
        "q": search_key_word,
        "sort": "created",
        "order": "desc"
    });

    let query_string = serde_urlencoded::to_string(&query_params).unwrap();
    let url_str = format!("https://api.github.com/search/issues?{}", query_string);

    let url = Uri::try_from(url_str.as_str()).unwrap();

    match Request::new(&url)
        .method(Method::GET)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/vnd.github.v3+json")
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {}
            let response: Result<SearchResult, _> = serde_json::from_slice(&writer);
            match response {
                Err(_e) => {}

                Ok(search_result) => {
                    let now = Utc::now();
                    // let one_hour_ago = now - Duration::minutes(60);
                    let one_day_earlier = now - Duration::minutes(1440);
                    for item in search_result.items {
                        let name = item.user.login;
                        let title = item.title;
                        let html_url = item.html_url;
                        let time = item.created_at;
                        let parsed = DateTime::parse_from_rfc3339(&time).unwrap_or_default();

                        if parsed > one_day_earlier {
                            // let text = format!(
                            //     "{name} mentioned WASMEDGE in issue: {title}  @{html_url}\n{time}"
                            // );

                            let data = serde_json::json!({
                                "Name": name,
                                "Repo": html_url,
                                "Created": time,
                            });
                            create_record(account, base_id, table_name, data.clone());

                            send_message_to_channel("ik8", "ch_out", data.to_string());
                        }
                    }
                }
            }
        }
        Err(_e) => {}
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
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}
