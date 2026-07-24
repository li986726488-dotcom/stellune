#[derive(Clone)]
pub struct NarrativeServiceConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Clone)]
pub struct ServiceConfig {
    pub astrology_api_key: Option<String>,
    pub narrative: Option<NarrativeServiceConfig>,
}

impl ServiceConfig {
    pub fn load() -> Self {
        Self {
            astrology_api_key: value(
                "STELLUNE_ASTROLOGY_API_KEY",
                option_env!("STELLUNE_ASTROLOGY_API_KEY"),
            ),
            narrative: narrative_config(),
        }
    }
}

fn narrative_config() -> Option<NarrativeServiceConfig> {
    let api_key = value(
        "STELLUNE_NARRATIVE_API_KEY",
        option_env!("STELLUNE_NARRATIVE_API_KEY"),
    )?;
    let base_url = value(
        "STELLUNE_NARRATIVE_BASE_URL",
        option_env!("STELLUNE_NARRATIVE_BASE_URL"),
    )?;
    let model = value(
        "STELLUNE_NARRATIVE_MODEL",
        option_env!("STELLUNE_NARRATIVE_MODEL"),
    )
    .unwrap_or_else(|| "deepseek-chat".into());

    Some(NarrativeServiceConfig {
        api_key,
        base_url,
        model,
    })
}

fn value(name: &str, embedded: Option<&'static str>) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            embedded
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
        })
}
