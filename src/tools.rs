// ACE Tools - Thinking, Search, Deep Research
use crate::imperative_shell::OllamaClient;
use crate::types::*;
use std::collections::HashMap;

pub struct ThinkingTool;

impl ThinkingTool {
    pub async fn think(&self, query: &str, client: &OllamaClient) -> Result<String> {
        let prompt = format!(
            "Think deeply about this query step by step:\n\nQuery: {}\n\nProvide detailed reasoning:\n1. Break down the problem\n2. Consider multiple approaches\n3. Analyze pros and cons\n4. Reach conclusion\n\nThinking process:",
            query
        );
        client.generate_with_thinking(&prompt, true).await
    }
}

pub struct SearchTool;

impl SearchTool {
    pub fn search(&self, query: &str, bullets: &HashMap<String, ContextBullet>) -> Vec<SearchResult> {
        let query_words: std::collections::HashSet<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut results: Vec<SearchResult> = bullets
            .values()
            .filter_map(|bullet| {
                let bullet_words: std::collections::HashSet<String> = bullet
                    .content
                    .to_lowercase()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                let overlap = query_words.intersection(&bullet_words).count();
                if overlap > 0 {
                    Some(SearchResult {
                        content: bullet.content.clone(),
                        relevance: overlap,
                        tags: bullet.tags.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.relevance.cmp(&a.relevance));
        results.into_iter().take(5).collect()
    }
}

#[allow(dead_code)]
pub struct SearchResult {
    pub content: String,
    pub relevance: usize,
    pub tags: Vec<String>,
}

pub struct DeepResearchTool;

impl DeepResearchTool {
    pub async fn research(
        &self,
        topic: &str,
        client: &OllamaClient,
        bullets: &HashMap<String, ContextBullet>,
    ) -> Result<String> {
        // Step 1: Search existing knowledge
        let search_tool = SearchTool;
        let existing = search_tool.search(topic, bullets);

        // Step 2: Generate research questions
        let questions_prompt = format!(
            "Research topic: {}\n\nGenerate 3 key research questions to explore:",
            topic
        );

        let questions = client.generate(&questions_prompt).await?;

        // Step 3: Answer each question
        let mut answers = Vec::new();
        for (i, line) in questions.lines().take(3).enumerate() {
            if !line.trim().is_empty() {
                let answer_prompt = format!("Question: {}\n\nProvide detailed answer:", line.trim());
                if let Ok(answer) = client.generate(&answer_prompt).await {
                    answers.push(format!("Q{}: {}\nA{}: {}", i + 1, line.trim(), i + 1, answer));
                }
            }
        }

        // Step 4: Synthesize
        let existing_text: Vec<String> = existing
            .iter()
            .take(3)
            .map(|e| e.content.chars().take(100).collect())
            .collect();

        let synthesis_prompt = format!(
            "Research topic: {}\n\nExisting knowledge:\n{}\n\nResearch findings:\n{}\n\nSynthesize comprehensive answer:",
            topic,
            existing_text.join("\n"),
            answers.join("\n")
        );

        client.generate(&synthesis_prompt).await
    }
}
