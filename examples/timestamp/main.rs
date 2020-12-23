use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use encoding_rs::{UTF_8};
// const C: &'static [u8] = b"0123456789ABCDEF";
/*
    let mut signature = Vec::with_capacity(code_bytes.len() * 2);
    for byte in code_bytes {
        signature.push(C[(byte >> 4) as usize]);
        signature.push(C[(byte & 0xf) as usize]);
    }
*/
#[tokio::main]
async fn main() {
    let (access_key, method, version) = ("50d7bf42-bvrge3rf7j-b05b1e00-e53c7", "HmacSHA256", "2");
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    println!("timestamp = {}", timestamp);
    let param = format!(
        "AccessKeyId={}&SignatureMethod={}&SignatureVersion={}&Timestamp={}",
        urlencoding::encode(access_key),
        urlencoding::encode(method),
        urlencoding::encode(version),
        urlencoding::encode(timestamp.as_str()),
    );
    // let ep = UTF_8.encode(param.as_str()).0;
    // println!("{}", String::from_utf8(ep.to_vec()).unwrap() == param);
    let mut mac = Hmac::<Sha256>::new_varkey(b"eb5bbf7c-753935db-220bf939-735de").expect("HMAC can take key of any size");
    // let encode_param = urlencoding::encode(String::from_utf8(encode_param.to_vec()).unwrap().as_str());
    // println!("{}\n{}", param, encode_param);
    // let payload = vec!["GET", "api.huobi.pro", "/v1/order/orders", encode_param.as_str()];
    // let payload: String = payload.join("\n");
    let payload = format!("GET\napi.huobi.pro\n/v1/account/accounts\n{}", param);
    println!("payload = {}", payload);
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    println!("{:?}", code_bytes);
    let base64_e = base64::encode(code_bytes);
    println!("{}", base64_e);
    // let param = urlencoding::encode(format!("{}&Signature={}", param, base64_e).as_str());
    let url = format!(
        "https://api.huobi.pro/v1/account/accounts?{}&Signature={}",
        param, urlencoding::encode(base64_e.as_str())
        // String::from_utf8_lossy(base64_e.as_bytes())
    );
    println!("{}", url);
    let resp = reqwest::get(&url).await.unwrap();
    println!("{}", resp.text().await.unwrap());
}