// ACE Framework - Agentic Context Engineering
use crate::functional_core::*;
use crate::imperative_shell::*;
use crate::types::*;

pub struct ACEGenerator {
    pub client: OllamaClient,
}

impl ACEGenerator {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }

    #[allow(unused)]
    pub async fn generate_trajectory(
        &self,
        query: &str,
        context: &ContextState,
    ) -> Result<Trajectory> {
        let bullets = get_relevant_bullets(context, query, 10);
        let _context_text = build_context_prompt(&bullets);

        let prompt = format!(
            "{}\n\nProvide a brief answer in this format:\nSTEPS: [step1; step2; step3]\nOUTCOME: your answer here\nSUCCESS: true\nUSED_BULLETS: []",
            query
        );

        let response = self.client.generate(&prompt).await?;
        Ok(parse_trajectory_response(query.to_string(), &response))
    }
}

pub struct ACEReflector {
    pub client: OllamaClient,
}

impl ACEReflector {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }

    #[allow(unused)]
    pub async fn reflect(&self, trajectory: &Trajectory) -> Result<Vec<Insight>> {
        let _steps_text: Vec<String> = trajectory
            .steps
            .iter()
            .take(3)
            .map(|s| s.description.clone())
            .collect();

        let prompt = format!(
            "Based on this task: {}\nResult: {}\n\nProvide one key insight:\n[Content: key learning from this task; Type: strategy; Confidence: 0.8]",
            trajectory.query, trajectory.outcome
        );

        let response = self.client.generate(&prompt).await?;
        Ok(parse_insights_response(&response, trajectory.query.clone()))
    }
}

pub struct ACECurator {
    context: ContextState,
}

impl ACECurator {
    pub fn new() -> Self {
        Self {
            context: ContextState::new(),
        }
    }

    #[allow(unused)]
    pub fn create_delta(&self, insights: Vec<Insight>) -> DeltaUpdate {
        insights_to_delta(insights)
    }

    #[allow(unused)]
    pub fn apply_delta(&mut self, delta: &DeltaUpdate) {
        self.context = merge_delta(&self.context, delta);
    }

    pub fn get_context(&self) -> &ContextState {
        &self.context
    }

    pub fn get_stats(&self) -> ContextStats {
        let helpful = self
            .context
            .bullets
            .values()
            .filter(|b| b.helpful_count > b.harmful_count)
            .count();

        let avg_helpfulness = if self.context.bullets.is_empty() {
            0.0
        } else {
            self.context
                .bullets
                .values()
                .map(|b| b.helpful_count as f64)
                .sum::<f64>()
                / self.context.bullets.len() as f64
        };

        ContextStats {
            total_bullets: self.context.bullets.len(),
            helpful_bullets: helpful,
            version: self.context.version,
            avg_helpfulness,
        }
    }
}

pub struct ContextStats {
    pub total_bullets: usize,
    pub helpful_bullets: usize,
    pub version: i32,
    pub avg_helpfulness: f64,
}

#[allow(dead_code)]
pub struct ACEFramework {
    pub generator: ACEGenerator,
    pub reflector: ACEReflector,
    pub curator: ACECurator,
}

impl ACEFramework {
    pub fn new(config: OllamaConfig) -> Self {
        let client1 = OllamaClient::new(config.clone());
        let client2 = OllamaClient::new(config);

        Self {
            generator: ACEGenerator::new(client1),
            reflector: ACEReflector::new(client2),
            curator: ACECurator::new(),
        }
    }

    pub async fn initialize(&self) -> Result<bool> {
        match self.generator.client.initialize().await {
            Ok(_) => {
                log_success("ACE Framework initialized");
                Ok(true)
            }
            Err(e) => {
                log_error(&format!("Initialization failed: {}", e));
                Err(e)
            }
        }
    }

    pub async fn process_query_stream(
        &mut self,
        query: &str,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let context = self.curator.get_context();
        
        // Get recent conversation bullets
        let mut conv_bullets: Vec<_> = context.bullets.values()
            .filter(|b| b.tags.contains(&"conversation".to_string()))
            .collect();
        conv_bullets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let recent_conv: Vec<_> = conv_bullets.into_iter().take(1).cloned().collect();
        
        let is_continue = query.trim().to_lowercase() == "continue" || 
                         query.trim().to_lowercase() == "tiếp tục";

        let prompt = if is_continue && !recent_conv.is_empty() {
            let last_conv = &recent_conv[0].content;
            format!(
                "{}\n\nContinue from where you stopped. Do not repeat, just continue:",
                last_conv
            )
        } else if !recent_conv.is_empty() {
            let context_text = build_context_prompt(&recent_conv);
            format!(
                "Previous conversation:\n{}\n\nNew query: {}\n\nAnswer:",
                context_text, query
            )
        } else {
            query.to_string()
        };

        let stream = self.generator.client.generate_stream(&prompt).await?;
        Ok(stream)
    }

    pub async fn learn_from_interaction(&mut self, query: &str, response: &str) {
        // Save full conversation as context
        let conv_text = format!("Q: {}\nA: {}", query, response);
        let bullet = create_bullet(conv_text, vec!["conversation".to_string()]);
        let delta = DeltaUpdate {
            bullets: vec![bullet],
            timestamp: chrono::Utc::now(),
        };
        self.curator.apply_delta(&delta);
    }
    
    pub fn get_context_stats(&self) -> ContextStats {
        self.curator.get_stats()
    }
}
