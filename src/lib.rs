use airtable_flows::create_record;
use github_flows::{
    get_octo, listen_to_event,
    octocrab::Result,
    octocrab::{self},
    EventPayload,
};
use schedule_flows::schedule_cron_job;
use serde::Deserialize;
use slack_flows::send_message_to_channel;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    let login = "jaykchen";
    let owner = "jaykchen";
    let repo = "vitesse-lite";

    listen_to_event(login, owner, repo, vec!["pull_request"], |payload| {
        handler(login, owner, repo, payload)
    })
    .await;
}

async fn handler(login: &str, owner: &str, repo: &str, payload: EventPayload) {
    let account: &str = "jaykchen";
    let base_id: &str = "apptJFYvsGrrywvWh";
    let table_name: &str = "users";

    let octocrab = get_octo(Some(String::from(login)));

    let query_str = "search/issues?q=WASMEDGE&sort=created&order=desc".to_string();

    send_message_to_channel("ik8", "ch_in", query_str.to_string());

    let response: Result<SearchResult, octocrab::Error> =
        octocrab.get(&query_str, None::<&()>).await;

    match response {
        Err(e) => {}

        Ok(response) => {
            for item in response.items {
                let name = item.user.login;
                let title = item.title;
                send_message_to_channel("ik8", "general", title.to_string());
                let html_url = item.html_url;
                let time = item.created_at;

                let text =
                    format!("{name} mentioned WASMEDGE in issue: {title}  @{html_url}\n{time}");
                send_message_to_channel("ik8", "general", text);

                let data = serde_json::json!({
                    "Name": name,
                    "Repo": html_url,
                    "Created": time,
                });
                create_record(account, base_id, table_name, data);
            }
        }
    };
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
