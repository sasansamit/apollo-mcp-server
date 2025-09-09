//! Cross Origin Resource Sharing (CORS) module for Apollo MCP Server
//!
//! Provides CORS configuration and middleware for HTTP-based transports (StreamableHttp and SSE).
//!
//! # Default Behavior
//!
//! When CORS is not configured or disabled, no CORS headers are added.
//! When enabled with default settings:
//! - **Origins:** `["https://studio.apollographql.com"]`
//! - **Methods:** `["GET", "POST", "OPTIONS"]`
//! - **Allow credentials:** `false`
//!
//! # Configuration
//!
//! CORS can be configured at the transport level for HTTP-based transports.
//! The configuration supports:
//! - Specific allowed origins or wildcard (`allow_any_origin`)
//! - Allowed HTTP methods
//! - Allowed headers
//! - Exposed headers
//! - Credentials support
//! - Preflight cache duration (max_age)

use std::time::Duration;

use http::{HeaderName, HeaderValue, Method};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};

/// Cross origin request configuration.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CorsConfig {
    /// Enable CORS support
    pub enabled: bool,

    /// Set to true to allow any origin. Defaults to false.
    pub allow_any_origin: bool,

    /// Set to true to add the `Access-Control-Allow-Credentials` header.
    pub allow_credentials: bool,

    /// The headers to allow.
    /// If this value is not set, the server will mirror the client's `Access-Control-Request-Headers`.
    pub allow_headers: Vec<String>,

    /// Which response headers should be made available to scripts running in the browser.
    pub expose_headers: Vec<String>,

    /// Allowed request methods.
    pub methods: Vec<String>,

    /// The `Access-Control-Max-Age` header value in time units
    #[serde(deserialize_with = "humantime_serde::deserialize", default)]
    #[schemars(with = "Option<String>", default)]
    pub max_age: Option<Duration>,

    /// The origin(s) to allow requests from.
    pub origins: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_any_origin: false,
            allow_credentials: false,
            allow_headers: Vec::new(),
            expose_headers: Vec::new(),
            methods: default_methods(),
            max_age: None,
            origins: default_origins(),
        }
    }
}

fn default_origins() -> Vec<String> {
    vec!["http://localhost".into()]
}

fn default_methods() -> Vec<String> {
    vec!["GET".into(), "POST".into(), "OPTIONS".into()]
}

impl CorsConfig {
    /// Creates a new CorsLayer from this configuration
    pub fn into_layer(self) -> Result<CorsLayer, String> {
        // Validate the configuration first
        self.validate()?;

        let mut cors = CorsLayer::new();

        // Configure allowed origins
        if self.allow_any_origin {
            cors = cors.allow_origin(AllowOrigin::any());
        } else if !self.origins.is_empty() {
            let origins: Result<Vec<HeaderValue>, _> = self
                .origins
                .iter()
                .map(|origin| {
                    HeaderValue::from_str(origin).map_err(|_| {
                        format!(
                            "origin '{}' is not valid: failed to parse header value",
                            origin
                        )
                    })
                })
                .collect();
            cors = cors.allow_origin(origins?);
        }

        // Configure allowed methods
        if !self.methods.is_empty() {
            let methods: Result<Vec<Method>, _> = self
                .methods
                .iter()
                .map(|method| {
                    Method::from_bytes(method.as_bytes()).map_err(|_| {
                        format!("method '{}' is not valid: invalid HTTP method", method)
                    })
                })
                .collect();
            cors = cors.allow_methods(AllowMethods::list(methods?));
        }

        // Configure allowed headers
        if self.allow_headers.is_empty() {
            // Mirror client headers if none specified
            cors = cors.allow_headers(AllowHeaders::mirror_request());
        } else {
            let headers: Result<Vec<HeaderName>, _> = self
                .allow_headers
                .iter()
                .map(|header| {
                    HeaderName::from_bytes(header.as_bytes()).map_err(|_| {
                        format!(
                            "allow header name '{}' is not valid: invalid HTTP header name",
                            header
                        )
                    })
                })
                .collect();
            cors = cors.allow_headers(headers?);
        }

        // Configure exposed headers
        if !self.expose_headers.is_empty() {
            let headers: Result<Vec<HeaderName>, _> = self
                .expose_headers
                .iter()
                .map(|header| {
                    HeaderName::from_bytes(header.as_bytes()).map_err(|_| {
                        format!(
                            "expose header name '{}' is not valid: invalid HTTP header name",
                            header
                        )
                    })
                })
                .collect();
            cors = cors.expose_headers(ExposeHeaders::list(headers?));
        }

        // Configure credentials
        if self.allow_credentials {
            cors = cors.allow_credentials(true);
        }

        // Configure max age
        if let Some(max_age) = self.max_age {
            cors = cors.max_age(max_age);
        }

        Ok(cors)
    }

    /// Validates the CORS configuration according to the CORS specification
    fn validate(&self) -> Result<(), String> {
        // Check for wildcard origins (should use allow_any_origin instead)
        if self.origins.iter().any(|x| x == "*") {
            return Err(
                "Invalid CORS configuration: use `allow_any_origin: true` to set `Access-Control-Allow-Origin: *`".to_string(),
            );
        }

        // Validate that origins don't have trailing slashes (per CORS spec)
        for origin in &self.origins {
            if origin.ends_with('/') && origin != "/" {
                return Err(
                    "Invalid CORS configuration: origins cannot have trailing slashes (a serialized origin has no trailing slash)".to_string(),
                );
            }
        }

        // Validate credentials with wildcards
        if self.allow_credentials {
            if self.allow_headers.iter().any(|x| x == "*") {
                return Err(
                    "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
                        with wildcard in `allow_headers`".to_string(),
                );
            }

            if self.methods.iter().any(|x| x == "*") {
                return Err(
                    "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
                    with wildcard in `methods`".to_string(),
                );
            }

            if self.allow_any_origin {
                return Err(
                    "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
                    with `allow_any_origin: true`".to_string(),
                );
            }

            if self.expose_headers.iter().any(|x| x == "*") {
                return Err(
                    "Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` \
                        with wildcard in `expose_headers`".to_string(),
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CorsConfig::default();
        assert!(!config.enabled);
        assert!(!config.allow_any_origin);
        assert!(!config.allow_credentials);
        assert_eq!(config.origins, vec!["https://studio.apollographql.com"]);
        assert_eq!(config.methods, vec!["GET", "POST", "OPTIONS"]);
        assert!(config.allow_headers.is_empty());
        assert!(config.expose_headers.is_empty());
        assert!(config.max_age.is_none());
    }

    #[test]
    fn test_valid_configuration() {
        let config = CorsConfig {
            enabled: true,
            allow_any_origin: false,
            allow_credentials: false,
            allow_headers: vec!["content-type".into(), "authorization".into()],
            expose_headers: vec!["x-custom-header".into()],
            methods: vec!["GET".into(), "POST".into()],
            max_age: Some(Duration::from_secs(3600)),
            origins: vec!["https://example.com".into()],
        };

        let result = config.into_layer();
        assert!(result.is_ok());
    }

    #[test]
    fn test_wildcard_origin_rejected() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["*".into()],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("use `allow_any_origin: true`"));
    }

    #[test]
    fn test_trailing_slash_origin_rejected() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com/".into()],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("origins cannot have trailing slashes")
        );
    }

    #[test]
    fn test_credentials_with_wildcard_origin_rejected() {
        let config = CorsConfig {
            enabled: true,
            allow_any_origin: true,
            allow_credentials: true,
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot combine `Access-Control-Allow-Credentials: true`")
        );
    }

    #[test]
    fn test_credentials_with_wildcard_headers_rejected() {
        let config = CorsConfig {
            enabled: true,
            allow_credentials: true,
            allow_headers: vec!["*".into()],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot combine `Access-Control-Allow-Credentials: true`")
        );
    }

    #[test]
    fn test_credentials_with_wildcard_methods_rejected() {
        let config = CorsConfig {
            enabled: true,
            allow_credentials: true,
            methods: vec!["*".into()],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot combine `Access-Control-Allow-Credentials: true`")
        );
    }

    #[test]
    fn test_invalid_method_rejected() {
        let config = CorsConfig {
            enabled: true,
            methods: vec!["INVALID\nMETHOD".into()],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid HTTP method"));
    }

    #[test]
    fn test_invalid_header_rejected() {
        let config = CorsConfig {
            enabled: true,
            allow_headers: vec!["invalid\nheader".into()],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid HTTP header name"));
    }

    #[test]
    fn test_allow_any_origin() {
        let config = CorsConfig {
            enabled: true,
            allow_any_origin: true,
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_age_configuration() {
        let config = CorsConfig {
            enabled: true,
            max_age: Some(Duration::from_secs(7200)),
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_origins_with_defaults() {
        let config = CorsConfig {
            enabled: true,
            origins: vec![],
            ..Default::default()
        };

        let result = config.into_layer();
        assert!(result.is_ok());
    }
}
