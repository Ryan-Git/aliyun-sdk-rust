use crate::client::*;

use chrono::{Local, Utc};
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha1::Sha1;
use serde::{Deserialize, Serialize};
use url::form_urlencoded::byte_serialize;

pub struct SmsRequest<T: Serialize> {
    pub phones: Vec<String>,
    pub sign_name: String,
    pub template_code: String,
    pub out_id: Option<String>,
    pub param: T,
}

const FIXED_SMS_PARAM: &[(&str, &str)] = &[
    ("Action", "SendSms"),
    ("Format", "JSON"),
    ("RegionId", "cn-hangzhou"),
    ("SignatureMethod", "HMAC-SHA1"),
    ("SignatureVersion", "1.0"),
    ("Version", "2017-05-25"),
];

impl Client {
    pub fn send_sms<T: Serialize>(&self, req: SmsRequest<T>) -> Result<()> {
        let nonce = format!("{}", Local::now().timestamp_subsec_nanos());
        let ts = format!("{}", Utc::now().format("%Y-%m-%dT%H:%M:%SZ"));
        let param = serde_json::to_string(&req.param).unwrap();
        let phones = req.phones.join(",");

        let mut params = Vec::from(FIXED_SMS_PARAM);
        params.push(("AccessKeyId", &self.access_key));
        params.push(("SignName", &req.sign_name));
        params.push(("TemplateCode", &req.template_code));
        params.push(("Timestamp", &ts));
        params.push(("TemplateParam", &param));
        params.push(("SignatureNonce", &nonce));
        params.push(("PhoneNumbers", &phones));
        params.sort_by_key(|item| item.0);
        let params: Vec<String> = params
            .into_iter()
            .map(|(k, v)| format!("{}={}", special_url_encode(k), special_url_encode(v)))
            .collect();
        let sorted_query_string = params.join("&");
        let string_to_sign = format!(
            "GET&{}&{}",
            special_url_encode("/"),
            special_url_encode(&sorted_query_string)
        );
        let sign = sign(format!("{}&", &self.secret), &string_to_sign);
        let signature = special_url_encode(&sign);
        let final_url = format!(
            "http://dysmsapi.aliyuncs.com/?Signature={}&{}",
            signature, sorted_query_string
        );
        self.http
            .get(&final_url)
            .send()?
            .json::<SmsResponse>()
            .map_err(From::from)
            .and_then(|resp| {
                if resp.code.eq("OK") {
                    Ok(())
                } else {
                    Err(Error::Internal(resp.message))
                }
            })
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct SmsResponse {
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "BizId")]
    biz_id: String,
    #[serde(rename = "Code")]
    code: String,
}

fn special_url_encode(s: &str) -> String {
    let s: String = byte_serialize(s.as_bytes()).collect();
    s.replace("+", "%20")
        .replace("*", "%2A")
        .replace("%7E", "~")
}

fn sign<S: Into<String>>(key: S, body: &str) -> String {
    let mut mac = Hmac::new(Sha1::new(), key.into().as_bytes());
    mac.input(body.as_bytes());
    let result = mac.result();
    let code = result.code();
    base64::encode(code)
}

#[cfg(test)]
mod tests {
    use crate::sms::sign;
    use crate::*;
    use std::collections::HashMap;
}
