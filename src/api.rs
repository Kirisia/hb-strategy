use std::collections::HashMap;
use hmac::{Hmac, NewMac, Mac};
use sha2::Sha256;
use reqwest::{Response, Client};
use std::time::Instant;

pub struct HbApi {
    access_key: String,
    secret_key: String,
    host: String,
    client: Client
}

type ResultResponse = Result<Response, reqwest::Error>;

impl HbApi {
    pub fn new<S: Into<String>>(access_key: S, secret_key: S, host: S) -> Self {
        let client = reqwest::Client::new();
        Self {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            host: host.into(),
            client,
        }
    }

    fn url_param<'a>(&self, map: &'a HashMap<&'a str, String>) -> String {
        let map = self.json_param(map);
        let mut param= String::new();
        let mut v = Vec::new();
        for key in map.keys() {
            v.push(key);
        }
        v.sort();
        for key in v {
            let val = map.get(key).unwrap();
            param = format!("{}{}={}&", param, key, urlencoding::encode(val));
        }
        param
    }

    fn json_param<'a>(&self, param: &'a HashMap<&str, String>) -> HashMap<&'a str, String> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut map = HashMap::new();
        map.insert("AccessKeyId", self.access_key.clone());
        map.insert("SignatureMethod", "HmacSHA256".into());
        map.insert("SignatureVersion", "2".into());
        map.insert("Timestamp", timestamp);
        for (key, val) in param {
            map.insert(key, val.clone());
        }
        map
    }

    fn hmac_base64_encode(&self, payload: String) -> String {
        let mut mac = Hmac::<Sha256>::new_varkey(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        let base64_e = urlencoding::encode(base64::encode(code_bytes).as_str());
        base64_e
    }

    pub async fn http_get<'a, S>(&self, req_url: S, param: &'a HashMap<&str, String>)
        -> ResultResponse
    where S: Into<String>
    {
        let param = self.url_param(param);
        let req_url = req_url.into();
        let payload = format!("GET\n{}\n{}\n{}", self.host, req_url, &param[..param.len() - 1]);
        let url = format!(
            "https://{}{}?{}Signature={}",
            self.host, req_url, param, self.hmac_base64_encode(payload)
        );
        println!("{}", url);
        reqwest::get(&url).await
    }

    pub async fn http_post<'a, S>(&self, req_url: S, param: &'a HashMap<&str, String>)
        -> ResultResponse
    where S: Into<String>
    {
        let param = self.url_param(&HashMap::new());
        let req_url = req_url.into();
        let payload = format!("POST\n{}\n{}\n{}", self.host, req_url, &param[..param.len() - 1]);
        let url = format!(
            "https://{}{}?{}Signature={}",
            self.host, req_url, param, self.hmac_base64_encode(payload)
        );
        println!("{}", url);
        self.client.post(&url)
            .json(&param)
            .send()
            .await
    }

    pub async fn get_symbols(&self) -> ResultResponse {
        self.http_get("/v1/common/symbols", &HashMap::new()).await
    }
}
