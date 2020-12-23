use hmac::{Hmac, NewMac, Mac};
use sha2::Sha256;
use hb_buy_strategy::api::HbApi;
use std::collections::HashMap;

const C: &'static [u8] = b"0123456789ABCDEF";

#[tokio::main]
async fn main() {
    let mut mac = Hmac::<Sha256>::new_varkey(b"eb5bbf7c-753935db-220bf939-735de").expect("...");
    mac.update(b"GET\napi.huobi.pro\n/v1/account/accounts\nAccessKeyId=50d7bf42-bvrge3rf7j-b05b1e00-e53c7&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp=2020-12-22T11%3A08%3A43");
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    // let mut signature = Vec::with_capacity(code_bytes.len() * 2);
    // for byte in code_bytes {
    //     signature.push(C[(byte >> 4) as usize]);
    //     signature.push(C[(byte & 0xf) as usize]);
    // }
    // println!("{}", String::from_utf8(signature).unwrap());
    let base64_e = base64::encode(code_bytes);// base64::encode_config(signature.to_ascii_lowercase(), base64::STANDARD);
    println!("{}", base64_e);

    // let s = "我才第三方库就192dsfjj==&".to_string();
    // println!("{}", &s[0..s.len() - 1]);
    let api = HbApi::new(
        "50d7bf42-bvrge3rf7j-b05b1e00-e53c7",
        "eb5bbf7c-753935db-220bf939-735de",
        "api.huobi.pro"
    );

    let resp = api.http_get("/v1/account/accounts", &HashMap::new()).await;
    println!("{}", resp.text().await.unwrap());
}