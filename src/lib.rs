use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::error::Error;

pub mod response;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct TencentCloudClient {
    secret_id: String,
    secret_key: String,
    service: String,
    host: String,
    region: String,
    token: String,
}

impl TencentCloudClient {
    pub fn new(
        service: &str,
        host: &str,
        region: &str,
        secret_id: &str,
        secret_key: &str,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            secret_id: secret_id.to_string(),
            secret_key: secret_key.to_string(),
            service: service.to_string(),
            host: host.to_string(),
            region: region.to_string(),
            token: String::new(),
        })
    }

    pub fn with_token(mut self, token: &str) -> Self {
        self.token = token.to_string();
        self
    }

    pub async fn request(
        &self,
        action: &str,
        version: &str,
        payload: serde_json::Value,
    ) -> Result<String, Box<dyn Error>> {
        let timestamp = Utc::now().timestamp();
        let date: DateTime<Utc> = DateTime::from_timestamp(timestamp, 0).unwrap();
        let date_str = date.format("%Y-%m-%d").to_string();

        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_querystring = "";
        let canonical_headers = format!(
            "content-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\n",
            self.host,
            action.to_lowercase()
        );
        let signed_headers = "content-type;host;x-tc-action";

        let payload_str = payload.to_string();
        let hashed_request_payload = Self::sha256_hex(&payload_str);

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            http_request_method,
            canonical_uri,
            canonical_querystring,
            canonical_headers,
            signed_headers,
            hashed_request_payload
        );

        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{}/{}/tc3_request", date_str, self.service);
        let hashed_canonical_request = Self::sha256_hex(&canonical_request);

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp, credential_scope, hashed_canonical_request
        );

        let secret_date = Self::hmac_sha256(
            format!("TC3{}", self.secret_key).as_bytes(),
            date_str.as_bytes(),
        );
        let secret_service = Self::hmac_sha256(&secret_date, self.service.as_bytes());
        let secret_signing = Self::hmac_sha256(&secret_service, b"tc3_request");
        let signature = Self::hmac_sha256_hex(&secret_signing, string_to_sign.as_bytes());

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.secret_id, credential_scope, signed_headers, signature
        );

        let client = Client::new();
        let url = format!("https://{}", self.host);

        let response = client
            .post(&url)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", &self.host)
            .header("X-TC-Action", action)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Version", version)
            .header("X-TC-Region", &self.region)
            .header("X-TC-Token", &self.token)
            .body(payload_str)
            .send()
            .await?;

        let response_text = response.text().await?;
        Ok(response_text)
    }

    fn sha256_hex(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        format!("{:x}", mac.finalize().into_bytes())
    }
}
