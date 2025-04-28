use axum::{
    Json,
    response::{IntoResponse},
    http::StatusCode,
    http::HeaderMap,
};
use serde_json::Value;
use regex::Regex;
use reqwest::Client;
use tracing::{info, error, debug, trace}; // Import tracing macros


// Helper function to parse a simplified JSONPath string into segments
fn parse_json_path(path: &str) -> Result<Vec<String>, String> {
    trace!("Parsing JSON path: {}", path);
    let mut segments = Vec::new();
    let mut current_segment = String::new();
    let mut in_bracket = false;

    // Remove leading "$." if present
    let path = path.strip_prefix("$.").unwrap_or(path);

    for char in path.chars() {
        match char {
            '.' => {
                if !in_bracket && !current_segment.is_empty() {
                    segments.push(current_segment.clone());
                    current_segment.clear();
                } else if in_bracket {
                    current_segment.push(char);
                }
            }
            '[' => {
                if !current_segment.is_empty() {
                     segments.push(current_segment.clone());
                     current_segment.clear();
                }
                in_bracket = true;
                current_segment.push(char);
            }
            ']' => {
                if in_bracket {
                    current_segment.push(char);
                    segments.push(current_segment.clone());
                    current_segment.clear();
                    in_bracket = false;
                } else {
                    return Err(format!("Mismatched closing bracket in path: {}", path));
                }
            }
            _ => {
                current_segment.push(char);
            }
        }
    }

    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    // Basic validation for array wildcard
    for segment in &segments {
        if segment.starts_with("[") && segment.ends_with("]") && segment != "[*]" {
             return Err(format!("Unsupported array syntax: {}", segment));
        }
    }


    Ok(segments)
}

// Helper function to traverse and modify the Value based on parsed segments
fn traverse_and_modify(value: &mut Value, segments: &[String]) {
    if segments.is_empty() {
        // Reached the end of the path, try to parse and replace if it's a string
        if let Some(string_val) = value.as_str() {
            match serde_json::from_str(string_val) {
                Ok(parsed_json) => {
                    *value = parsed_json;
                }
                Err(e) => {
                    error!("Failed to parse JSON from string value: {}", e);
                }
            }
        }
        return;
    }

    let current_segment = &segments[0];
    let remaining_segments = &segments[1..];

    if current_segment == "[*]" {
        // Array wildcard, iterate over array elements
        if let Some(array) = value.as_array_mut() {
            for item in array {
                traverse_and_modify(item, remaining_segments);
            }
        } else {
            error!("Expected an array at path segment '[*]', but found something else.");
        }
    } else {
        // Object key
        // Remove brackets from key if present (e.g., "[key]" -> "key")
        let key = if current_segment.starts_with("[") && current_segment.ends_with("]") {
            current_segment[1..current_segment.len() - 1].to_string()
        } else {
            current_segment.clone()
        };

        if let Some(obj) = value.as_object_mut() {
            if let Some(field_value) = obj.get_mut(&key) {
                traverse_and_modify(field_value, remaining_segments);
            } else {
                error!("Key '{}' not found in object.", key);
            }
        } else {
             error!("Expected an object at path segment '{}', but found something else.", key);
        }
    }
}


#[axum::debug_handler]
pub async fn resend_handler(
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    info!("Received request in resend_handler");
    debug!("Request payload: {:?}", payload);

    // 1、检查location是否匹配环境参数中的可路由表中的所有正则；
    // This requires access to environment parameters/config, which is not available in the handler directly.
    // For now, I will add a placeholder check. A proper implementation would involve
    // passing the allowed routes/regexes via state or a global config.
    let location = payload["location"].as_str().unwrap_or_default();
    info!("Attempting to forward request to: {}", location);

    let allowed_routes: Vec<Regex> = vec![
        // Placeholder regex - replace with actual allowed routes
        Regex::new(r"^https://open\.feishu\.cn/open-apis/bitable/v1/apps/.*/tables/.*/records/batch_create").unwrap(),
    ];

    let is_allowed = allowed_routes.iter().any(|r| r.is_match(location));
    if !is_allowed {
        error!("Location not allowed: {}", location);
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Location not allowed"}))).into_response();
    }
    debug!("Location is allowed");

    // 2、根据命令对参数中的某个对象进行处理（例如json_parse从字符串转成合法的json对象）
    let mut mutable_payload = payload.clone(); // Clone the entire payload to modify

    let json_parse_paths: Vec<String> = if let Some(commands) = mutable_payload["commands"].as_object() {
        if let Some(json_parse_values) = commands.get("json_parse").and_then(|v| v.as_array()) {
            json_parse_values.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    for path in json_parse_paths {
        match parse_json_path(&path) {
            Ok(segments) => {
                debug!("Parsing and traversing JSON path: {}", path);
                traverse_and_modify(&mut mutable_payload, &segments);
            }
            Err(e) => {
                error!("Failed to parse JSON path {}: {}", path, e);
            }
        }
    }

    // 3、将所有headers处理后的params转发到location
    let client = Client::new();

    // Add headers from the incoming request, excluding host and content-length
    let mut reqwest_headers = reqwest::header::HeaderMap::new();
    for (key, value) in headers.iter() {
        if key.as_str().to_lowercase() != "host" && key.as_str().to_lowercase() != "content-length" {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_ref()) {
                    if let Ok(header_value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                    reqwest_headers.insert(header_name, header_value);
                }
            }
        }
    }

    // Add headers from the payload
    if let Some(payload_headers) = mutable_payload["headers"].as_object() {
        for (key, value) in payload_headers {
            if let Some(header_value_str) = value.as_str() {
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = reqwest::header::HeaderValue::from_str(header_value_str) {
                        reqwest_headers.insert(header_name, header_value);
                    }
                }
            }
        }
    }

    debug!("Forwarding with headers: {:?}", reqwest_headers);
    debug!("Forwarding with params: {:?}", mutable_payload["params"]);

    // Send the request
    let res = client.post(location).headers(reqwest_headers).json(&mutable_payload["params"]).send().await;

    // 4、获取发送的返回作为这个接口的返回返回
    match res {
        Ok(response) => {
            let status = axum::http::StatusCode::from_u16(response.status().as_u16()).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            info!("Forwarded request returned status: {}", status);
            let body = response.bytes().await.unwrap_or_default();
            (status, body).into_response()
        }
        Err(e) => {
            error!("Request forwarding failed: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("Request forwarding failed: {}", e)}))).into_response()
        }
    }
}
