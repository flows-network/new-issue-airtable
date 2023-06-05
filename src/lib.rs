use airtable_flows::create_record;
use chrono::{Duration, Utc};
use dotenv::dotenv;
use github_flows::{get_octo, GithubLogin::Default};
use schedule_flows::schedule_cron_job;
use slack_flows::send_message_to_channel;
use std::env;
use tokio;

#[no_mangle]
pub fn run() {
    dotenv().ok();
    //time_to_invoke is a string of 3 numbers separated by spaces, representing minute, hour, and day
    //* is the spaceholder for non-specified numbers
    let mut time_to_invoke = env::var("time_to_invoke").unwrap_or("* 12 *".to_string());
    time_to_invoke.push_str(" * *");

    schedule_cron_job(time_to_invoke, String::from("cron_job_evoked"), callback);
}

#[tokio::main(flavor = "current_thread")]
async fn callback(_body: Vec<u8>) {
    let github_owner = env::var("github_owner").unwrap_or("WASMEDGE".to_string());
    let github_repo = env::var("github_repo").unwrap_or("WASMEDGE".to_string());
    let airtable_token_name = env::var("airtable_token_name").unwrap_or("a-test".to_string());
    let airtable_base_id = env::var("airtable_base_id").unwrap_or("apptJFYvsGrrywvWh".to_string());
    let airtable_table_name = env::var("airtable_table_name").unwrap_or("users".to_string());

    let n_days = env::var("n_days").unwrap_or("1".to_string());

    let n_days_ago_formatted = Utc::now()
        .checked_sub_signed(Duration::days(n_days.parse::<i64>().unwrap_or(1)))
        .unwrap_or(Utc::now())
        .date_naive();
    let query = format!(
        "repo:{github_owner}/{github_repo} is:issue state:open comments:0 updated:>{n_days_ago_formatted}"
    );

    let octocrab = get_octo(&Default);

    let res = octocrab
        .search()
        .issues_and_pull_requests(&query)
        .send()
        .await;

    if let Ok(page) = res {
        for item in page {
            let name = item.user.login;
            let title = item.title;
            let html_url = item.html_url;
            let time = item.created_at;

            let data = serde_json::json!({
                "Name": name,
                "Repo": html_url,
                "Created": time,
            });
            create_record(
                &airtable_token_name,
                &airtable_base_id,
                &airtable_table_name,
                data.clone(),
            );

            send_message_to_channel("ik8", "ch_out", data.to_string());
        }
    }
}
