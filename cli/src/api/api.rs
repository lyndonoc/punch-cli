use reqwest::{blocking, Result, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchAccessTokenPayload {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskInfoPayload {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APITaskInfo {
    pub id: i64,
    pub name: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APITaskStat {
    pub name: String,
    pub status: String,
    pub duration: i64,
}

pub fn fetch_access_token(api_endpoint: String, access_token: &String) -> String {
    let res = blocking::Client::new()
        .post(api_endpoint)
        .json(&FetchAccessTokenPayload {
            access_token: access_token.clone(),
        })
        .send()
        .expect("failed to fetch API access token");
    res.text()
        .expect("failed to parse API access token response")
}

pub fn verify_access_token(api_endpoint: String, access_token: &str) -> Result<bool> {
    let res = blocking::Client::new()
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .expect("request to verify the token failed");
    Ok(res.status() == StatusCode::NO_CONTENT)
}

pub fn start_task(
    api_endpoint: String,
    access_token: String,
    task_name: String,
) -> std::result::Result<APITaskInfo, String> {
    let res = blocking::Client::new()
        .post(api_endpoint)
        .json(&TaskInfoPayload { name: task_name })
        .header("Authorization", format!("Bearer {}", access_token))
        .send();
    match res {
        Ok(resp) => match resp.status() {
            StatusCode::OK => resp
                .json::<APITaskInfo>()
                .map_err(|e| format!("failed to parse the response: {}", e)),
            _ => match resp.text() {
                Ok(err_msg) => Err(err_msg),
                Err(err) => Err(format!("failed to parse the response: {}", err)),
            },
        },
        Err(err) => Err(err.to_string()),
    }
}

pub fn finish_task(
    api_endpoint: String,
    access_token: String,
    task_name: String,
) -> std::result::Result<APITaskInfo, String> {
    let res = blocking::Client::new()
        .post(api_endpoint)
        .json(&TaskInfoPayload { name: task_name })
        .header("Authorization", format!("Bearer {}", access_token))
        .send();
    match res {
        Ok(resp) => match resp.status() {
            StatusCode::OK => resp
                .json::<APITaskInfo>()
                .map_err(|e| format!("failed to parse the response: {}", e)),
            _ => match resp.text() {
                Ok(err_msg) => Err(err_msg),
                Err(err) => Err(format!("failed to parse the response: {}", err)),
            },
        },
        Err(err) => Err(err.to_string()),
    }
}

pub fn cancel_task(
    api_endpoint: String,
    access_token: String,
    task_name: String,
) -> std::result::Result<(), String> {
    let res = blocking::Client::new()
        .post(api_endpoint)
        .json(&TaskInfoPayload { name: task_name })
        .header("Authorization", format!("Bearer {}", access_token))
        .send();
    match res {
        Ok(resp) => match resp.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => match resp.text() {
                Ok(err_msg) => Err(err_msg),
                Err(err) => Err(format!("failed to parse the response: {}", err)),
            },
        },
        Err(err) => Err(err.to_string()),
    }
}

pub fn get_task(
    api_endpoint: String,
    access_token: String,
) -> std::result::Result<APITaskStat, String> {
    let res = blocking::Client::new()
        .get(api_endpoint)
        .header("Authorization", format!("Bearer {}", access_token))
        .send();
    match res {
        Ok(resp) => match resp.status() {
            StatusCode::OK => resp
                .json::<APITaskStat>()
                .map_err(|e| format!("failed to parse the response: {}", e)),
            _ => match resp.text() {
                Ok(err_msg) => Err(err_msg),
                Err(err) => Err(format!("failed to parse the response: {}", err)),
            },
        },
        Err(err) => Err(err.to_string()),
    }
}
