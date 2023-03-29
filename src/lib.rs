use airtable_flows::create_record;
use github_flows::{
    get_octo,
    octocrab::Result,
    octocrab::{self},
};
use schedule_flows::schedule_cron_job;
use serde::Deserialize;
use slack_flows::send_message_to_channel;


#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    schedule_cron_job(
        String::from("25 * * * *"),
        String::from("cron_job_evoked"),
        Box::new(|body: Vec<u8>| {
            Box::pin(async move {
                let login: &str = "jaykchen";
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
                            let html_url = item.html_url;
                            let time = item.created_at;
                            
                            let text = format!("{name} mentioned WASMEDGE in issue: {title}  @{html_url}\n{time}");
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
                ()
            });
            ()
        }),
    );
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
