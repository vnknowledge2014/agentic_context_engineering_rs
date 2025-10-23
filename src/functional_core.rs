// ACE Functional Core - Pure Functions
use crate::types::*;
use chrono::Utc;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// Pure functions for context operations
pub fn create_bullet(content: String, tags: Vec<String>) -> ContextBullet {
    ContextBullet {
        id: Uuid::new_v4().to_string(),
        content,
        helpful_count: 0,
        harmful_count: 0,
        created_at: Utc::now(),
        tags,
    }
}

pub fn update_bullet_feedback(bullet: &ContextBullet, helpful: bool) -> ContextBullet {
    ContextBullet {
        id: bullet.id.clone(),
        content: bullet.content.clone(),
        helpful_count: bullet.helpful_count + if helpful { 1 } else { 0 },
        harmful_count: bullet.harmful_count + if helpful { 0 } else { 1 },
        created_at: bullet.created_at,
        tags: bullet.tags.clone(),
    }
}

pub fn score_bullet(bullet: &ContextBullet, query_words: &HashSet<String>) -> f64 {
    let bullet_words: HashSet<String> = bullet
        .content
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    
    let overlap = query_words.intersection(&bullet_words).count() as f64;
    let feedback_score = (bullet.helpful_count - bullet.harmful_count) as f64 * 0.1;
    overlap + feedback_score
}

pub fn get_relevant_bullets(
    context: &ContextState,
    query: &str,
    max_bullets: usize,
) -> Vec<ContextBullet> {
    if context.bullets.is_empty() {
        return Vec::new();
    }

    let query_words: HashSet<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let mut scored: Vec<(f64, ContextBullet)> = context
        .bullets
        .values()
        .map(|b| (score_bullet(b, &query_words), b.clone()))
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scored
        .into_iter()
        .filter(|(score, _)| *score > 0.0)
        .take(max_bullets)
        .map(|(_, b)| b)
        .collect()
}

pub fn merge_delta(context: &ContextState, delta: &DeltaUpdate) -> ContextState {
    let mut new_bullets = context.bullets.clone();

    for bullet in &delta.bullets {
        if let Some(existing_id) = find_duplicate_bullet(bullet, &new_bullets) {
            if let Some(existing) = new_bullets.get(&existing_id) {
                new_bullets.insert(existing_id, update_bullet_feedback(existing, true));
            }
        } else {
            new_bullets.insert(bullet.id.clone(), bullet.clone());
        }
    }

    ContextState {
        bullets: new_bullets,
        version: context.version + 1,
    }
}

pub fn find_duplicate_bullet(
    new_bullet: &ContextBullet,
    existing: &HashMap<String, ContextBullet>,
) -> Option<String> {
    let new_words: HashSet<String> = new_bullet
        .content
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    for (id, bullet) in existing {
        let existing_words: HashSet<String> = bullet
            .content
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if !new_words.is_empty() && !existing_words.is_empty() {
            let overlap = new_words.intersection(&existing_words).count();
            let similarity = overlap as f64 / new_words.len() as f64;
            if similarity >= 0.7 {
                return Some(id.clone());
            }
        }
    }
    None
}

pub fn parse_trajectory_response(query: String, response: &str) -> Trajectory {
    let steps_re = Regex::new(r"(?i)STEPS:\s*\[(.*?)\]").unwrap();
    let outcome_re = Regex::new(r"(?i)OUTCOME:\s*(.+?)(?=\n|$)").unwrap();
    let success_re = Regex::new(r"(?i)SUCCESS:\s*(true|false)").unwrap();

    let steps = if let Some(caps) = steps_re.captures(response) {
        caps.get(1)
            .map(|m| m.as_str())
            .unwrap_or("")
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(|s| ReasoningStep {
                description: s.trim().to_string(),
                timestamp: Utc::now(),
            })
            .collect()
    } else {
        vec![ReasoningStep {
            description: "Processed query".to_string(),
            timestamp: Utc::now(),
        }]
    };

    let outcome = outcome_re
        .captures(response)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| response.chars().take(200).collect());

    let success = success_re
        .captures(response)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_lowercase() == "true")
        .unwrap_or(true);

    Trajectory {
        query,
        steps,
        outcome,
        success,
        used_bullets: Vec::new(),
        feedback: None,
    }
}

pub fn parse_insights_response(response: &str, source_id: String) -> Vec<Insight> {
    let re = Regex::new(r"(?i)\[Content:\s*(.+?);\s*Type:\s*(.+?);\s*Confidence:\s*([0-9.]+)\]")
        .unwrap();

    let mut insights = Vec::new();
    for caps in re.captures_iter(response) {
        if let (Some(content), Some(itype), Some(conf)) = (caps.get(1), caps.get(2), caps.get(3)) {
            if let Ok(confidence) = conf.as_str().parse::<f64>() {
                if (0.0..=1.0).contains(&confidence) {
                    insights.push(Insight {
                        content: content.as_str().trim().to_string(),
                        insight_type: itype.as_str().trim().to_string(),
                        confidence,
                        source_id: source_id.clone(),
                    });
                }
            }
        }
    }

    if insights.is_empty() {
        insights.push(Insight {
            content: "Task completed successfully".to_string(),
            insight_type: "strategy".to_string(),
            confidence: 0.5,
            source_id,
        });
    }

    insights
}

pub fn insights_to_delta(insights: Vec<Insight>) -> DeltaUpdate {
    let bullets = insights
        .into_iter()
        .filter(|i| i.confidence >= 0.5)
        .map(|i| create_bullet(i.content, vec![i.insight_type]))
        .collect();

    DeltaUpdate {
        bullets,
        timestamp: Utc::now(),
    }
}

pub fn build_context_prompt(bullets: &[ContextBullet]) -> String {
    if bullets.is_empty() {
        return "No previous context available.".to_string();
    }

    bullets
        .iter()
        .map(|b| {
            format!(
                "[{}] {} (helpful: {}, harmful: {})",
                &b.id[..8.min(b.id.len())],
                b.content,
                b.helpful_count,
                b.harmful_count
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
