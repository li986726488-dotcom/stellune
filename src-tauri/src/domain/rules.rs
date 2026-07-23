use chrono::{Datelike, NaiveDate, Weekday};

use crate::domain::models::{
    DailyReading, LuckyColor, LuckyHint, LuckyHints, PlanetPosition, Profile, ReadingBrief,
    ReadingHero, ReadingScores, ReadingSource, ReadingTrace, RuleImpact, ScoreDimensions,
    ZodiacSign,
};
use crate::error::AppError;

pub const RULES_VERSION: &str = "2026.07.2";
pub const FORTUNE_RULES_VERSION: &str = "2026.07.2";

const ZODIACS: [(&str, &str, &str); 12] = [
    ("aries", "白羊座", "♈"),
    ("taurus", "金牛座", "♉"),
    ("gemini", "双子座", "♊"),
    ("cancer", "巨蟹座", "♋"),
    ("leo", "狮子座", "♌"),
    ("virgo", "处女座", "♍"),
    ("libra", "天秤座", "♎"),
    ("scorpio", "天蝎座", "♏"),
    ("sagittarius", "射手座", "♐"),
    ("capricorn", "摩羯座", "♑"),
    ("aquarius", "水瓶座", "♒"),
    ("pisces", "双鱼座", "♓"),
];

pub fn zodiac_from_birthday(birthday: &str) -> Result<ZodiacSign, AppError> {
    let date = NaiveDate::parse_from_str(birthday, "%Y-%m-%d")
        .map_err(|_| AppError::validation("生日格式应为 YYYY-MM-DD。"))?;
    let marker = date.month() * 100 + date.day();
    let index = match marker {
        321..=419 => 0,
        420..=520 => 1,
        521..=621 => 2,
        622..=722 => 3,
        723..=822 => 4,
        823..=922 => 5,
        923..=1023 => 6,
        1024..=1122 => 7,
        1123..=1221 => 8,
        1222..=1231 | 101..=119 => 9,
        120..=218 => 10,
        219..=320 => 11,
        _ => return Err(AppError::validation("生日不是有效日期。")),
    };
    Ok(zodiac_by_index(index))
}

pub fn zodiac_from_slug(slug: &str) -> Option<ZodiacSign> {
    ZODIACS
        .iter()
        .position(|item| item.0 == slug)
        .map(zodiac_by_index)
}

pub fn weekday_cn(date: NaiveDate) -> String {
    match date.weekday() {
        Weekday::Mon => "星期一",
        Weekday::Tue => "星期二",
        Weekday::Wed => "星期三",
        Weekday::Thu => "星期四",
        Weekday::Fri => "星期五",
        Weekday::Sat => "星期六",
        Weekday::Sun => "星期日",
    }
    .to_string()
}

pub fn calculate_reading(
    profile: &Profile,
    positions: &[PlanetPosition],
    date: NaiveDate,
) -> (DailyReading, ReadingTrace) {
    let zodiac_index = zodiac_index(&profile.zodiac.slug);
    let reference_degree = zodiac_index as f64 * 30.0 + 15.0;
    let mut dimensions = ScoreDimensions {
        love: 75,
        work: 75,
        wealth: 75,
        social: 75,
        inner: 75,
    };
    let mut impacts = Vec::new();

    for position in positions {
        if let Some((aspect, orb)) = major_aspect(position.longitude, reference_degree) {
            let impact = aspect_impact(&position.planet, aspect, position.retrograde);
            if !is_zero(&impact) {
                apply_impact(&mut dimensions, &impact);
                impacts.push(RuleImpact {
                    id: format!(
                        "{}-{}-{}",
                        position.planet.to_lowercase(),
                        aspect,
                        profile.zodiac.slug
                    ),
                    planet: position.planet.clone(),
                    aspect: aspect.to_string(),
                    orb,
                    impact,
                });
            }
        }

        let relative_house = relative_house(&position.sign, zodiac_index);
        let impact = house_impact(&position.planet, relative_house);
        if !is_zero(&impact) {
            apply_impact(&mut dimensions, &impact);
            impacts.push(RuleImpact {
                id: format!(
                    "{}-house-{}",
                    position.planet.to_lowercase(),
                    relative_house
                ),
                planet: position.planet.clone(),
                aspect: format!("house-{relative_house}"),
                orb: 0.0,
                impact,
            });
        }
    }

    clamp_dimensions(&mut dimensions);
    let overall = ((dimensions.love as f64 * 0.20)
        + (dimensions.work as f64 * 0.25)
        + (dimensions.wealth as f64 * 0.15)
        + (dimensions.social as f64 * 0.20)
        + (dimensions.inner as f64 * 0.20))
        .round() as i32;

    let theme = choose_theme(&dimensions, &impacts);
    let impact_ids = impacts
        .iter()
        .map(|impact| impact.id.clone())
        .collect::<Vec<_>>();
    let hero = render_hero(theme);
    let briefs = render_briefs(&dimensions, &impact_ids);
    let echo = render_echo(theme).to_string();
    let lucky = calculate_lucky(positions, zodiac_index, date);
    let date_key = date.format("%Y-%m-%d").to_string();

    let reading = DailyReading {
        id: format!("reading-{date_key}-{}", profile.zodiac.slug),
        date: date_key,
        weekday: weekday_cn(date),
        zodiac: profile.zodiac.clone(),
        personalization_mode: profile.personalization_mode.clone(),
        hero,
        scores: ReadingScores {
            overall,
            dimensions,
        },
        briefs,
        echo,
        lucky,
        source: ReadingSource {
            provider: "AstrologyAPI".into(),
            rules_version: RULES_VERSION.into(),
            narrative_provider: "template".into(),
            stale: false,
        },
    };

    let trace = ReadingTrace {
        reference_degree,
        rules_version: RULES_VERSION.into(),
        impacts,
    };

    (reading, trace)
}

fn major_aspect(longitude: f64, reference: f64) -> Option<(&'static str, f64)> {
    let raw = (longitude - reference).abs();
    let distance = raw.min(360.0 - raw);
    [
        ("conjunction", 0.0),
        ("sextile", 60.0),
        ("square", 90.0),
        ("trine", 120.0),
        ("opposition", 180.0),
    ]
    .into_iter()
    .map(|(name, target)| (name, (distance - target).abs()))
    .filter(|(_, orb)| *orb <= 6.0)
    .min_by(|left, right| left.1.total_cmp(&right.1))
}

fn aspect_impact(planet: &str, aspect: &str, retrograde: bool) -> ScoreDimensions {
    let supportive = matches!(aspect, "trine" | "sextile");
    let challenging = matches!(aspect, "square" | "opposition");
    let mut impact = ScoreDimensions::default();

    match planet.to_ascii_lowercase().as_str() {
        "mars" if supportive => {
            impact.work = 8;
            impact.social = 7;
            impact.love = 2;
        }
        "mars" if challenging => {
            impact.work = -4;
            impact.inner = -3;
        }
        "mercury" if challenging => {
            impact.work = -5;
            impact.wealth = -2;
            impact.social = -3;
            impact.inner = -3;
        }
        "mercury" if supportive => {
            impact.work = 5;
            impact.social = 4;
        }
        "saturn" if challenging => {
            impact.love = -8;
            impact.social = -6;
            impact.inner = -5;
        }
        "jupiter" if supportive => {
            impact.work = 4;
            impact.wealth = 5;
            impact.social = 5;
        }
        "venus" if supportive => {
            impact.love = 5;
            impact.inner = 3;
        }
        "moon" if supportive => impact.inner = 4,
        "moon" if challenging => impact.inner = -3,
        "sun" if supportive => {
            impact.work = 3;
            impact.social = 4;
        }
        _ => {}
    }

    if retrograde && matches!(planet.to_ascii_lowercase().as_str(), "mercury") {
        impact.work -= 1;
        impact.inner -= 1;
    }
    impact
}

fn house_impact(planet: &str, house: i32) -> ScoreDimensions {
    let mut impact = ScoreDimensions::default();
    match (planet.to_ascii_lowercase().as_str(), house) {
        ("sun", 11) => {
            impact.work = 3;
            impact.social = 4;
        }
        ("jupiter", 11) => {
            impact.work = 3;
            impact.wealth = 5;
            impact.social = 6;
        }
        ("venus", 12) => {
            impact.love = 3;
            impact.wealth = -2;
            impact.inner = 6;
        }
        ("pluto", 5) => impact.love = 4,
        ("uranus", 9) => impact.work = 2,
        ("moon", 2) => impact.inner = 2,
        ("neptune", 7) => impact.inner = -2,
        _ => {}
    }
    impact
}

fn relative_house(sign: &str, zodiac_index: usize) -> i32 {
    let planet_index = english_sign_index(sign);
    ((planet_index + 12 - zodiac_index) % 12 + 1) as i32
}

fn apply_impact(target: &mut ScoreDimensions, impact: &ScoreDimensions) {
    target.love += impact.love;
    target.work += impact.work;
    target.wealth += impact.wealth;
    target.social += impact.social;
    target.inner += impact.inner;
}

fn clamp_dimensions(dimensions: &mut ScoreDimensions) {
    dimensions.love = dimensions.love.clamp(55, 92);
    dimensions.work = dimensions.work.clamp(55, 92);
    dimensions.wealth = dimensions.wealth.clamp(55, 92);
    dimensions.social = dimensions.social.clamp(55, 92);
    dimensions.inner = dimensions.inner.clamp(55, 92);
}

fn is_zero(value: &ScoreDimensions) -> bool {
    value.love == 0 && value.work == 0 && value.wealth == 0 && value.social == 0 && value.inner == 0
}

fn choose_theme(dimensions: &ScoreDimensions, impacts: &[RuleImpact]) -> &'static str {
    if impacts
        .iter()
        .any(|impact| impact.id.starts_with("saturn-opposition"))
    {
        return "校准";
    }
    if impacts
        .iter()
        .any(|impact| impact.id.starts_with("mercury-square"))
    {
        return "复核";
    }
    let scores = [
        ("柔软", dimensions.love),
        ("行动", dimensions.work),
        ("整理", dimensions.wealth),
        ("靠近", dimensions.social),
        ("沉淀", dimensions.inner),
    ];
    scores
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .map(|(theme, _)| theme)
        .unwrap_or("沉淀")
}

fn render_hero(theme: &str) -> ReadingHero {
    match theme {
        "校准" => ReadingHero {
            title: "把光留给真正值得回应的地方".into(),
            subtitle: "行动可以向前，答案不必急着说完。".into(),
            theme: theme.into(),
        },
        "复核" => ReadingHero {
            title: "慢一点确认，也是一种清醒".into(),
            subtitle: "今天适合把重要的话再听一遍。".into(),
            theme: theme.into(),
        },
        "行动" => ReadingHero {
            title: "你的方向，正在变得清晰".into(),
            subtitle: "先迈出一步，新的回应会随后出现。".into(),
            theme: theme.into(),
        },
        "靠近" => ReadingHero {
            title: "真诚正在缩短彼此的距离".into(),
            subtitle: "今天适合把温柔说得更具体一点。".into(),
            theme: theme.into(),
        },
        _ => ReadingHero {
            title: "你的星图，正在缓慢变亮".into(),
            subtitle: "今天适合先听见自己，再回应世界。".into(),
            theme: theme.into(),
        },
    }
}

fn render_briefs(dimensions: &ScoreDimensions, based_on: &[String]) -> Vec<ReadingBrief> {
    let ids = based_on.iter().take(3).cloned().collect::<Vec<_>>();
    vec![
        ReadingBrief {
            key: "relationship".into(),
            label: "关系".into(),
            text: if dimensions.love >= 80 {
                "真诚表达正在拉近距离，不必把温柔藏得太深。"
            } else {
                "清楚表达边界，比勉强维持和谐更靠近答案。"
            }
            .into(),
            based_on: ids.clone(),
        },
        ReadingBrief {
            key: "work".into(),
            label: "工作".into(),
            text: if dimensions.work >= 80 {
                "行动力正在回升，重要消息发出前再确认一次。"
            } else {
                "先完成最重要的一件事，再回应额外的声音。"
            }
            .into(),
            based_on: ids.clone(),
        },
        ReadingBrief {
            key: "self".into(),
            label: "自我".into(),
            text: if dimensions.inner >= 80 {
                "独处会带来清晰，让感受自然地沉淀下来。"
            } else {
                "给独处留一点空间，情绪会在安静里重新排好顺序。"
            }
            .into(),
            based_on: ids,
        },
    ]
}

fn render_echo(theme: &str) -> &'static str {
    match theme {
        "校准" => "慢一点回答，不代表你没有答案。有时，那是内心在替你筛选真正重要的声音。",
        "复核" => "有些事情值得再看一遍，不是因为犹豫，而是你正在靠近更真实的答案。",
        "行动" => "不用一次看清整段路，先走向眼前最明亮的那一步。",
        "靠近" => "真正的靠近，不需要完美，只需要让彼此看见诚实的部分。",
        _ => "那些暂时没有答案的事，也可以先交给夜色保管。",
    }
}

fn calculate_lucky(
    positions: &[PlanetPosition],
    zodiac_index: usize,
    date: NaiveDate,
) -> LuckyHints {
    let moon = positions
        .iter()
        .find(|position| position.planet.eq_ignore_ascii_case("moon"));
    let mars = positions
        .iter()
        .find(|position| position.planet.eq_ignore_ascii_case("mars"));

    let (color, hex) = match mars.map(|position| english_sign_index(&position.sign) % 4) {
        Some(0) => ("余晖金", "#D9B978"),
        Some(1) => ("晨雾蓝", "#9FB7D8"),
        Some(2) => ("雾紫", "#B4A0D9"),
        _ => ("月光银", "#C8CEDB"),
    };
    let moon_degree = moon
        .map(|position| position.degree_in_sign.floor() as u32)
        .unwrap_or(date.day());
    let number = ((moon_degree + zodiac_index as u32 + 1) % 9) + 1;
    let (time, time_basis) = calculate_lucky_time(moon, zodiac_index);

    LuckyHints {
        color: LuckyColor {
            value: color.into(),
            hex: hex.into(),
            basis: "主要行动行星与元素色板映射".into(),
            certainty: "entertainment-rule".into(),
        },
        number: LuckyHint {
            value: number.to_string(),
            basis: "月亮度数、星座序号与本地日期稳定计算".into(),
            certainty: "entertainment-rule".into(),
        },
        time: LuckyHint {
            value: time,
            basis: time_basis.into(),
            certainty: "entertainment-rule".into(),
        },
    }
}

fn calculate_lucky_time(
    moon: Option<&PlanetPosition>,
    zodiac_index: usize,
) -> (String, &'static str) {
    let Some(moon) = moon else {
        return ("全天平稳".into(), "缺少月亮数据，不指定单一幸运时段");
    };
    let reference_degree = zodiac_index as f64 * 30.0 + 15.0;
    let mut closest = (7, f64::INFINITY);

    for hour in 7..=21 {
        let midpoint_hour = hour as f64 + 1.0;
        let projected_moon =
            normalize_degree(moon.longitude + moon.speed * ((midpoint_hour - 12.0) / 24.0));
        let separation = angular_distance(projected_moon, reference_degree);
        let orb = [0.0, 60.0, 120.0]
            .into_iter()
            .map(|angle| (separation - angle).abs())
            .fold(f64::INFINITY, f64::min);
        if orb < closest.1 {
            closest = (hour, orb);
        }
    }

    if closest.1 <= 3.0 {
        (
            format!("{:02}:00—{:02}:00", closest.0, closest.0 + 2),
            "月亮日内运行接近太阳星座和谐相位的时段",
        )
    } else {
        ("全天平稳".into(), "今日月亮未在可用时段内接近精确和谐相位")
    }
}

fn normalize_degree(value: f64) -> f64 {
    value.rem_euclid(360.0)
}

fn angular_distance(first: f64, second: f64) -> f64 {
    let distance = (first - second).abs().rem_euclid(360.0);
    distance.min(360.0 - distance)
}

fn zodiac_by_index(index: usize) -> ZodiacSign {
    let item = ZODIACS[index % 12];
    ZodiacSign {
        slug: item.0.into(),
        name: item.1.into(),
        symbol: item.2.into(),
    }
}

fn zodiac_index(slug: &str) -> usize {
    ZODIACS.iter().position(|item| item.0 == slug).unwrap_or(6)
}

fn english_sign_index(sign: &str) -> usize {
    match sign.to_ascii_lowercase().as_str() {
        "aries" => 0,
        "taurus" => 1,
        "gemini" => 2,
        "cancer" => 3,
        "leo" => 4,
        "virgo" => 5,
        "libra" => 6,
        "scorpio" => 7,
        "sagittarius" => 8,
        "capricorn" => 9,
        "aquarius" => 10,
        "pisces" => 11,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_libra_birthday() {
        let zodiac = zodiac_from_birthday("1998-10-08").expect("valid date");
        assert_eq!(zodiac.slug, "libra");
    }

    #[test]
    fn detects_mars_trine_with_small_orb() {
        let aspect = major_aspect(77.082_214_226_974_13, 195.0).expect("aspect");
        assert_eq!(aspect.0, "trine");
        assert!((aspect.1 - 2.082_214_226_974_13).abs() < 0.000_001);
    }

    #[test]
    fn lucky_and_scores_are_deterministic() {
        let profile = Profile {
            id: "profile-local".into(),
            guest_id: "guest".into(),
            nickname: "星际旅人".into(),
            birthday: "1998-10-08".into(),
            birth_time: None,
            birth_city: None,
            zodiac: zodiac_from_birthday("1998-10-08").expect("zodiac"),
            personalization_mode: "sunSignLite".into(),
            completeness: 40,
            calculation_version: 1,
            created_at: "2026-07-23T00:00:00+08:00".into(),
            updated_at: "2026-07-23T00:00:00+08:00".into(),
        };
        let positions = vec![
            PlanetPosition {
                planet: "Mars".into(),
                longitude: 77.082_214_226_974_13,
                degree_in_sign: 17.082_214_226_974_13,
                sign: "Gemini".into(),
                speed: 0.68,
                retrograde: false,
            },
            PlanetPosition {
                planet: "Mercury".into(),
                longitude: 106.342_791_045_069_99,
                degree_in_sign: 16.342_791_045_069_987,
                sign: "Cancer".into(),
                speed: -0.06,
                retrograde: true,
            },
        ];
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("date");
        let (first, _) = calculate_reading(&profile, &positions, date);
        let (second, _) = calculate_reading(&profile, &positions, date);
        assert_eq!(first.scores.overall, second.scores.overall);
        assert_eq!(first.lucky.number.value, second.lucky.number.value);
    }

    #[test]
    fn lucky_time_can_select_a_daytime_window() {
        let moon = PlanetPosition {
            planet: "Moon".into(),
            longitude: 197.166_666_666_7,
            degree_in_sign: 17.166_666_666_7,
            sign: "Libra".into(),
            speed: 13.0,
            retrograde: false,
        };
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("date");

        let lucky = calculate_lucky(&[moon], 6, date);

        assert_eq!(lucky.time.value, "07:00—09:00");
        assert_eq!(lucky.time.basis, "月亮日内运行接近太阳星座和谐相位的时段");
    }

    #[test]
    fn lucky_time_does_not_force_a_window_without_a_close_aspect() {
        let moon = PlanetPosition {
            planet: "Moon".into(),
            longitude: 229.47,
            degree_in_sign: 19.47,
            sign: "Scorpio".into(),
            speed: 13.0,
            retrograde: false,
        };
        let date = NaiveDate::from_ymd_opt(2026, 7, 23).expect("date");

        let lucky = calculate_lucky(&[moon], 6, date);

        assert_eq!(lucky.time.value, "全天平稳");
        assert_eq!(lucky.time.basis, "今日月亮未在可用时段内接近精确和谐相位");
    }
}
