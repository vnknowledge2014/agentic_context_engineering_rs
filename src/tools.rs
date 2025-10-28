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

pub struct SearchTool {
    pub enable_web_search: bool,
}

impl SearchTool {
    pub fn new(enable_web_search: bool) -> Self {
        Self { enable_web_search }
    }

    pub fn search_context(&self, query: &str, bullets: &HashMap<String, ContextBullet>) -> Vec<SearchResult> {
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
                        source: "context".to_string(),
                        url: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.relevance.cmp(&a.relevance));
        results.into_iter().take(5).collect()
    }

    pub async fn search_web(&self, query: &str) -> Vec<SearchResult> {
        if !self.enable_web_search {
            return vec![];
        }

        let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1", 
            urlencoding::encode(query));
        
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    let mut results = Vec::new();
                    
                    if let Some(abstract_text) = data["Abstract"].as_str() {
                        if !abstract_text.is_empty() {
                            results.push(SearchResult {
                                content: abstract_text.to_string(),
                                relevance: 10,
                                tags: vec![],
                                source: "web".to_string(),
                                url: data["AbstractURL"].as_str().map(|s| s.to_string()),
                            });
                        }
                    }
                    
                    if let Some(topics) = data["RelatedTopics"].as_array() {
                        for topic in topics.iter().take(3) {
                            if let Some(text) = topic["Text"].as_str() {
                                results.push(SearchResult {
                                    content: text.to_string(),
                                    relevance: 5,
                                    tags: vec![],
                                    source: "web".to_string(),
                                    url: topic["FirstURL"].as_str().map(|s| s.to_string()),
                                });
                            }
                        }
                    }
                    
                    return results;
                }
            }
            _ => {}
        }
        vec![]
    }

    pub async fn search(&self, query: &str, bullets: &HashMap<String, ContextBullet>) -> Vec<SearchResult> {
        let mut context_results = self.search_context(query, bullets);
        let web_results = self.search_web(query).await;
        
        context_results.extend(web_results);
        context_results.sort_by(|a, b| b.relevance.cmp(&a.relevance));
        context_results.into_iter().take(5).collect()
    }
}

pub struct SearchResult {
    pub content: String,
    pub relevance: usize,
    pub tags: Vec<String>,
    pub source: String,
    pub url: Option<String>,
}

pub struct DeepResearchTool {
    pub enable_web_search: bool,
}

impl DeepResearchTool {
    pub fn new(enable_web_search: bool) -> Self {
        Self { enable_web_search }
    }

    pub async fn research(
        &self,
        topic: &str,
        client: &OllamaClient,
        bullets: &HashMap<String, ContextBullet>,
    ) -> Result<String> {
        let mut output = Vec::new();
        
        output.push("🔍 Step 1: Searching knowledge sources...".to_string());
        let search_tool = SearchTool::new(self.enable_web_search);
        let existing = search_tool.search(topic, bullets).await;
        
        if !existing.is_empty() {
            output.push(format!("   Found {} relevant sources", existing.len()));
            for (i, result) in existing.iter().take(3).enumerate() {
                let source_type = if result.source == "web" { "🌐 Web" } else { "📚 Context" };
                let preview: String = result.content.chars().take(80).collect();
                output.push(format!("   {}. {}: {}...", i + 1, source_type, preview));
            }
        }
        
        output.push("\n🤔 Step 2: Generating research questions...".to_string());
        let questions_prompt = format!(
            "Research topic: {}\n\nBased on available information, generate 3 specific research questions to explore:",
            topic
        );
        
        let questions = client.generate(&questions_prompt).await?;
        let question_list: Vec<String> = questions
            .lines()
            .take(3)
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect();
        
        for (i, q) in question_list.iter().enumerate() {
            output.push(format!("   Q{}: {}", i + 1, q));
        }
        
        output.push("\n💡 Step 3: Researching answers...".to_string());
        let mut answers = Vec::new();
        for (i, question) in question_list.iter().enumerate() {
            let q_results = search_tool.search(question, bullets).await;
            let context_info: String = q_results
                .iter()
                .take(2)
                .map(|r| r.content.chars().take(150).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            
            let answer_prompt = format!(
                "Question: {}\n\nRelevant information:\n{}\n\nProvide detailed answer:",
                question, context_info
            );
            
            if let Ok(answer) = client.generate(&answer_prompt).await {
                output.push(format!("   ✓ Answered Q{}", i + 1));
                answers.push(format!("Q{}: {}\nA{}: {}", i + 1, question, i + 1, answer));
            }
        }
        
        output.push("\n📝 Step 4: Synthesizing comprehensive report...\n".to_string());
        
        let sources_text: String = existing
            .iter()
            .take(3)
            .map(|e| format!("- {}", e.content.chars().take(200).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n");
        
        let synthesis_prompt = format!(
            "Research topic: {}\n\nSources consulted:\n{}\n\nResearch findings:\n{}\n\nSynthesize a comprehensive, well-structured report with:\n1. Executive summary\n2. Key findings\n3. Detailed analysis\n4. Conclusion\n\nReport:",
            topic,
            sources_text,
            answers.join("\n")
        );
        
        let synthesis = client.generate(&synthesis_prompt).await?;
        
        output.push("=".repeat(60));
        output.push(synthesis);
        
        Ok(output.join("\n"))
    }
}
