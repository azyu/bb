use std::fs;
use std::io::{Read, Write};

use reqwest::Method;
use serde_json::Value;

use crate::ApiRequest;
use crate::client::ApiResponse;
use crate::error::CliError;
use crate::render;

use super::support::{client_from_profile, collect_query, optional_trimmed};

pub(super) fn handle_api<R: Read, O: Write>(
    request: &ApiRequest,
    stdin: &mut R,
    stdout: &mut O,
) -> Result<(), CliError> {
    validate_api_request_options(request)?;
    let endpoint = request
        .endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError::InvalidInput("usage: bb api [flags] <endpoint>".to_string()))?;
    let client = client_from_profile(request.profile.as_deref())?;
    let query = collect_query([
        ("q", request.q.as_deref()),
        ("sort", request.sort.as_deref()),
        ("fields", request.fields.as_deref()),
    ]);

    if request.paginate {
        let values = client.get_all_values(endpoint, &query)?;
        return render::print_json(stdout, &values);
    }

    let method = request.method.trim().to_uppercase();
    let body = read_api_input_body_from(request.input.as_deref(), stdin)?;
    let response = client.request_api(
        Method::from_bytes(method.as_bytes())
            .map_err(|error| CliError::InvalidInput(format!("invalid HTTP method: {error}")))?,
        endpoint,
        &query,
        body,
        stdout,
    )?;
    match response {
        ApiResponse::Json(value) => render::print_json(stdout, &value),
        ApiResponse::Raw => Ok(()),
    }
}
fn read_api_input_body_from<R: Read>(
    input: Option<&str>,
    stdin: &mut R,
) -> Result<Option<Value>, CliError> {
    let Some(input) = optional_trimmed(input) else {
        return Ok(None);
    };

    let raw = if input == "-" {
        let mut raw = String::new();
        stdin.read_to_string(&mut raw)?;
        raw
    } else {
        fs::read_to_string(input).map_err(|error| CliError::Io(format!("read --input: {error}")))?
    };

    let body = serde_json::from_str(&raw)
        .map_err(|error| CliError::InvalidInput(format!("invalid JSON body: {error}")))?;
    Ok(Some(body))
}

fn validate_api_request_options(request: &ApiRequest) -> Result<(), CliError> {
    if request.paginate && optional_trimmed(request.input.as_deref()).is_some() {
        return Err(CliError::InvalidInput(
            "--paginate cannot be combined with --input".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde_json::json;

    use super::*;

    #[test]
    fn read_api_input_body_from_stdin_parses_json() {
        let mut stdin = Cursor::new(b"{\"content\":{\"raw\":\"stdin body\"}}\n");

        let body =
            read_api_input_body_from(Some("-"), &mut stdin).expect("stdin body should parse");

        assert_eq!(
            body,
            Some(json!({
                "content": {
                    "raw": "stdin body"
                }
            }))
        );
    }

    #[test]
    fn validate_api_request_options_rejects_paginate_with_input() {
        let request = ApiRequest {
            method: "POST".to_string(),
            input: Some("body.json".to_string()),
            paginate: true,
            profile: None,
            q: None,
            sort: None,
            fields: None,
            endpoint: Some("repositories/acme/widgets/pullrequests/42/comments".to_string()),
        };

        let error = validate_api_request_options(&request).expect_err("request should be rejected");

        assert_eq!(error.code(), "invalid_input");
        assert_eq!(
            error.message(),
            "--paginate cannot be combined with --input"
        );
    }
}
