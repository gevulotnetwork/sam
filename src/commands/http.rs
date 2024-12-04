
use reqwest::header::HeaderMap;
use rhai::{Dynamic, EvalAltResult, Position};

fn get_url_and_headers(options: &Dynamic) -> Result<(String, HeaderMap), Box<EvalAltResult>> {
    let mut url = options
        .as_map_ref()?
        .get("url")
        .ok_or(Box::new(EvalAltResult::ErrorRuntime(
            "Missing 'url' parameter".into(),
            Position::NONE,
        )))?
        .to_owned()
        .into_string()?;

    let params: Vec<(String, String)> =
        if let Some(params) = options.as_map_ref()?.get("params") {
            params
                .as_map_ref()
                .map(|p| {
                    p.iter()
                        .map(|(key, value)| {
                            (key.to_owned().to_string(), value.to_owned().to_string())
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

    if !params.is_empty() {
        url = format!(
            "{}?{}",
            url,
            params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("&")
        );
    }

    let headers = options
        .as_map_ref()?
        .get("headers")
        .map(|headers| {
            headers
                .as_map_ref()?
                .iter()
                .map(|(key, value)| {
                    let key = key.to_owned().to_string();
                    let value = value.to_owned().to_string();
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>, Box<EvalAltResult>>>()
        })
        .transpose()?
        .unwrap_or_default();
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        header_map.insert(
            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(&value).unwrap(),
        );
    }
    Ok((url, header_map))
}

pub async fn http_get(options: Dynamic) -> Result<String, Box<EvalAltResult>> {
    let (url, headers) = get_url_and_headers(&options)?;
    let client = reqwest::Client::new();
    client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("Failed to get URL: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?
        .text()
        .await
        .map_err(|e| {
            let msg = format!("Failed to parse response body: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })
}

pub async fn http_post(options: Dynamic) -> Result<String, Box<EvalAltResult>> {
    let (url, headers) = get_url_and_headers(&options)?;
    let body = options
        .as_map_ref()?
        .get("body")
        .map(|body| body.to_owned().to_string())
        .unwrap_or_default();
    let client = reqwest::Client::new();
    client
        .post(url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("Failed to post to URL: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?
        .text()
        .await
        .map_err(|e| {
            let msg = format!("Failed to parse response body: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })
}

pub async fn http_head(options: Dynamic) -> Result<(), Box<EvalAltResult>> {
    let (url, headers) = get_url_and_headers(&options)?;
    let client = reqwest::Client::new();
    client
        .head(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("Failed to head URL: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })
        .map(|_| ())
}
