use std::collections::HashMap;
use std::future::Future;

use ascot_library::device::DeviceEnvironment;
use ascot_library::hazards::Hazards;
use ascot_library::parameters::ParametersData;
use ascot_library::response::ResponseKind;
use ascot_library::route::{RestKind, RouteConfig, RouteConfigs};

use tracing::warn;

use crate::error::Error;
use crate::parameters::{convert_to_parameter_value, Parameters};
use crate::response::{
    InfoResponseParser, OkResponseParser, Response, SerialResponseParser, StreamResponse,
};

fn slash_end(s: &str) -> &str {
    if s.len() > 1 && s.ends_with('/') {
        &s[..s.len() - 1]
    } else {
        s
    }
}

fn slash_start(s: &str) -> &str {
    if s.len() > 1 && s.starts_with('/') {
        &s[1..]
    } else {
        s
    }
}

fn slash_start_end(s: &str) -> &str {
    slash_start(slash_end(s))
}

#[derive(Debug, PartialEq)]
struct RequestData {
    request: String,
    parameters: HashMap<String, String>,
}

impl RequestData {
    const fn new(request: String, parameters: HashMap<String, String>) -> Self {
        Self {
            request,
            parameters,
        }
    }
}

pub(crate) fn create_requests_senders(
    route_configs: RouteConfigs,
    complete_address: &str,
    main_route: &str,
    environment: DeviceEnvironment,
) -> HashMap<String, RequestSender> {
    route_configs
        .into_iter()
        .map(|route| {
            (
                route.data.name.to_string(),
                RequestSender::new(complete_address, main_route, environment, route),
            )
        })
        .collect()
}

/// Request sender.
///
/// It sends a request to a device.
///
/// The request can be formed by the chaining of different parameters or a
/// plain one.
#[derive(Debug, PartialEq)]
pub struct RequestSender {
    pub(crate) kind: RestKind,
    pub(crate) hazards: Hazards,
    pub(crate) route: String,
    pub(crate) parameters_data: ParametersData,
    pub(crate) response_kind: ResponseKind,
    pub(crate) device_environment: DeviceEnvironment,
}

impl RequestSender {
    pub(crate) fn new(
        address: &str,
        main_route: &str,
        device_environment: DeviceEnvironment,
        route_config: RouteConfig,
    ) -> Self {
        let kind = route_config.rest_kind;
        let route = format!(
            "{}/{}/{}",
            slash_end(address),
            slash_start_end(main_route),
            slash_start_end(&route_config.data.name)
        );
        let hazards = route_config.data.hazards;
        let parameters_data = route_config.data.parameters;
        let response_kind = route_config.response_kind;

        Self {
            kind,
            hazards,
            route,
            parameters_data,
            response_kind,
            device_environment,
        }
    }

    /// Sends a request to a device, getting in return a [`Response`].
    ///
    /// # Errors
    ///
    /// While sending a request to a device, some network failures or timeouts
    /// can prevent the effective sending. Moreover, the same issues can also
    /// affect the returned response.
    pub async fn send(&self) -> Result<Response, Error> {
        self.retrieve_response(|| async { self.plain_send().await })
            .await
    }

    /// Sends a request to a device with the given [`Parameters`], getting in
    /// return a [`Response`].
    ///
    /// # Errors
    ///
    /// While sending a request to a device, some network failures or timeouts
    /// can prevent the effective sending. Moreover, the same issues can also
    /// affect the returned response.
    pub async fn send_with_parameters(&self, parameters: Parameters) -> Result<Response, Error> {
        if self.parameters_data.is_empty() {
            warn!("The request does not have input parameters.");
            return self.send().await;
        }

        self.retrieve_response(|| async { self.create_response(&parameters).await })
            .await
    }

    /// Returns a reference to request [`Hazards`].
    #[must_use]
    pub fn hazards(&self) -> &Hazards {
        &self.hazards
    }

    /// Returns request [`RestKind`].
    #[must_use]
    pub fn kind(&self) -> RestKind {
        self.kind
    }

    /// Returns [`ParametersData`] associated with the request.
    ///
    /// If [`None`], the request does not contain any [`ParametersData`].
    #[must_use]
    pub fn params(&self) -> Option<&ParametersData> {
        self.parameters_data
            .is_empty()
            .then_some(&self.parameters_data)
    }

    async fn retrieve_response<F, Fut>(&self, retrieve_response: F) -> Result<Response, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<reqwest::Response, Error>>,
    {
        let response = retrieve_response().await?;

        Ok(match self.response_kind {
            ResponseKind::Ok => Response::OkBody(OkResponseParser::new(response)),
            ResponseKind::Serial => Response::SerialBody(SerialResponseParser::new(response)),
            ResponseKind::Info => Response::InfoBody(InfoResponseParser::new(response)),
            ResponseKind::Stream => Response::StreamBody(StreamResponse::new(response)),
        })
    }

    async fn plain_send(&self) -> Result<reqwest::Response, Error> {
        let request_data =
            self.request_data(|| self.axum_get_plain(), || self.create_params_plain());

        self.inputs_send(request_data).await
    }

    async fn create_response(&self, parameters: &Parameters) -> Result<reqwest::Response, Error> {
        let request_data = self.create_request(parameters)?;
        self.inputs_send(request_data).await
    }

    async fn inputs_send(&self, request_data: RequestData) -> Result<reqwest::Response, Error> {
        let RequestData {
            request,
            parameters,
        } = request_data;

        let client = reqwest::Client::new();

        Ok(match self.kind {
            RestKind::Get => client.get(request).send(),
            RestKind::Post => client.post(request).json(&parameters).send(),
            RestKind::Put => client.put(request).json(&parameters).send(),
            RestKind::Delete => client.delete(request).json(&parameters).send(),
        }
        .await?)
    }

    fn request_data<A, F>(&self, axum_get: A, params: F) -> RequestData
    where
        A: FnOnce() -> String,
        F: FnOnce() -> HashMap<String, String>,
    {
        let request =
            if self.kind == RestKind::Get && self.device_environment == DeviceEnvironment::Os {
                axum_get()
            } else {
                self.route.to_string()
            };

        let parameters = params();

        RequestData::new(request, parameters)
    }

    fn create_request(&self, parameters: &Parameters) -> Result<RequestData, Error> {
        // Check parameters.
        parameters.check_parameters(&self.parameters_data)?;

        Ok(self.request_data(
            || self.axum_get(parameters),
            || self.create_params(parameters),
        ))
    }

    fn axum_get_plain(&self) -> String {
        let mut route = self.route.to_string();
        for (_, parameter_kind) in &self.parameters_data {
            let Some(value) = convert_to_parameter_value(parameter_kind) else {
                // TODO: Skip bytes stream
                continue;
            };
            route.push_str(&format!("/{}", value.as_string()));
        }
        route
    }

    fn create_params_plain(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for (name, parameter_kind) in &self.parameters_data {
            let Some(value) = convert_to_parameter_value(parameter_kind) else {
                // TODO: Skip bytes stream
                continue;
            };
            params.insert(name.to_string(), value.as_string());
        }
        params
    }

    // Axum parameters: hello/{{1}}/{{2}}
    //                  hello/0.5/1
    fn axum_get(&self, parameters: &Parameters) -> String {
        let mut route = String::from(&self.route);
        for (name, parameter_kind) in &self.parameters_data {
            let value = if let Some(value) = parameters.get(name) {
                value.as_string()
            } else {
                let Some(value) = convert_to_parameter_value(parameter_kind) else {
                    // TODO: Skip bytes stream
                    continue;
                };
                value.as_string()
            };
            route.push_str(&format!("/{value}"));
        }

        route
    }

    fn create_params(&self, parameters: &Parameters) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for (name, parameter_kind) in &self.parameters_data {
            let (name, value) = if let Some(value) = parameters.get(name) {
                (name, value.as_string())
            } else {
                let Some(value) = convert_to_parameter_value(parameter_kind) else {
                    // TODO: Skip bytes stream
                    continue;
                };
                (name, value.as_string())
            };
            params.insert(name.to_string(), value);
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::time::Duration;

    use ascot_library::hazards::Hazard;
    use ascot_library::parameters::{
        ParameterKind, Parameters as LibraryParameters, ParametersData,
    };
    use ascot_library::response::{OkResponse, SerialResponse};
    use ascot_library::route::Route;

    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::json;

    use serial_test::serial;

    use crate::device::Device;
    use crate::parameters::{parameter_error, Parameters};
    use crate::response::Response;
    use crate::tests::{find_devices, light_with_toggle, Brightness};

    use super::{
        DeviceEnvironment, Error, Future, HashMap, Hazards, RequestData, RequestSender,
        ResponseKind, RestKind, RouteConfig,
    };

    const ADDRESS_ROUTE: &str = "http://ascot.local/";
    const ADDRESS_ROUTE_WITHOUT_SLASH: &str = "http://ascot.local/";
    const COMPLETE_ROUTE: &str = "http://ascot.local/light/route";

    fn plain_request(route: Route, kind: RestKind, hazards: Hazards) {
        let route = route.serialize_data();

        let request_sender =
            RequestSender::new(ADDRESS_ROUTE, "light/", DeviceEnvironment::Os, route);

        assert_eq!(
            request_sender,
            RequestSender {
                kind,
                hazards,
                route: COMPLETE_ROUTE.into(),
                parameters_data: ParametersData::new(),
                response_kind: ResponseKind::Ok,
                device_environment: DeviceEnvironment::Os,
            }
        );
    }

    fn request_with_inputs(route: Route, kind: RestKind, hazards: &Hazards) {
        let route = route
            .with_parameters(
                LibraryParameters::new()
                    .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
                    .rangef64("rangef64", (0., 20., 0.1)),
            )
            .serialize_data();

        let parameters_data = ParametersData::new()
            .insert(
                "rangeu64".into(),
                ParameterKind::RangeU64 {
                    min: 0,
                    max: 20,
                    step: 1,
                    default: 5,
                },
            )
            .insert(
                "rangef64".into(),
                ParameterKind::RangeF64 {
                    min: 0.,
                    max: 20.,
                    step: 0.1,
                    default: 0.,
                },
            );

        let request_sender =
            RequestSender::new(ADDRESS_ROUTE, "light/", DeviceEnvironment::Os, route);

        assert_eq!(
            request_sender,
            RequestSender {
                kind,
                hazards: hazards.clone(),
                route: COMPLETE_ROUTE.into(),
                parameters_data,
                response_kind: ResponseKind::Ok,
                device_environment: DeviceEnvironment::Os,
            }
        );

        // Non-existent value.
        assert_eq!(
            request_sender.create_request(&Parameters::new().u64("wrong", 0)),
            Err(parameter_error("`wrong` does not exist".into()))
        );

        // Wrong input type.
        assert_eq!(
            request_sender.create_request(&Parameters::new().f64("rangeu64", 0.)),
            Err(parameter_error("`rangeu64` must be of type `u64`".into()))
        );

        let mut parameters = HashMap::with_capacity(2);
        parameters.insert("rangeu64".into(), "3".into());
        parameters.insert("rangef64".into(), "0".into());

        assert_eq!(
            request_sender.create_request(&Parameters::new().u64("rangeu64", 3)),
            Ok(RequestData {
                request: if kind == RestKind::Get {
                    format!("{COMPLETE_ROUTE}/3/0")
                } else {
                    COMPLETE_ROUTE.into()
                },
                parameters,
            })
        );
    }

    fn request_generator(
        route: &str,
        main_route: &str,
        device_environment: DeviceEnvironment,
        route_config: RouteConfig,
    ) {
        assert_eq!(
            RequestSender::new(route, main_route, device_environment, route_config),
            RequestSender {
                kind: RestKind::Put,
                hazards: Hazards::new(),
                route: COMPLETE_ROUTE.into(),
                parameters_data: ParametersData::new(),
                response_kind: ResponseKind::Ok,
                device_environment: DeviceEnvironment::Os,
            }
        );
    }

    #[test]
    fn check_request_generator() {
        let route = Route::put("/route").serialize_data();
        let environment = DeviceEnvironment::Os;

        request_generator(ADDRESS_ROUTE, "light/", environment, route.clone());
        request_generator(ADDRESS_ROUTE_WITHOUT_SLASH, "light", environment, route);
    }

    #[test]
    fn create_plain_get_request() {
        let route = Route::get("/route").description("A GET route.");
        plain_request(route, RestKind::Get, Hazards::new());
    }

    #[test]
    fn create_plain_post_request() {
        let route = Route::post("/route").description("A POST route.");
        plain_request(route, RestKind::Post, Hazards::new());
    }

    #[test]
    fn create_plain_put_request() {
        let route = Route::put("/route").description("A PUT route.");
        plain_request(route, RestKind::Put, Hazards::new());
    }

    #[test]
    fn create_plain_delete_request() {
        let route = Route::delete("/route").description("A DELETE route.");
        plain_request(route, RestKind::Delete, Hazards::new());
    }

    #[test]
    fn create_plain_get_request_with_hazards() {
        let hazards = Hazards::new()
            .insert(Hazard::FireHazard)
            .insert(Hazard::AirPoisoning);
        plain_request(
            Route::get("/route")
                .description("A GET route.")
                .with_hazards(hazards.clone()),
            RestKind::Get,
            hazards,
        );
    }

    #[test]
    fn create_get_request_with_inputs() {
        request_with_inputs(
            Route::get("/route").description("A GET route."),
            RestKind::Get,
            &Hazards::new(),
        );
    }

    #[test]
    fn create_post_request_with_inputs() {
        let route = Route::post("/route").description("A POST route.");
        request_with_inputs(route, RestKind::Post, &Hazards::new());
    }

    #[test]
    fn create_put_request_with_inputs() {
        let route = Route::put("/route").description("A PUT route.");
        request_with_inputs(route, RestKind::Put, &Hazards::new());
    }

    #[test]
    fn create_delete_request_with_inputs() {
        let route = Route::delete("/route").description("A DELETE route.");
        request_with_inputs(route, RestKind::Delete, &Hazards::new());
    }

    #[test]
    fn create_get_request_with_hazards_and_inputs() {
        let hazards = Hazards::new()
            .insert(Hazard::FireHazard)
            .insert(Hazard::AirPoisoning);

        request_with_inputs(
            Route::get("/route")
                .description("A GET route.")
                .with_hazards(hazards.clone()),
            RestKind::Get,
            &hazards,
        );
    }

    async fn check_ok_response_plain(device: &Device, route: &str) {
        check_ok_response(device, route, |request| async { request.send().await }).await;
    }

    async fn check_ok_response_with_parameters(
        device: &Device,
        route: &str,
        parameters: Parameters,
    ) {
        check_ok_response(device, route, |request| async {
            request.send_with_parameters(parameters).await
        })
        .await;
    }

    async fn check_ok_response<'a, F, Fut>(device: &'a Device, route: &'a str, get_response: F)
    where
        F: FnOnce(&'a RequestSender) -> Fut,
        Fut: Future<Output = Result<Response, Error>>,
    {
        let request = device
            .request(route)
            .unwrap_or_else(|| panic!("No `{route}` request found."));

        let ok_response = get_response(request).await.unwrap();
        if let Response::OkBody(response) = ok_response {
            let ok_response = response.parse_body().await.unwrap();
            assert_eq!(ok_response, OkResponse::ok());
        }
    }

    async fn check_serial_response_plain<T: Serialize + DeserializeOwned + Debug + PartialEq>(
        device: &Device,
        route: &str,
        value: T,
    ) {
        check_serial_response(
            device,
            route,
            |request| async { request.send().await },
            value,
        )
        .await;
    }

    async fn check_serial_response_with_parameters<
        T: Serialize + DeserializeOwned + Debug + PartialEq,
    >(
        device: &Device,
        route: &str,
        parameters: Parameters,
        value: T,
    ) {
        check_serial_response(
            device,
            route,
            |request| async { request.send_with_parameters(parameters).await },
            value,
        )
        .await;
    }

    async fn check_serial_response<'a, F, Fut, T>(
        device: &'a Device,
        route: &'a str,
        get_response: F,
        value: T,
    ) where
        F: FnOnce(&'a RequestSender) -> Fut,
        Fut: Future<Output = Result<Response, Error>>,
        T: Serialize + DeserializeOwned + Debug + PartialEq,
    {
        let request = device
            .request(route)
            .unwrap_or_else(|| panic!("No `{route}` request found."));

        let serial_response = get_response(request).await.unwrap();
        if let Response::SerialBody(response) = serial_response {
            let serial_response = response.parse_body::<T>().await.unwrap();
            assert_eq!(serial_response, SerialResponse::new(value));
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial]
    async fn test_requests() {
        let _ = tracing_subscriber::fmt().try_init();

        let (close_tx, close_rx) = tokio::sync::oneshot::channel();

        // Run a device task.
        let device_handle = tokio::spawn(async { light_with_toggle(close_rx).await });

        // Wait for device task to be configured.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Discover device.
        let devices = find_devices().await;

        // Get device.
        let device = devices.get(0).expect("No device found.");

        // Run "/on" request and get "Ok" response.
        check_ok_response_plain(device, "/on").await;

        // Run "/off" request and get "Ok" response.
        check_ok_response_plain(device, "/off").await;

        // Run "/toggle" request and get "Ok" response.
        check_serial_response_plain(
            device,
            "/toggle",
            json!({
                "brightness": 0,
            }),
        )
        .await;

        // With parameters
        let parameters = Parameters::new().u64("brightness", 5);

        // Run "/on" request and get an "Ok" response with parameters.
        check_ok_response_with_parameters(device, "/on", parameters.clone()).await;

        // Run "/off" request and get an "Ok" response with parameters.
        check_ok_response_with_parameters(device, "/off", parameters.clone()).await;

        // Run "/toggle" request and get an "Ok" response with parameters.
        check_serial_response_with_parameters(
            device,
            "/toggle",
            parameters,
            Brightness { brightness: 5 },
        )
        .await;

        // Shutdown device server.
        _ = close_tx.send(());

        // Wait for device server to gracefully shutdown.
        _ = device_handle.await;
    }
}
