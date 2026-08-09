use std::io::Write;

use serde_json::Value;

use crate::client::Client;
use crate::config::{self, Profile};
use crate::error::CliError;
use crate::render;
use crate::{ListOutput, WriteOutput};

pub(super) fn client_from_profile(profile_name: Option<&str>) -> Result<Client, CliError> {
    let profile = profile_from_config(profile_name)?;
    Client::from_profile(&profile)
}

pub(super) fn profile_from_config(profile_name: Option<&str>) -> Result<Profile, CliError> {
    let config = config::load()?;
    let (profile, _) = config.active_profile(profile_name)?;
    if profile.token.trim().is_empty() {
        return Err(CliError::Config(
            "profile has no token configured".to_string(),
        ));
    }
    Ok(profile)
}
pub(super) fn parse_list_output(value: &str) -> Result<ListOutput, CliError> {
    match value.trim().to_lowercase().as_str() {
        "table" => Ok(ListOutput::Table),
        "json" => Ok(ListOutput::Json),
        other => Err(CliError::UnsupportedOutput(format!(
            "unsupported output format: {other}"
        ))),
    }
}

pub(super) fn parse_write_output(value: &str) -> Result<WriteOutput, CliError> {
    match value.trim().to_lowercase().as_str() {
        "text" => Ok(WriteOutput::Text),
        "json" => Ok(WriteOutput::Json),
        other => Err(CliError::UnsupportedOutput(format!(
            "unsupported output format: {other}"
        ))),
    }
}

pub(super) fn parse_json_fields(
    value: Option<&str>,
    output_is_json: bool,
    command: &str,
    allowed: &[&str],
) -> Result<Option<Vec<String>>, CliError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if !output_is_json {
        return Err(CliError::InvalidInput(
            "--json-fields requires --output json".to_string(),
        ));
    }

    let mut parsed = Vec::new();
    for field in value.split(',').map(str::trim) {
        if field.is_empty() {
            return Err(CliError::InvalidInput(
                "--json-fields requires a comma-separated field list".to_string(),
            ));
        }
        if !allowed.contains(&field) {
            return Err(CliError::InvalidInput(format!(
                "unknown --json-fields value for {command}: {field} (allowed: {})",
                allowed.join(", ")
            )));
        }
        if !parsed.iter().any(|existing| existing == field) {
            parsed.push(field.to_string());
        }
    }

    if parsed.is_empty() {
        return Err(CliError::InvalidInput(
            "--json-fields requires a comma-separated field list".to_string(),
        ));
    }

    Ok(Some(parsed))
}

pub(super) fn print_json_object<O: Write>(
    stdout: &mut O,
    value: &Value,
    fields: Option<&[String]>,
) -> Result<(), CliError> {
    if let Some(fields) = fields {
        return render::print_json(stdout, &render::project_json_object(value, fields));
    }
    render::print_json(stdout, value)
}

pub(super) fn print_json_list<O: Write>(
    stdout: &mut O,
    values: &[Value],
    fields: Option<&[String]>,
) -> Result<(), CliError> {
    if let Some(fields) = fields {
        return render::print_json(stdout, &render::project_json_list(values, fields));
    }
    render::print_json(stdout, values)
}
pub(super) fn collect_query<const N: usize>(
    pairs: [(&str, Option<&str>); N],
) -> Vec<(String, String)> {
    pairs
        .into_iter()
        .filter_map(|(key, value)| {
            optional_trimmed(value).map(|value| (key.to_string(), value.to_string()))
        })
        .collect()
}

pub(super) fn required_string<'a>(
    message: &str,
    value: Option<&'a str>,
) -> Result<&'a str, CliError> {
    optional_trimmed(value).ok_or_else(|| CliError::InvalidInput(message.to_string()))
}

pub(super) fn optional_trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

pub(super) fn set_optional_string(target: &mut Value, key: &str, value: Option<&str>) {
    if let Some(value) = optional_trimmed(value) {
        target[key] = Value::String(value.to_string());
    }
}
