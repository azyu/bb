use std::io::{Seek, SeekFrom, Write};

use reqwest::Method;
use reqwest::blocking::Client as HttpClient;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::Profile;
use crate::error::CliError;

#[derive(Debug, Deserialize)]
struct ListPage {
    #[serde(default)]
    values: Vec<Value>,
    #[serde(default)]
    next: Option<String>,
    #[serde(default)]
    size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiResponse {
    Json(Value),
    Raw,
}

pub struct Client {
    base_url: String,
    token: String,
    username: String,
    http: HttpClient,
}

impl Client {
    pub fn from_profile(profile: &Profile) -> Result<Self, CliError> {
        if profile.token.trim().is_empty() {
            return Err(CliError::Config(
                "profile has no token configured".to_string(),
            ));
        }

        Ok(Self {
            base_url: profile.base_url.trim_end_matches('/').to_string(),
            token: profile.token.clone(),
            username: profile.username.trim().to_string(),
            http: HttpClient::builder()
                .user_agent("bb-cli/dev")
                .build()
                .map_err(|error| CliError::Internal(error.to_string()))?,
        })
    }

    pub fn get_page(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<(Vec<Value>, Option<u64>), CliError> {
        let page: ListPage = self.request_json(Method::GET, path, query, None::<Value>)?;
        Ok((page.values, page.size))
    }

    pub fn get_all_values(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<Vec<Value>, CliError> {
        self.get_values(path, query, None, |_, _| Ok(()))
    }

    pub fn get_all_values_with_progress<F>(
        &self,
        path: &str,
        query: &[(String, String)],
        progress: F,
    ) -> Result<Vec<Value>, CliError>
    where
        F: FnMut(usize, Option<u64>) -> Result<(), CliError>,
    {
        self.get_values(path, query, None, progress)
    }

    pub fn get_values_up_to(
        &self,
        path: &str,
        query: &[(String, String)],
        limit: usize,
    ) -> Result<Vec<Value>, CliError> {
        self.get_values(path, query, Some(limit), |_, _| Ok(()))
    }

    fn get_values<F>(
        &self,
        path: &str,
        query: &[(String, String)],
        limit: Option<usize>,
        mut progress: F,
    ) -> Result<Vec<Value>, CliError>
    where
        F: FnMut(usize, Option<u64>) -> Result<(), CliError>,
    {
        let mut next = path.to_string();
        let mut current_query = query.to_vec();
        let page_len = limit.unwrap_or(100).clamp(10, 100);
        current_query.push(("pagelen".to_string(), page_len.to_string()));
        let mut values = Vec::with_capacity(limit.unwrap_or(100).min(100));

        while !next.is_empty() && limit.is_none_or(|limit| values.len() < limit) {
            let page: ListPage =
                self.request_json(Method::GET, &next, &current_query, None::<Value>)?;
            let remaining = limit
                .map(|limit| limit.saturating_sub(values.len()))
                .unwrap_or(usize::MAX);
            let total = page.size;
            values.extend(page.values.into_iter().take(remaining));
            progress(values.len(), total)?;
            next = page.next.unwrap_or_default();
            current_query.clear();
        }

        Ok(values)
    }

    pub fn request_value(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<Value, CliError> {
        self.request_json(method, path, query, body)
    }

    pub fn request_api<W: Write>(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
        raw_out: &mut W,
    ) -> Result<ApiResponse, CliError> {
        let mut response = self.send_request(method, path, query, body)?;
        let is_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(is_json_content_type);

        if !is_json {
            // stdout must receive either the complete body or only the error envelope, never a
            // mix: errors are printed as JSON to the same stream. Spool to an anonymous temp file
            // so a mid-download failure never touches the sink, without buffering in memory.
            let mut spool = tempfile::tempfile().map_err(CliError::from)?;
            std::io::copy(&mut response, &mut spool).map_err(CliError::from)?;
            spool.seek(SeekFrom::Start(0)).map_err(CliError::from)?;
            std::io::copy(&mut spool, raw_out).map_err(CliError::from)?;
            return Ok(ApiResponse::Raw);
        }

        let bytes = response
            .bytes()
            .map_err(|error| CliError::Internal(format!("decode response: {error}")))?;
        if bytes.is_empty() {
            return Ok(ApiResponse::Raw);
        }
        let value = serde_json::from_slice(&bytes)
            .map_err(|error| CliError::Internal(format!("decode response: {error}")))?;
        Ok(ApiResponse::Json(value))
    }

    pub fn request_text(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
    ) -> Result<String, CliError> {
        let response = self.send_request(method, path, query, None)?;
        response
            .text()
            .map_err(|error| CliError::Internal(format!("decode response: {error}")))
    }

    pub fn request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<T, CliError> {
        let response = self.send_request(method, path, query, body)?;
        response
            .json::<T>()
            .map_err(|error| CliError::Internal(format!("decode response: {error}")))
    }

    fn send_request(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<reqwest::blocking::Response, CliError> {
        let mut url = if path.starts_with("http://") || path.starts_with("https://") {
            reqwest::Url::parse(path).map_err(|error| CliError::InvalidInput(error.to_string()))?
        } else {
            let normalized = if path.starts_with('/') {
                format!("{}{}", self.base_url, path)
            } else {
                format!("{}/{}", self.base_url, path)
            };
            reqwest::Url::parse(&normalized)
                .map_err(|error| CliError::InvalidInput(error.to_string()))?
        };

        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }

        let mut request = self.http.request(method, url);
        if self.username.is_empty() {
            request = request.bearer_auth(&self.token);
        } else {
            request = request.basic_auth(&self.username, Some(&self.token));
        }
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().map_err(|error| CliError::Api {
            status: 0,
            body: error.to_string(),
        })?;
        let status = response.status().as_u16();
        if status >= 400 {
            let body = response.text().unwrap_or_default();
            return Err(CliError::Api { status, body });
        }

        Ok(response)
    }
}

fn is_json_content_type(header: &str) -> bool {
    let mime = header.split(';').next().unwrap_or_default().trim();
    let Some((_, subtype)) = mime.split_once('/') else {
        return false;
    };
    subtype.eq_ignore_ascii_case("json")
        || subtype
            .rsplit_once('+')
            .is_some_and(|(_, suffix)| suffix.eq_ignore_ascii_case("json"))
}

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use serde_json::json;

    use super::*;

    #[test]
    fn get_all_values_follows_next_link() {
        let server = MockServer::start();
        let page2 = server.mock(|when, then| {
            when.method(GET).path("/page2");
            then.json_body(json!({"values":[{"slug":"two"}]}));
        });
        let page1 = server.mock(|when, then| {
            when.method(GET).path("/repositories/acme");
            then.json_body(json!({
                "values":[{"slug":"one"}],
                "next": format!("{}/page2", server.base_url())
            }));
        });

        let client = Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap();

        let values = client.get_all_values("/repositories/acme", &[]).unwrap();
        assert_eq!(values.len(), 2);
        page1.assert();
        page2.assert();
    }

    #[test]
    fn get_all_values_reports_each_completed_page() {
        let server = MockServer::start();
        let page2 = server.mock(|when, then| {
            when.method(GET).path("/page2");
            then.json_body(json!({"values":[{"slug":"two"}],"size":2}));
        });
        let page1 = server.mock(|when, then| {
            when.method(GET).path("/repositories/acme");
            then.json_body(json!({
                "values":[{"slug":"one"}],
                "size":2,
                "next": format!("{}/page2", server.base_url())
            }));
        });
        let client = Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap();
        let mut updates = Vec::new();

        let values = client
            .get_all_values_with_progress("/repositories/acme", &[], |fetched, total| {
                updates.push((fetched, total));
                Ok(())
            })
            .unwrap();

        assert_eq!(values.len(), 2);
        assert_eq!(updates, vec![(1, Some(2)), (2, Some(2))]);
        page1.assert();
        page2.assert();
    }

    #[test]
    fn get_values_up_to_uses_bitbucket_page_bounds_and_stops_at_limit() {
        let server = MockServer::start();
        let page2 = server.mock(|when, then| {
            when.method(GET).path("/page2");
            then.json_body(json!({"values":[{"slug":"three"}]}));
        });
        let page1 = server.mock(|when, then| {
            when.method(GET)
                .path("/repositories/acme")
                .query_param("pagelen", "10");
            then.json_body(json!({
                "values":[{"slug":"one"},{"slug":"two"}],
                "next": format!("{}/page2", server.base_url())
            }));
        });

        let client = Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap();

        let values = client
            .get_values_up_to("/repositories/acme", &[], 1)
            .unwrap();
        assert_eq!(values, vec![json!({"slug":"one"})]);
        page1.assert();
        assert_eq!(page2.hits(), 0);
    }

    #[test]
    fn get_values_up_to_follows_next_and_caps_first_page_at_bitbucket_maximum() {
        let server = MockServer::start();
        let first_values = (0..100).map(|id| json!({"id": id})).collect::<Vec<_>>();
        let second_values = (100..110).map(|id| json!({"id": id})).collect::<Vec<_>>();
        let page2 = server.mock(|when, then| {
            when.method(GET).path("/page2");
            then.json_body(json!({"values": second_values}));
        });
        let page1 = server.mock(|when, then| {
            when.method(GET)
                .path("/repositories/acme")
                .query_param("pagelen", "100");
            then.json_body(json!({
                "values": first_values,
                "next": format!("{}/page2", server.base_url())
            }));
        });

        let client = Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap();

        let values = client
            .get_values_up_to("/repositories/acme", &[], 105)
            .unwrap();
        assert_eq!(values.len(), 105);
        assert_eq!(values.last(), Some(&json!({"id": 104})));
        page1.assert();
        page2.assert();
    }

    #[test]
    fn request_text_reads_plain_text_response() {
        let server = MockServer::start();
        let diff = server.mock(|when, then| {
            when.method(GET).path("/diff");
            then.body("diff --git a/file b/file\n");
        });

        let client = Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap();

        let body = client.request_text(Method::GET, "/diff", &[]).unwrap();
        assert_eq!(body, "diff --git a/file b/file\n");
        diff.assert();
    }

    fn api_client(server: &MockServer) -> Client {
        Client::from_profile(&Profile {
            base_url: server.base_url(),
            token: "token".to_string(),
            username: String::new(),
        })
        .unwrap()
    }

    #[test]
    fn request_api_returns_raw_body_for_plain_text_response() {
        let server = MockServer::start();
        let log = server.mock(|when, then| {
            when.method(GET).path("/log");
            then.header("content-type", "text/plain")
                .body("+ npm test\nfailed\n");
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/log", &[], None, &mut sink)
            .expect("plain text response should succeed");

        assert_eq!(response, ApiResponse::Raw);
        assert_eq!(sink, b"+ npm test\nfailed\n");
        log.assert();
    }

    #[test]
    fn request_api_returns_raw_body_for_non_utf8_response() {
        let server = MockServer::start();
        let bytes: &[u8] = b"\x89PNG\r\n\x1a\n\xff\xfe";
        let avatar = server.mock(|when, then| {
            when.method(GET).path("/avatar");
            then.header("content-type", "image/png").body(bytes);
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/avatar", &[], None, &mut sink)
            .expect("binary response should succeed");

        assert_eq!(response, ApiResponse::Raw);
        assert_eq!(sink, bytes);
        avatar.assert();
    }

    #[test]
    fn request_api_parses_json_content_type_with_parameters() {
        let server = MockServer::start();
        let repo = server.mock(|when, then| {
            when.method(GET).path("/repo");
            then.header("content-type", "application/json; charset=utf-8")
                .json_body(json!({"slug": "widgets"}));
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/repo", &[], None, &mut sink)
            .expect("json response should succeed");

        assert_eq!(response, ApiResponse::Json(json!({"slug": "widgets"})));
        assert!(sink.is_empty());
        repo.assert();
    }

    #[test]
    fn request_api_parses_json_suffixed_content_type() {
        let server = MockServer::start();
        let repo = server.mock(|when, then| {
            when.method(GET).path("/repo");
            then.header("content-type", "application/hal+json")
                .json_body(json!({"slug": "widgets"}));
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/repo", &[], None, &mut sink)
            .expect("hal+json response should succeed");

        assert_eq!(response, ApiResponse::Json(json!({"slug": "widgets"})));
        assert!(sink.is_empty());
        repo.assert();
    }

    #[test]
    fn request_api_returns_raw_body_for_ndjson_content_type() {
        let server = MockServer::start();
        let stream = server.mock(|when, then| {
            when.method(GET).path("/stream");
            then.header("content-type", "application/x-ndjson")
                .body("{\"id\":1}\n{\"id\":2}\n");
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/stream", &[], None, &mut sink)
            .expect("ndjson response should succeed");

        assert_eq!(response, ApiResponse::Raw);
        assert_eq!(sink, b"{\"id\":1}\n{\"id\":2}\n");
        stream.assert();
    }

    #[test]
    fn request_api_returns_raw_body_when_content_type_is_missing() {
        let server = MockServer::start();
        let untyped = server.mock(|when, then| {
            when.method(GET).path("/untyped");
            then.body("{\"slug\":\"widgets\"}");
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/untyped", &[], None, &mut sink)
            .expect("untyped response should succeed");

        assert_eq!(response, ApiResponse::Raw);
        assert_eq!(sink, b"{\"slug\":\"widgets\"}");
        untyped.assert();
    }

    #[test]
    fn request_api_returns_empty_raw_body_for_empty_json_response() {
        let server = MockServer::start();
        let empty = server.mock(|when, then| {
            when.method(GET).path("/empty");
            then.header("content-type", "application/json").body("");
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let response = client
            .request_api(Method::GET, "/empty", &[], None, &mut sink)
            .expect("empty response should succeed");

        assert_eq!(response, ApiResponse::Raw);
        assert!(sink.is_empty());
        empty.assert();
    }

    #[test]
    fn request_api_rejects_malformed_json_body() {
        let server = MockServer::start();
        let broken = server.mock(|when, then| {
            when.method(GET).path("/broken");
            then.header("content-type", "application/json")
                .body("{not-json");
        });
        let client = api_client(&server);
        let mut sink: Vec<u8> = Vec::new();

        let error = client
            .request_api(Method::GET, "/broken", &[], None, &mut sink)
            .expect_err("malformed json should fail");

        assert_eq!(error.code(), "internal_error");
        assert!(
            error.message().starts_with("decode response"),
            "unexpected message: {}",
            error.message()
        );
        assert!(sink.is_empty());
        broken.assert();
    }
}
