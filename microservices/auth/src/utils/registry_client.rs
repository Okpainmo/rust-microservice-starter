use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub struct MeshRegistryClient {
    http: Client,
    mesh_url: String,
    mesh_token: Option<String>,
    advertised_host: String,
    service_name: String,
    service_version: String,
    service_port: u16,
    container_id: Option<String>,
    external_host: Option<String>,
    external_port: Option<u16>,
    external_scheme: String,
}

#[derive(Debug, Serialize)]
struct ServiceRegistrationRequest<'a> {
    service_name: &'a str,
    service_version: &'a str,
    service_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_host: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_scheme: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndpointDetails {
    pub ip: String,
    pub port: u16,
    pub internal_ip: String,
    pub internal_port: u16,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct MeshResponse<T> {
    response: Option<T>,
}

#[derive(Clone, Debug)]
pub struct MeshRegistryClientConfig {
    pub mesh_url: String,
    pub mesh_token: Option<String>,
    pub advertised_host: String,
    pub service_name: String,
    pub service_version: String,
    pub service_port: u16,
    pub container_id: Option<String>,
    pub external_host: Option<String>,
    pub external_port: Option<u16>,
    pub external_scheme: String,
}

impl MeshRegistryClient {
    pub fn new(config: MeshRegistryClientConfig) -> Self {
        Self {
            http: Client::new(),
            mesh_url: config.mesh_url.trim_end_matches('/').to_string(),
            mesh_token: clean_optional_string(config.mesh_token),
            advertised_host: config.advertised_host,
            service_name: config.service_name,
            service_version: config.service_version,
            service_port: config.service_port,
            container_id: clean_optional_string(config.container_id),
            external_host: clean_optional_string(config.external_host),
            external_port: config.external_port,
            external_scheme: config.external_scheme,
        }
    }

    pub async fn register(&self) -> Result<EndpointDetails> {
        self.send_registry_request(
            self.http
                .post(format!("{}/api/v1/mesh/services", self.mesh_url)),
            "registration",
        )
        .await
        .and_then(|response| response.context("mesh service registration response was empty"))
    }

    pub async fn heartbeat(&self) -> Result<()> {
        self.send_registry_request(
            self.http
                .post(format!("{}/api/v1/mesh/services/heartbeat", self.mesh_url)),
            "heartbeat",
        )
        .await
        .map(|_| ())
    }

    pub async fn unregister(&self) -> Result<()> {
        self.send_registry_request(
            self.http
                .delete(format!("{}/api/v1/mesh/services", self.mesh_url)),
            "unregistration",
        )
        .await
        .map(|_| ())
    }

    pub fn start_heartbeat(&self, heartbeat_interval_secs: u64) -> JoinHandle<()> {
        let client = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(heartbeat_interval_secs));

            loop {
                interval.tick().await;

                if let Err(error) = client.heartbeat().await {
                    tracing::error!(
                        service_name = %client.service_name,
                        service_version = %client.service_version,
                        error = %format!("{error:#}"),
                        "mesh heartbeat failed"
                    );
                }
            }
        })
    }

    async fn send_registry_request(
        &self,
        request: reqwest::RequestBuilder,
        action: &str,
    ) -> Result<Option<EndpointDetails>> {
        let mut request = self
            .authorized(request)
            .header("x-mesh-advertise-host", self.advertised_host.as_str());

        if let Some(container_id) = self.container_id.as_deref() {
            request = request.header("x-mesh-container-id", container_id);
        }

        let response = request
            .json(&self.registration_body())
            .send()
            .await
            .with_context(|| format!("failed to send service {action} request"))?
            .error_for_status()
            .with_context(|| format!("mesh rejected service {action} request"))?
            .json::<MeshResponse<EndpointDetails>>()
            .await
            .with_context(|| format!("failed to decode service {action} response"))?
            .response;

        Ok(response)
    }

    fn registration_body(&self) -> ServiceRegistrationRequest<'_> {
        let has_external_endpoint = self.external_host.is_some() && self.external_port.is_some();

        ServiceRegistrationRequest {
            service_name: &self.service_name,
            service_version: &self.service_version,
            service_port: self.service_port,
            external_host: if has_external_endpoint {
                self.external_host.as_deref()
            } else {
                None
            },
            external_port: if has_external_endpoint {
                self.external_port
            } else {
                None
            },
            external_scheme: if has_external_endpoint {
                Some(self.external_scheme.as_str())
            } else {
                None
            },
        }
    }

    fn authorized(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.mesh_token.as_deref() {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }
}

fn clean_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
