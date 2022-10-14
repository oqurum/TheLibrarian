// Copy Paste from my other repo: https://github.com/Its-its/Image-Host/blob/main/src/upload/service/b2.rs

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use actix_web::web::Bytes;
use concread::EbrCell;
use lazy_static::lazy_static;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::{Digest, Sha1};
use tokio::runtime::Runtime;
use tokio::time::sleep;

use crate::config::ConfigStoreB2;
use crate::error::Result;
use crate::{Error, InternalError};

// const API_URL_V5: &str = "https://api.backblazeb2.com/b2api/v5";
// const API_URL_V4: &str = "https://api.backblazeb2.com/b2api/v4";
// const API_URL_V3: &str = "https://api.backblazeb2.com/b2api/v3";
const API_URL_V2: &str = "https://api.backblazeb2.com/b2api/v2";
// const API_URL_V1: &str = "https://api.backblazeb2.com/b2api/v1";

#[derive(Clone)]
struct AuthWrapper {
    credentials: Credentials,
    auth: B2Authorization,
    last_authed: Instant,
}

impl AuthWrapper {
    pub async fn re_auth(&mut self) -> Result<()> {
        self.auth = self.credentials.authorize().await?;
        self.last_authed = Instant::now();

        Ok(())
    }
}

lazy_static! {
    static ref AUTH: EbrCell<Option<AuthWrapper>> = EbrCell::new(None);
}

// TODO: Use check_and_update_auth for 401 error.

fn get_auth() -> Result<B2Authorization> {
    #[allow(clippy::unwrap_used)]
    Ok(AUTH.read().as_ref().unwrap().auth.clone())
}

async fn check_and_update_auth() -> Result<()> {
    #[allow(clippy::unwrap_used)]
    if AUTH.read().as_ref().unwrap().last_authed.elapsed() >= Duration::from_secs(60 * 60 * 16) {
        let mut wrapper = AUTH.write();

        let mutation = wrapper.get_mut();

        if let Err(e) = mutation.as_mut().unwrap().re_auth().await {
            eprintln!("{}", e);
        }

        wrapper.commit();
    }

    Ok(())
}

pub struct Service {
    bucket_id: String,

    pub directory: PathBuf,

    base_url: String,
}

impl Service {
    pub async fn new(config: &ConfigStoreB2) -> Result<Self> {
        if config.id.is_empty() {
            panic!("B2 Service ID is empty.");
        }

        if config.key.is_empty() {
            panic!("B2 Service Key is empty.");
        }

        if config.bucket_id.is_empty() {
            panic!("B2 Service Bucked ID is empty.");
        }

        // Spawn Authentication Thread.
        // TODO: Should I use it this way?
        thread::spawn(|| {
            #[allow(clippy::expect_used)]
            let rt = Runtime::new().expect("Thread Auth RT");

            loop {
                thread::sleep(Duration::from_secs(30));

                rt.block_on(async {
                    if let Err(e) = check_and_update_auth().await {
                        eprintln!("Auth Thread Error: {}", e);
                    }
                });
            }
        });

        let credentials = Credentials::new(&config.id, &config.key);
        let auth = credentials.authorize().await?;

        {
            let mut write = AUTH.write();

            *write.get_mut() = Some(AuthWrapper {
                credentials,
                auth,
                last_authed: Instant::now(),
            });

            write.commit();
        }

        Ok(Self {
            bucket_id: config.bucket_id.clone(),

            directory: PathBuf::from(&config.directory),
            base_url: config.base_url.clone(),
        })
    }

    pub fn get_http_url(&self, value: &str) -> Result<Url> {
        let mut url = Url::parse(&self.base_url)?;
        url.set_path(value);

        Ok(url)
    }

    pub async fn upload(&self, full_file_path: PathBuf, contents: Vec<u8>) -> Result<()> {
        let auth = get_auth()?;

        upload_file_multi_try(
            full_file_path
                .to_str()
                .ok_or_else(|| Error::from(InternalError::ConvertPathBufToString))?,
            contents,
            &auth,
            &self.bucket_id,
        )
        .await?;

        Ok(())
    }

    pub async fn hide_file(&self, full_file_path: PathBuf) -> Result<()> {
        let auth = get_auth()?;

        try_hide_file_multi(
            full_file_path
                .to_str()
                .ok_or_else(|| Error::from(InternalError::ConvertPathBufToString))?,
            &auth,
            &self.bucket_id,
        )
        .await?;

        Ok(())
    }
}

async fn upload_file_multi_try(
    file_name: &str,
    image_buffer: Vec<u8>,
    auth: &B2Authorization,
    bucket_id: &str,
) -> Result<()> {
    let image_buffer = Bytes::from(image_buffer);

    let mut prev_error = None;

    for _ in 0..5 {
        // For Some reason getting the upload url errors.
        let upload_url = match auth.get_upload_url(bucket_id).await {
            Ok(v) => v,
            Err(e) => {
                prev_error = Some(e);
                sleep(Duration::from_millis(1000)).await;
                continue;
            }
        };

        match auth
            .upload_file(&upload_url, file_name, image_buffer.clone())
            .await
        {
            Ok(Err(error)) => {
                prev_error = Some(error.into());
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            Err(error) => {
                prev_error = Some(error);
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            _ => (),
        }

        return Ok(());
    }

    #[allow(clippy::unwrap_used)]
    Err(prev_error.unwrap())
}

async fn try_hide_file_multi(
    file_path: &str,
    auth: &B2Authorization,
    bucket_id: &str,
) -> Result<()> {
    let mut prev_error = None;

    for _ in 0..5 {
        match auth.hide_file(bucket_id, file_path).await {
            Ok(Err(error)) => {
                // Ignore "Not Found" errors.
                if error.status == 404 {
                    return Ok(());
                }

                prev_error = Some(error.into());
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            Err(error) => {
                prev_error = Some(error);
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            _ => (),
        }

        return Ok(());
    }

    #[allow(clippy::unwrap_used)]
    Err(prev_error.unwrap())
}

#[derive(Clone)]
pub struct Credentials {
    pub id: String,
    pub key: String,
}

impl Credentials {
    pub fn new<S: Into<String>>(id: S, key: S) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
        }
    }

    fn header_name(&self) -> &str {
        "Authorization"
    }

    fn id_key(&self) -> String {
        format!("{}:{}", self.id, self.key)
    }

    pub fn auth_string(&self) -> String {
        format!("Basic {}", base64::encode(&self.id_key()))
    }

    pub async fn authorize(&self) -> Result<B2Authorization> {
        let client = reqwest::Client::new();

        let resp = client
            .get(format!("{}/b2_authorize_account", API_URL_V2).as_str())
            .header(self.header_name(), self.auth_string())
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(B2Authorization::new(self.id.clone(), resp.json().await?))
        } else {
            Err(resp.json::<JsonErrorStruct>().await?.into())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct B2AuthResponse {
    // account_id: String,
    // allowed: Object,
    absolute_minimum_part_size: usize,
    api_url: String,
    authorization_token: String,
    download_url: String,
    recommended_part_size: usize,
}

/// Authorization Token expires after 24 hours.
#[derive(Debug, Clone)]
pub struct B2Authorization {
    pub account_id: String,
    pub authorization_token: String,
    pub api_url: String,
    pub download_url: String,
    pub recommended_part_size: usize,
    pub absolute_minimum_part_size: usize,
}

impl B2Authorization {
    fn new(id: String, resp: B2AuthResponse) -> B2Authorization {
        B2Authorization {
            account_id: id,
            authorization_token: resp.authorization_token,
            api_url: resp.api_url,
            download_url: resp.download_url,
            recommended_part_size: resp.recommended_part_size,
            absolute_minimum_part_size: resp.absolute_minimum_part_size,
        }
    }

    pub async fn get_upload_url(&self, bucket_id: &str) -> Result<UploadUrlResponse> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({ "bucketId": bucket_id });

        let resp = client
            .post(format!("{}/b2api/v2/b2_get_upload_url", self.api_url).as_str())
            .header("Authorization", self.authorization_token.as_str())
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            eprintln!("get_upload_url: {:?}", resp.text().await?);
            Err(InternalError::B2GetUploadUrl.into())
        }
    }

    /// https://www.backblaze.com/b2/docs/b2_upload_file.html
    pub async fn upload_file(
        &self,
        upload: &UploadUrlResponse,
        file_name: &str,
        image: Bytes,
    ) -> Result<std::result::Result<serde_json::Value, JsonErrorStruct>> {
        let client = reqwest::Client::new();

        let sha = format!("{:X}", Sha1::digest(image.as_ref()));

        let resp = client
            .post(upload.upload_url.as_str())
            .header("Authorization", upload.authorization_token.as_str())
            .header("Content-Type", "b2/x-auto")
            .header("Content-Length", image.len())
            .header("X-Bz-File-Name", encode_file_name(file_name).as_str())
            .header("X-Bz-Content-Sha1", sha.as_str())
            .body(image)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(Ok(resp.json().await?))
        } else {
            Ok(Err(resp.json().await?))
        }
    }

    /// https://www.backblaze.com/b2/docs/b2_hide_file.html
    pub async fn hide_file(
        &self,
        bucket_id: &str,
        file_path: &str,
    ) -> Result<std::result::Result<serde_json::Value, JsonErrorStruct>> {
        let client = reqwest::Client::new();

        let body = json!({
            "bucketId": bucket_id,
            "fileName": encode_file_name(file_path)
        });

        let resp = client
            .post(format!("{}/b2api/v2/b2_hide_file", self.api_url).as_str())
            .header("Authorization", self.authorization_token.as_str())
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(Ok(resp.json().await?))
        } else {
            Ok(Err(resp.json().await?))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrlResponse {
    authorization_token: String,
    bucket_id: String,
    upload_url: String,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
#[error("Backblaze Error:\nStatus: {status},\nCode: {code},\nMessage: {message}")]
pub struct JsonErrorStruct {
    status: isize,
    code: String,
    message: String,
}

// Names can be pretty much any UTF-8 string up to 1024 bytes long. There are a few picky rules:
// No character codes below 32 are allowed.
// Backslashes are not allowed.
// DEL characters (127) are not allowed.
// File names cannot start with "/", end with "/", or contain "//".

pub fn encode_file_name(file_name: &str) -> String {
    let mut file_name = file_name
        .replace('\\', "/")
        .replace("//", "--")
        .replace(' ', "%20");

    if file_name.starts_with('/') {
        file_name.remove(0);
    }

    file_name
}
