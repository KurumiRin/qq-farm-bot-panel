use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const CHROME_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStatus {
    pub ret: String,
    pub msg: String,
    pub nickname: String,
    pub jump_url: String,
}

struct QrPreset {
    aid: &'static str,
    daid: &'static str,
    redirect_uri: &'static str,
    referrer: &'static str,
}

const VIP_PRESET: QrPreset = QrPreset {
    aid: "8000201",
    daid: "18",
    redirect_uri: "https://vip.qq.com/loginsuccess.html",
    referrer: "https://xui.ptlogin2.qq.com/cgi-bin/xlogin?appid=8000201&style=20&s_url=https%3A%2F%2Fvip.qq.com%2Floginsuccess.html&maskOpacity=60&daid=18&target=self",
};

const QZONE_PRESET: QrPreset = QrPreset {
    aid: "549000912",
    daid: "5",
    redirect_uri: "https://qzs.qzone.qq.com/qzone/v5/loginsucc.html?para=izone",
    referrer: "https://qzone.qq.com/",
};

pub struct QrLoginSession;

impl QrLoginSession {
    fn preset(key: &str) -> &'static QrPreset {
        match key {
            "qzone" => &QZONE_PRESET,
            _ => &VIP_PRESET,
        }
    }

    /// Request a QR code image for scanning
    /// Returns (qrsig cookie, base64 data URL of QR image)
    pub async fn request_qr_code(preset_key: &str) -> AppResult<(String, String)> {
        let config = Self::preset(preset_key);

        let url = format!(
            "https://ssl.ptlogin2.qq.com/ptqrshow?appid={}&e=2&l=M&s=3&d=72&v=4&t={}&daid={}&u1={}",
            config.aid,
            rand_f64(),
            config.daid,
            urlencoding::encode(config.redirect_uri),
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Referer", config.referrer)
            .header("User-Agent", CHROME_UA)
            .send()
            .await?;

        // Extract qrsig from Set-Cookie headers
        let qrsig = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|cookie_str| {
                cookie_str
                    .split(';')
                    .next()
                    .and_then(|part| {
                        let mut kv = part.splitn(2, '=');
                        let key = kv.next()?.trim();
                        let val = kv.next()?.trim();
                        if key == "qrsig" { Some(val.to_string()) } else { None }
                    })
            })
            .ok_or_else(|| AppError::Auth("No qrsig cookie in response".into()))?;

        let image_bytes = response.bytes().await?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
        let data_url = format!("data:image/png;base64,{}", b64);

        Ok((qrsig, data_url))
    }

    /// Check QR code scan status
    pub async fn check_status(qrsig: &str, preset_key: &str) -> AppResult<LoginStatus> {
        let config = Self::preset(preset_key);
        let ptqrtoken = djb2_hash(qrsig);

        let url = format!(
            "https://ssl.ptlogin2.qq.com/ptqrlogin?ptqrtoken={}&from_ui=1&aid={}&daid={}&action=0-0-{}&pt_uistyle=40&js_ver=21020514&js_type=1&u1={}",
            ptqrtoken,
            config.aid,
            config.daid,
            chrono::Utc::now().timestamp_millis(),
            urlencoding::encode(config.redirect_uri),
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Cookie", format!("qrsig={}", qrsig))
            .header("Referer", config.referrer)
            .header("User-Agent", CHROME_UA)
            .send()
            .await?;

        let text = response.text().await?;

        // Parse ptuiCB('ret','...','jumpUrl','...','msg','nickname')
        let args = parse_ptui_cb(&text)?;

        Ok(LoginStatus {
            ret: args.get(0).cloned().unwrap_or_default(),
            msg: args.get(4).cloned().unwrap_or_default(),
            nickname: args.get(5).cloned().unwrap_or_default(),
            jump_url: args.get(2).cloned().unwrap_or_default(),
        })
    }
}

/// Mini program login (QQ farm specific)
pub struct MiniProgramLoginSession;

const MP_QUA: &str = "V1_HT5_QDT_0.70.2209190_x64_0_DEV_D";

impl MiniProgramLoginSession {
    /// Request a login code and generate QR code
    pub async fn request_login_code() -> AppResult<(String, String)> {
        let client = reqwest::Client::new();
        let response: serde_json::Value = client
            .get("https://q.qq.com/ide/devtoolAuth/GetLoginCode")
            .header("qua", MP_QUA)
            .header("host", "q.qq.com")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("user-agent", CHROME_UA)
            .send()
            .await?
            .json()
            .await?;

        let code = response["data"]["code"]
            .as_str()
            .ok_or_else(|| AppError::Auth("No login code in response".into()))?
            .to_string();

        let login_url = format!("https://h5.qzone.qq.com/qqq/code/{}?_proxy=1&from=ide", code);

        // Generate QR code as SVG string (frontend can render it)
        let qr = qrcode::QrCode::new(login_url.as_bytes())
            .map_err(|e| AppError::Auth(format!("QR code generation failed: {}", e)))?;

        let svg = qr
            .render::<qrcode::render::svg::Color>()
            .min_dimensions(300, 300)
            .build();

        let data_url = format!(
            "data:image/svg+xml;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(svg.as_bytes())
        );

        Ok((code, data_url))
    }

    /// Query login status for mini program flow
    pub async fn query_status(code: &str) -> AppResult<MpLoginResult> {
        let client = reqwest::Client::new();
        let response: serde_json::Value = client
            .get(&format!(
                "https://q.qq.com/ide/devtoolAuth/syncScanSateGetTicket?code={}",
                code
            ))
            .header("qua", MP_QUA)
            .header("host", "q.qq.com")
            .header("accept", "application/json")
            .header("user-agent", CHROME_UA)
            .send()
            .await?
            .json()
            .await?;

        let res_code = response["code"].as_i64().unwrap_or(-1);

        if res_code == 0 {
            let data = &response["data"];
            if data["ok"].as_i64().unwrap_or(0) != 1 {
                return Ok(MpLoginResult::Waiting);
            }
            return Ok(MpLoginResult::Success {
                ticket: data["ticket"].as_str().unwrap_or("").to_string(),
                uin: data["uin"].as_str().unwrap_or("").to_string(),
                nickname: data["nick"].as_str().unwrap_or("").to_string(),
            });
        }

        if res_code == -10003 {
            return Ok(MpLoginResult::Used);
        }

        Ok(MpLoginResult::Error(format!("Code: {}", res_code)))
    }

    /// Exchange ticket for auth code
    pub async fn get_auth_code(ticket: &str, appid: &str) -> AppResult<String> {
        let client = reqwest::Client::new();
        let response: serde_json::Value = client
            .post("https://q.qq.com/ide/login")
            .header("qua", MP_QUA)
            .header("host", "q.qq.com")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("user-agent", CHROME_UA)
            .json(&serde_json::json!({
                "appid": appid,
                "ticket": ticket,
            }))
            .send()
            .await?
            .json()
            .await?;

        Ok(response["code"].as_str().unwrap_or("").to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MpLoginResult {
    Waiting,
    Success {
        ticket: String,
        uin: String,
        nickname: String,
    },
    Used,
    Error(String),
}

/// DJB2 hash function (used for ptqrtoken)
fn djb2_hash(s: &str) -> i64 {
    let mut hash: i64 = 5381;
    for c in s.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as i64);
    }
    hash & 0x7FFFFFFF
}

/// Parse ptuiCB('arg1','arg2',...) response
fn parse_ptui_cb(text: &str) -> AppResult<Vec<String>> {
    let start = text.find("ptuiCB(").ok_or_else(|| AppError::Auth("Invalid ptui response".into()))?;
    let inner = &text[start + 7..];
    let end = inner.find(')').unwrap_or(inner.len());
    let inner = &inner[..end];

    let mut args = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for ch in inner.chars() {
        match ch {
            '\'' if !in_quote => {
                in_quote = true;
                current.clear();
            }
            '\'' if in_quote => {
                in_quote = false;
                args.push(current.clone());
            }
            c if in_quote => current.push(c),
            _ => {}
        }
    }

    Ok(args)
}

fn rand_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64) / 1_000_000_000.0
}
