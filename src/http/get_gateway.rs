use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetGateway {
  pub url: String,
}

pub async fn get_gateway() -> Result<GetGateway, Box<dyn std::error::Error + Send + Sync>> {
  let uri = "https://discord.com/api/gateway";
  let response: GetGateway = surf::get(uri).recv_json().await?;
  Ok(response)
}
