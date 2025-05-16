use crate::types::{Config, Route};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Regex for matching wildcard hostnames
static WILDCARD_HOST_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\*\.(.+)$").expect("Failed to compile wildcard host regex"));

/// RouteMatcher handles matching incoming requests to configured routes
pub struct RouteMatcher {
    config: Arc<RwLock<Config>>,
}

impl RouteMatcher {
    /// Create a new RouteMatcher with the given configuration
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { config }
    }

    /// Match a request to a route based on host and path
    pub async fn match_route(&self, host: &str, path: &str) -> Option<Route> {
        let config = self.config.read().await;

        for route in &config.routes {
            if self.match_host(host, &route.host) && self.match_path(path, &route.path) {
                debug!("Matched route: host={}, path={}", route.host, route.path);
                return Some(route.clone());
            }
        }

        debug!("No matching route found for host={}, path={}", host, path);
        None
    }

    /// Match a host against a route host pattern
    fn match_host(&self, request_host: &str, route_host: &str) -> bool {
        // Exact match
        if request_host == route_host {
            trace!("Exact host match: {}", request_host);
            return true;
        }

        // Wildcard match (*.example.com)
        if let Some(captures) = WILDCARD_HOST_REGEX.captures(route_host) {
            if let Some(domain_suffix) = captures.get(1) {
                let domain_suffix = domain_suffix.as_str();
                if request_host.ends_with(domain_suffix) && request_host.len() > domain_suffix.len()
                {
                    let prefix = &request_host[0..request_host.len() - domain_suffix.len()];
                    if prefix.ends_with('.') {
                        trace!(
                            "Wildcard host match: {} matches pattern {}",
                            request_host,
                            route_host
                        );
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Match a path against a route path pattern
    fn match_path(&self, request_path: &str, route_path: &str) -> bool {
        // Exact match
        if request_path == route_path {
            trace!("Exact path match: {}", request_path);
            return true;
        }

        // Prefix match
        if route_path.ends_with('*') {
            let prefix = &route_path[0..route_path.len() - 1];
            if request_path.starts_with(prefix) {
                trace!(
                    "Prefix path match: {} matches pattern {}",
                    request_path,
                    route_path
                );
                return true;
            }
        }

        false
    }
}
