use http_client::native::NativeClient;
use surf::{Client as SurfClient, Request};

pub mod channel;
pub mod get_gateway;

#[derive(Debug)]
pub struct HttpClient {
  base_url: String,
  token: String,
  surf_client: SurfClient<NativeClient>,
}

impl HttpClient {
  pub fn new(token: String) -> Self {
    Self {
      token,
      base_url: "https://discord.com/api".to_owned(),
      surf_client: SurfClient::new(),
    }
  }

  fn with_auth(&self, request: Request<NativeClient>) -> Request<NativeClient> {
    request.set_header(
      "Authorization".parse().unwrap(),
      format!("Bot {}", self.token),
    )
  }

  fn url(&self, uri: &str) -> String {
    format!("{}{}", self.base_url, uri)
  }

  pub fn get(&self, uri: &str) -> Request<NativeClient> {
    self.with_auth(self.surf_client.get(self.url(uri)))
  }

  pub fn post(&self, uri: &str) -> Request<NativeClient> {
    self.with_auth(self.surf_client.post(self.url(uri)))
  }

  pub fn put(&self, uri: &str) -> Request<NativeClient> {
    self.with_auth(self.surf_client.put(self.url(uri)))
  }

  pub fn delete(&self, uri: &str) -> Request<NativeClient> {
    self.with_auth(self.surf_client.delete(self.url(uri)))
  }
}
