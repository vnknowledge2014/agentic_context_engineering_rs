// ACE Framework - Agentic Context Engineering
use crate::functional_core::*;
use crate::imperative_shell::*;
use crate::types::*;

pub struct ACEGenerator {
    client: OllamaClient,
}

impl ACEGenerator {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }

    pub async fn generate_trajectory(
        &self,
        query: &str,
        context: &ContextState,
    ) -> Result<Trajectory> {
        let bullets = get_relevant_bullets(context, query, 10);
        let context_text = build_context_prompt(&bullets);

        let prompt = format!(
            "{}\n\nProvide a brief answer in this format:\nSTEPS: [step1; step2; step3]\nOUTCOME: your answer here\nSUCCESS: true\nUSED_BULLETS: []",
            query
        );

        let response = self.client.generate(&prompt).await?;
        Ok(parse_trajectory_response(query.to_string(), &response))
    }
}

pub struct ACEReflector {
    client: OllamaClient,
}

impl ACEReflector {
    pub fn new(client: OllamaClient) -> Self {
        Self { client }
    }

    pub async fn reflect(&self, trajectory: &Trajectory) -> Result<Vec<Insight>> {
        let steps_text: Vec<String> = trajectory
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

    pub fn create_delta(&self, insights: Vec<Insight>) -> DeltaUpdate {
        insights_to_delta(insights)
    }

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

pub struct ACEFramework {
    generator: ACEGenerator,
    reflector: ACEReflector,
    curator: ACECurator,
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
        let bullets = get_relevant_bullets(context, query, 10);
        let _context_text = build_context_prompt(&bullets);

        let prompt = format!(
            "{}\n\nProvide a brief answer in this format:\nSTEPS: [step1; step2; step3]\nOUTCOME: your answer here\nSUCCESS: true\nUSED_BULLETS: []",
            query
        );

        let stream = self.generator.client.generate_stream(&prompt).await?;
        Ok(stream)
    }

    pub fn get_context_stats(&self) -> ContextStats {
        self.curator.get_stats()
    }
}
