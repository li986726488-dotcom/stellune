use chrono::{Datelike, NaiveDate};
use reqwest::StatusCode;
use serde_json::{Value, json};

use crate::{domain::models::PlanetPosition, error::AppError, infrastructure::logger::AppLogger};

pub const ASTROLOGY_ENDPOINT: &str = "https://json.astrologyapi.com/v1/planets/tropical";

#[derive(Clone)]
pub struct AstrologyProvider {
    client: reqwest::Client,
    logger: AppLogger,
    api_key: Option<String>,
}

pub struct SkyResponse {
    pub request: Value,
    pub raw: Value,
    pub positions: Vec<PlanetPosition>,
}

impl AstrologyProvider {
    pub fn new(logger: AppLogger, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            logger,
            api_key,
        }
    }

    pub async fn fetch(
        &self,
        correlation_id: &str,
        date: NaiveDate,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> Result<SkyResponse, AppError> {
        let api_key = self.api_key.as_deref().ok_or_else(|| {
            AppError::astrology(
                "ASTROLOGY_API_KEY_MISSING",
                "尚未配置星相数据密钥，请设置 STELLUNE_ASTROLOGY_API_KEY。",
                false,
            )
        })?;
        let request = json!({
            "day": date.day(),
            "month": date.month(),
            "year": date.year(),
            "hour": 12,
            "min": 0,
            "lat": latitude,
            "lon": longitude,
            "tzone": timezone,
            "house_type": "placidus"
        });
        self.logger.external_request(
            correlation_id,
            "astrologyapi",
            ASTROLOGY_ENDPOINT,
            request.clone(),
        );

        let response = self
            .client
            .post(ASTROLOGY_ENDPOINT)
            .header("x-astrologyapi-key", api_key)
            .json(&request)
            .send()
            .await
            .map_err(|error| {
                self.logger.error(
                    "external.error",
                    correlation_id,
                    json!({
                        "provider": "astrologyapi",
                        "endpoint": ASTROLOGY_ENDPOINT,
                        "stage": "send",
                        "error": error.to_string(),
                    }),
                );
                AppError::astrology(
                    "ASTROLOGY_NETWORK_ERROR",
                    format!("真实天象暂时无法连接：{error}"),
                    true,
                )
            })?;
        let status = response.status();
        let raw_text = response.text().await.map_err(|error| {
            self.logger.error(
                "external.error",
                correlation_id,
                json!({
                    "provider": "astrologyapi",
                    "endpoint": ASTROLOGY_ENDPOINT,
                    "stage": "read-body",
                    "status": status.as_u16(),
                    "error": error.to_string(),
                }),
            );
            AppError::astrology(
                "ASTROLOGY_INVALID_RESPONSE",
                format!("星相数据格式无法读取：{error}"),
                true,
            )
        })?;
        let raw = serde_json::from_str::<Value>(&raw_text).map_err(|error| {
            self.logger.error(
                "external.invalid-response",
                correlation_id,
                json!({
                    "provider": "astrologyapi",
                    "endpoint": ASTROLOGY_ENDPOINT,
                    "status": status.as_u16(),
                    "rawBodyLength": raw_text.len(),
                    "error": error.to_string(),
                }),
            );
            AppError::astrology(
                "ASTROLOGY_INVALID_RESPONSE",
                format!("星相数据格式无法识别：{error}"),
                true,
            )
        })?;
        self.logger
            .external_response(correlation_id, "astrologyapi", status.as_u16(), raw.clone());

        if !status.is_success() {
            let message = raw
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("星相服务暂时不可用。");
            return Err(match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    AppError::astrology("ASTROLOGY_AUTH_ERROR", message, false)
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    AppError::astrology("ASTROLOGY_QUOTA_ERROR", message, true)
                }
                _ => AppError::astrology("ASTROLOGY_SERVICE_ERROR", message, true),
            });
        }

        let positions = normalize_positions(&raw)?;
        self.logger.info(
            "external.normalized",
            correlation_id,
            json!({
                "provider": "astrologyapi",
                "positions": positions,
            }),
        );
        Ok(SkyResponse {
            request,
            raw,
            positions,
        })
    }
}

fn normalize_positions(raw: &Value) -> Result<Vec<PlanetPosition>, AppError> {
    let source = raw
        .as_array()
        .or_else(|| raw.get("planets").and_then(Value::as_array))
        .or_else(|| raw.get("data").and_then(Value::as_array))
        .ok_or_else(|| {
            AppError::astrology(
                "ASTROLOGY_INVALID_RESPONSE",
                "星相服务未返回行星位置列表。",
                true,
            )
        })?;

    let positions = source
        .iter()
        .filter_map(|value| {
            let planet = value
                .get("name")
                .or_else(|| value.get("planet"))
                .and_then(Value::as_str)?
                .to_string();
            let longitude = number(value, &["fullDegree", "longitude", "full_degree"])?;
            let degree_in_sign = number(value, &["normDegree", "degree", "degree_in_sign"])
                .unwrap_or(longitude % 30.0);
            let sign = value
                .get("sign")
                .or_else(|| value.get("signName"))
                .and_then(Value::as_str)
                .unwrap_or("Aries")
                .to_string();
            let speed = number(value, &["speed"]).unwrap_or(0.0);
            let retrograde = value
                .get("isRetro")
                .or_else(|| value.get("retrograde"))
                .map(bool_value)
                .unwrap_or(speed < 0.0);
            Some(PlanetPosition {
                planet,
                longitude,
                degree_in_sign,
                sign,
                speed,
                retrograde,
            })
        })
        .collect::<Vec<_>>();

    if positions.is_empty() {
        return Err(AppError::astrology(
            "ASTROLOGY_INVALID_RESPONSE",
            "星相服务返回的行星数据为空。",
            true,
        ));
    }
    Ok(positions)
}

fn number(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|candidate| {
            candidate
                .as_f64()
                .or_else(|| candidate.as_str().and_then(|text| text.parse().ok()))
        })
    })
}

fn bool_value(value: &Value) -> bool {
    value.as_bool().unwrap_or_else(|| {
        matches!(
            value
                .as_str()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .as_str(),
            "true" | "yes" | "1" | "r"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_astrology_api_shape() {
        let raw = json!([{
            "name": "Mars",
            "fullDegree": 77.08,
            "normDegree": 17.08,
            "sign": "Gemini",
            "speed": 0.68,
            "isRetro": "false"
        }]);
        let result = normalize_positions(&raw).expect("valid response");
        assert_eq!(result[0].planet, "Mars");
        assert!(!result[0].retrograde);
    }
}
