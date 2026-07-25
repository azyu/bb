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
        self.get_values(path, query, None)
    }

    pub fn get_values_up_to(
        &self,
        path: &str,
        query: &[(String, String)],
        limit: usize,
    ) -> Result<Vec<Value>, CliError> {
        self.get_values(path, query, Some(limit))
    }

    fn get_values(
        &self,
        path: &str,
        query: &[(String, String)],
        limit: Option<usize>,
    ) -> Result<Vec<Value>, CliError> {
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
            values.extend(page.values.into_iter().take(remaining));
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
}
