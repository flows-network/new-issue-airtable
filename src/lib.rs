use airtable_flows::create_record;
use flowsnet_platform_sdk::write_error_log;
use github_flows::{get_octo, octocrab};
use schedule_flows::schedule_cron_job;
use slack_flows::send_message_to_channel;
use tokio::spawn;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    schedule_cron_job(
        String::from("30 1 * * *"),
        String::from("cron_job_evoked"),
        |body| {
            spawn(async {
                let login: &str = "jaykchen";
                let account: &str = "jaykchen";
                let base_id: &str = "apptJFYvsGrrywvWh";
                let table_name: &str = "users";

                let octo = get_octo(Some(String::from(login)));

                let pages = octocrab::instance()
                    .search()
                    .issues_and_pull_requests("GitHub WASMEDGE")
                    .sort("comments")
                    .order("asc")
                    .send()
                    .await;

                match pages {
                    Err(_e) => {
                        write_error_log!( _e.to_string());
                    }

                    Ok(pages) => {
                        for item in pages {
                            let name = item.author_association;
                            let html_url = item.html_url;
                            let time = item.created_at;

                            let text = format!("{name} mentioned WASMEDGE  @{html_url}\n{time}");
                            send_message_to_channel("ik8", "general", text);

                            let data = serde_json::json!({
                            "Name": name,
                            "Repo": html_url,
                            "Created": time,
                            });
                            create_record(account, base_id, table_name, data)
                        }
                    }
                }
            });
        },
    );
}
