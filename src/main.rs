use anyhow::Result;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

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

    let addresses = [
        "0x186292Be4050F1FC6f51e624f6fc4532045829c2",
        "0xE801a0DE9DEaBde8bCEdeF412775a6A91F722150",
        "0x3664076A86FaD6cd211D47FcF7c117669e91E405",
    ];
    
    let joined = addresses.join(",");
    
    let client = reqwest::Client::new();
    
    let res: ApiResponse = client.get(BASE_URL).query(&[("module", "account"),("action", "balancemulti"),("address", joined.as_str()),]).send().await?.error_for_status()?.json().await?;
    
    if res.status != "1" {
        anyhow::bail!("API error: {}", res.message);
    }
    
    for item in res.result {
        let wei = Decimal::from_str(&item.balance)?;
        let dia = wei / Decimal::from(1_000_000_000_000_000_000u128);

        println!(
            "{} => {} DIA",
            item.account,
            dia,
        );
    }

    Ok(())
}
