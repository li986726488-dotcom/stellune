use crate::domain::{
    models::{CompatibilityFactor, CompatibilityReading, DailyReading, EmotionGuide},
    rules::zodiac_from_slug,
};

pub const COMPATIBILITY_RULES_VERSION: &str = "2026.07.1";
pub const EMOTION_RULES_VERSION: &str = "2026.07.1";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Element {
    Fire,
    Earth,
    Air,
    Water,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Modality {
    Cardinal,
    Fixed,
    Mutable,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Polarity {
    Yang,
    Yin,
}

#[derive(Clone, Copy)]
struct SignAttributes {
    index: usize,
    element: Element,
    modality: Modality,
    polarity: Polarity,
}

pub fn calculate_compatibility(
    primary_slug: &str,
    partner_slug: &str,
) -> Option<CompatibilityReading> {
    let primary_sign = zodiac_from_slug(primary_slug)?;
    let partner_sign = zodiac_from_slug(partner_slug)?;
    let primary = sign_attributes(primary_slug)?;
    let partner = sign_attributes(partner_slug)?;

    let (element_score, element_label) = element_score(primary.element, partner.element);
    let (modality_score, modality_label) = modality_score(primary.modality, partner.modality);
    let (polarity_score, polarity_label) = polarity_score(primary.polarity, partner.polarity);
    let distance = zodiac_distance(primary.index, partner.index);
    let (aspect_score, aspect_label) = aspect_score(distance);
    let score = (10 + element_score + modality_score + polarity_score + aspect_score).clamp(45, 95);

    Some(CompatibilityReading {
        mode: "sunSignCompatibility".into(),
        primary_sign,
        partner_sign,
        score,
        level: compatibility_level(score).into(),
        title: compatibility_title(primary.element, partner.element).into(),
        summary: format!(
            "{}{}{}",
            element_summary(primary.element, partner.element),
            modality_summary(primary.modality, partner.modality),
            aspect_summary(distance)
        ),
        factors: vec![
            CompatibilityFactor {
                key: "element".into(),
                label: element_label.into(),
                score: element_score,
            },
            CompatibilityFactor {
                key: "modality".into(),
                label: modality_label.into(),
                score: modality_score,
            },
            CompatibilityFactor {
                key: "polarity".into(),
                label: polarity_label.into(),
                score: polarity_score,
            },
            CompatibilityFactor {
                key: "aspect".into(),
                label: aspect_label.into(),
                score: aspect_score,
            },
        ],
        certainty: "sun-sign-entertainment".into(),
        rules_version: COMPATIBILITY_RULES_VERSION.into(),
    })
}

pub fn render_emotion_guide(mood: &str, reading: &DailyReading) -> Option<EmotionGuide> {
    let scores = &reading.scores.dimensions;
    let (need, title, action, mood_summary) = match mood {
        "开心" => (
            "分享与保留",
            if scores.social >= 80 {
                "把快乐分享给喜欢的人"
            } else {
                "把这一刻的明亮留久一点"
            },
            if scores.social >= 80 {
                "给一个想起时会微笑的人发一条消息。"
            } else {
                "记下一件今天让你觉得值得的小事。"
            },
            "允许这份好心情自然流动。",
        ),
        "平静" => (
            "维持节奏",
            if scores.inner >= 80 {
                "安静正在替你整理答案"
            } else {
                "守住此刻不被打扰的节奏"
            },
            "为自己保留十分钟不被消息打断的时间。",
            "此刻不需要额外证明什么。",
        ),
        "迷茫" => (
            "减少选择",
            if scores.work >= 80 {
                "先走一步，方向会慢慢出现"
            } else {
                "不用现在看清整段路"
            },
            "只决定眼前最小、最确定的一件事。",
            "迷茫不是没有方向，只是线索还没有排好顺序。",
        ),
        "疲惫" => (
            "休息与减负",
            if scores.work >= 80 {
                "有力量，也要允许自己停一停"
            } else {
                "今天先把自己放回优先级"
            },
            "取消或推迟一件今天并不必要的任务。",
            "疲惫是在提醒你收回分散出去的能量。",
        ),
        "低落" => (
            "安全与陪伴",
            if scores.inner < 65 {
                "先让自己安全地待一会儿"
            } else {
                "低落不是对今天的结论"
            },
            "先照顾身体，再把感受告诉一个可信任的人。",
            "不必急着把自己从情绪里拉出来。",
        ),
        _ => return None,
    };

    let score_summary = score_summary(mood, scores.love, scores.work, scores.social, scores.inner);
    let theme_summary = theme_summary(&reading.hero.theme);

    Some(EmotionGuide {
        mood: mood.into(),
        title: title.into(),
        summary: format!("{mood_summary}{score_summary}{theme_summary}"),
        action: action.into(),
        need: need.into(),
        based_on: vec![
            format!("mood:{mood}"),
            format!("theme:{}", reading.hero.theme),
            format!("love:{}", score_band(scores.love)),
            format!("work:{}", score_band(scores.work)),
            format!("social:{}", score_band(scores.social)),
            format!("inner:{}", score_band(scores.inner)),
        ],
        generator: "template".into(),
        certainty: "emotional-companion".into(),
        rules_version: EMOTION_RULES_VERSION.into(),
    })
}

fn sign_attributes(slug: &str) -> Option<SignAttributes> {
    let attributes = match slug {
        "aries" => (0, Element::Fire, Modality::Cardinal, Polarity::Yang),
        "taurus" => (1, Element::Earth, Modality::Fixed, Polarity::Yin),
        "gemini" => (2, Element::Air, Modality::Mutable, Polarity::Yang),
        "cancer" => (3, Element::Water, Modality::Cardinal, Polarity::Yin),
        "leo" => (4, Element::Fire, Modality::Fixed, Polarity::Yang),
        "virgo" => (5, Element::Earth, Modality::Mutable, Polarity::Yin),
        "libra" => (6, Element::Air, Modality::Cardinal, Polarity::Yang),
        "scorpio" => (7, Element::Water, Modality::Fixed, Polarity::Yin),
        "sagittarius" => (8, Element::Fire, Modality::Mutable, Polarity::Yang),
        "capricorn" => (9, Element::Earth, Modality::Cardinal, Polarity::Yin),
        "aquarius" => (10, Element::Air, Modality::Fixed, Polarity::Yang),
        "pisces" => (11, Element::Water, Modality::Mutable, Polarity::Yin),
        _ => return None,
    };
    Some(SignAttributes {
        index: attributes.0,
        element: attributes.1,
        modality: attributes.2,
        polarity: attributes.3,
    })
}

fn element_score(first: Element, second: Element) -> (i32, &'static str) {
    if first == second {
        return (30, "同元素共鸣");
    }
    if matches!(
        (first, second),
        (Element::Fire, Element::Air)
            | (Element::Air, Element::Fire)
            | (Element::Earth, Element::Water)
            | (Element::Water, Element::Earth)
    ) {
        (35, "元素彼此助长")
    } else {
        (18, "元素需要磨合")
    }
}

fn modality_score(first: Modality, second: Modality) -> (i32, &'static str) {
    if first != second {
        return (15, "行动节奏互补");
    }
    match first {
        Modality::Mutable => (12, "同为变动宫"),
        Modality::Cardinal => (9, "同为基本宫"),
        Modality::Fixed => (7, "同为固定宫"),
    }
}

fn polarity_score(first: Polarity, second: Polarity) -> (i32, &'static str) {
    if first == second {
        (10, "表达方向相近")
    } else {
        (6, "表达方向互补")
    }
}

fn zodiac_distance(first: usize, second: usize) -> usize {
    let raw = first.abs_diff(second);
    raw.min(12 - raw)
}

fn aspect_score(distance: usize) -> (i32, &'static str) {
    match distance {
        0 => (16, "太阳星座形成合相关系"),
        1 => (10, "太阳星座形成半六分关系"),
        2 => (18, "太阳星座形成六分关系"),
        3 => (8, "太阳星座形成四分关系"),
        4 => (20, "太阳星座形成三分关系"),
        5 => (6, "太阳星座形成梅花相关系"),
        _ => (14, "太阳星座形成对冲关系"),
    }
}

fn compatibility_level(score: i32) -> &'static str {
    match score {
        90..=95 => "高度共鸣",
        80..=89 => "默契靠近",
        70..=79 => "平衡互补",
        60..=69 => "需要理解",
        _ => "差异成长",
    }
}

fn compatibility_title(first: Element, second: Element) -> &'static str {
    match (first, second) {
        (Element::Air, Element::Air) => "风与风的默契",
        (Element::Fire, Element::Fire) => "火光照见火光",
        (Element::Earth, Element::Earth) => "并肩生长的根系",
        (Element::Water, Element::Water) => "两片潮汐的回声",
        (Element::Fire, Element::Air) | (Element::Air, Element::Fire) => "风让火焰更明亮",
        (Element::Earth, Element::Water) | (Element::Water, Element::Earth) => "雨水拥抱大地",
        _ => "差异里的吸引力",
    }
}

fn element_summary(first: Element, second: Element) -> &'static str {
    if first == second {
        return "你们很容易理解彼此表达世界的方式。";
    }
    if matches!(
        (first, second),
        (Element::Fire, Element::Air)
            | (Element::Air, Element::Fire)
            | (Element::Earth, Element::Water)
            | (Element::Water, Element::Earth)
    ) {
        "你们的差异能自然补充彼此需要的能量。"
    } else {
        "你们看待事情的方式并不相同，理解会比猜测更重要。"
    }
}

fn modality_summary(first: Modality, second: Modality) -> &'static str {
    if first != second {
        "一方带来方向，另一方补上不同的节奏。"
    } else {
        "相似的行动节奏带来默契，也需要为彼此留出转身的空间。"
    }
}

fn aspect_summary(distance: usize) -> &'static str {
    match distance {
        0 => "熟悉感很强，记得保留各自的独特性。",
        2 | 4 => "保持坦率交流，会让这份默契更轻盈。",
        6 => "吸引与张力同时存在，清楚表达边界会让关系更稳定。",
        3 | 5 => "把分歧说具体，会比追求完全一致更靠近答案。",
        _ => "多确认一次彼此的意思，能减少不必要的误会。",
    }
}

fn score_summary(mood: &str, love: i32, work: i32, social: i32, inner: i32) -> &'static str {
    if mood == "开心" && social >= 80 {
        "今天适合把好心情说给可信任的人听。"
    } else if matches!(mood, "疲惫" | "低落") && inner < 65 {
        "现在更需要降低刺激，让身体和情绪先找到安全感。"
    } else if mood == "迷茫" && work >= 80 {
        "你并不缺少行动力，只需要把方向缩小到眼前一步。"
    } else if love < 65 {
        "不必用迎合换取理解，先让自己的边界清楚一点。"
    } else if social >= 80 {
        "与可信任的人交流，会让感受变得更清晰。"
    } else {
        "给自己一点不被催促的空间，感受会慢慢排好顺序。"
    }
}

fn theme_summary(theme: &str) -> &'static str {
    match theme {
        "校准" => "把注意力留给真正重要的人和事。",
        "复核" => "重要回应可以慢一点，先确认信息也确认感受。",
        "行动" => "不用完成很多，做成一个小而明确的动作就好。",
        "靠近" => "把需要说得具体，关系会更容易接住你。",
        "柔软" => "允许感受存在，不必立刻把它解决。",
        "整理" => "减少一点杂乱，会帮助你重新找回掌控感。",
        _ => "先留一点空白，让情绪自然沉淀。",
    }
}

fn score_band(score: i32) -> &'static str {
    match score {
        ..=64 => "low",
        65..=79 => "medium",
        _ => "high",
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::domain::{
        models::{
            DailyReading, Profile, ReadingBrief, ReadingHero, ReadingScores, ReadingSource,
            ScoreDimensions,
        },
        rules::{calculate_reading, zodiac_from_birthday},
    };

    #[test]
    fn libra_and_gemini_have_explainable_compatibility() {
        let reading = calculate_compatibility("libra", "gemini").expect("known signs");

        assert_eq!(reading.score, 85);
        assert_eq!(reading.level, "默契靠近");
        assert_eq!(reading.title, "风与风的默契");
        assert_eq!(reading.factors.len(), 4);
        assert_eq!(reading.factors[0].score, 30);
        assert_eq!(reading.factors[3].score, 20);
    }

    #[test]
    fn libra_and_cancer_show_their_challenge() {
        let reading = calculate_compatibility("libra", "cancer").expect("known signs");

        assert_eq!(reading.score, 51);
        assert_eq!(reading.level, "差异成长");
        assert_eq!(reading.title, "差异里的吸引力");
    }

    #[test]
    fn emotion_guide_changes_with_selected_mood() {
        let reading = fixture_reading();

        let happy = render_emotion_guide("开心", &reading).expect("happy guide");
        let tired = render_emotion_guide("疲惫", &reading).expect("tired guide");

        assert_eq!(happy.title, "把快乐分享给喜欢的人");
        assert_eq!(happy.need, "分享与保留");
        assert_eq!(tired.title, "有力量，也要允许自己停一停");
        assert_eq!(tired.need, "休息与减负");
        assert_ne!(happy.summary, tired.summary);
    }

    fn fixture_reading() -> DailyReading {
        DailyReading {
            id: "reading-test".into(),
            date: "2026-07-23".into(),
            weekday: "星期四".into(),
            zodiac: zodiac_from_birthday("1998-10-08").expect("zodiac"),
            personalization_mode: "sunSignLite".into(),
            hero: ReadingHero {
                title: "测试标题".into(),
                subtitle: "测试副标题".into(),
                theme: "校准".into(),
            },
            scores: ReadingScores {
                overall: 79,
                dimensions: ScoreDimensions {
                    love: 76,
                    work: 85,
                    wealth: 76,
                    social: 83,
                    inner: 72,
                },
            },
            briefs: Vec::<ReadingBrief>::new(),
            echo: String::new(),
            lucky: calculate_reading(
                &Profile {
                    id: "profile".into(),
                    guest_id: "guest".into(),
                    nickname: "旅人".into(),
                    birthday: "1998-10-08".into(),
                    birth_time: None,
                    birth_city: None,
                    zodiac: zodiac_from_birthday("1998-10-08").expect("zodiac"),
                    personalization_mode: "sunSignLite".into(),
                    completeness: 40,
                    calculation_version: 1,
                    created_at: String::new(),
                    updated_at: String::new(),
                },
                &[],
                NaiveDate::from_ymd_opt(2026, 7, 23).expect("date"),
            )
            .0
            .lucky,
            source: ReadingSource {
                provider: "fixture".into(),
                rules_version: "test".into(),
                narrative_provider: "template".into(),
                stale: false,
            },
        }
    }
}
