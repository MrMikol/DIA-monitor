use anyhow::Result;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use serde_json::json;
use dotenvy::dotenv;
use dotenvy::from_path;

const BASE_URL: &str = "https://explorer.diadata.org/api";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: String,
    message: String,
    result: Vec<BalanceResult>,
}

#[derive(Debug, Deserialize)]
struct BalanceResult {
    account: String,
    balance: String,
    #[serde(default)]
    stale: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    from_path("/etc/opt/app/slack/.env").ok();
    let addresses = [
        "0x186292Be4050F1FC6f51e624f6fc4532045829c2",
        "0xE801a0DE9DEaBde8bCEdeF412775a6A91F722150",
        "0x3664076A86FaD6cd211D47FcF7c117669e91E405",
    ];

    let joined = addresses.join(",");
    let client = reqwest::Client::new();
    let res: ApiResponse = client.get(BASE_URL).query(&[("module", "account"),("action", "balancemulti"),("address", joined.as_str()),]).send().await?.error_for_status()?.json().await?;

    let mut slack_text = String::from("DIA Balance\n\n");
    let threshold = Decimal::from_str("1.0")?;
    let slack_webhook_url = std::env::var("SLACK_WEBHOOK_URL")?;


    if res.status != "1" {
        anyhow::bail!("API error: {}", res.message);
    }

    //println!("DIA Balance\n");

    for item in res.result {
        let wei = Decimal::from_str(&item.balance)?;
        let dia = wei / Decimal::from(1_000_000_000_000_000_000u128);

        let indicator = if dia > threshold {
            ":large_green_circle:"
        } else {
            ":large_red_circle:"
        };

        slack_text.push_str(&format!(
            "{} Balance ({:.8} DIA) > 1.0 DIA `[{}]`\n",
            indicator,
            dia,
            item.account
        ));
    }

    client
        .post(slack_webhook_url)
        .json(&json!({
        "text": slack_text
    }))
        .send()
        .await?
        .error_for_status()?;

    println!("Success");
    
    Ok(())
}
