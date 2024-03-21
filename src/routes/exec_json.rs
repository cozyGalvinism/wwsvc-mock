use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::{AppConfig, OptionalJson};

use super::{ComResultBuilder, ServiceResponse, WebserviceRequest};

pub async fn exec_json(
    State(app_config): State<Arc<AppConfig>>,
    Json(request): Json<WebserviceRequest>,
) -> ServiceResponse<OptionalJson> {
    let resource = match request.lookup_resource(&app_config.mock_resources) {
        Some(resource) => resource,
        None => {
            let comresult = ComResultBuilder::with_status(StatusCode::BAD_REQUEST)
                .bereich("WWSVC")
                .code("400 Bad Request")
                .info("Es wurde eine fehlerhafte Anforderung übergeben.")
                .info2("Funktionsname nicht bekannt.")
                .info3(&request.function.function_name)
                .errno("20")
                .errnotxt("SVCERR_UNKNOWN_FUNCTION (20)")
                .build()
                .unwrap();
            return ServiceResponse::<OptionalJson> {
                comresult,
                body: OptionalJson(None),
            };
        }
    };

    let json = resource.data_source.as_json_value();
    let comresult = ComResultBuilder::with_status(StatusCode::OK)
        .bereich("WWSVC")
        .code("200 OK")
        .errno("0")
        .errnotxt("SVCERR_NO_ERROR (0)")
        .info("Kein Fehler")
        .info2("")
        .info3("")
        .build()
        .unwrap();

    ServiceResponse::<OptionalJson> {
        comresult,
        body: json,
    }
}
