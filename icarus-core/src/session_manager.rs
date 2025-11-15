// Session Manager for Markovian Reasoning
// Manages the lifecycle of reasoning sessions

use crate::causal_trace::CausalTrace;
use crate::concept_space::{ConceptSpace, ConceptSpaceConfig};
use crate::state::{MarkovianState, StateConfig};
use crate::storm_mitigation::StormMitigation;
use crate::trace::{ReasoningTrace, TerminationReason};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

/// A single reasoning session
#[derive(Debug, Clone)]
pub struct ReasoningSession {
    pub id: Uuid,
    pub state: MarkovianState,
    pub trace: ReasoningTrace,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Session metadata for listing
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub problem: String,
    pub iteration: usize,
    pub total_tokens: usize,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_complete: bool,
}

/// Manages all active reasoning sessions
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<Uuid, ReasoningSession>>>,
    storm_mitigations: Arc<Mutex<HashMap<Uuid, StormMitigation>>>,
    causal_traces: Arc<Mutex<HashMap<Uuid, CausalTrace>>>,
    concept_spaces: Arc<Mutex<HashMap<Uuid, ConceptSpace>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            storm_mitigations: Arc::new(Mutex::new(HashMap::new())),
            causal_traces: Arc::new(Mutex::new(HashMap::new())),
            concept_spaces: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new reasoning session
    pub async fn create_session(
        &self,
        problem: String,
        config: StateConfig,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let state = MarkovianState::new(problem.clone(), config.clone());

        let trace = ReasoningTrace::new(
            problem.clone(),
            "claude-code".to_string(),
            config.chunk_size,
            config.carryover_size,
            config.max_iterations,
        );

        let session = ReasoningSession {
            id: session_id,
            state,
            trace,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        self.sessions.lock().await.insert(session_id, session);

        // Create storm mitigation for this session
        let storm_mitigation = StormMitigation::new(config.storm_mitigation_config.clone());
        self.storm_mitigations
            .lock()
            .await
            .insert(session_id, storm_mitigation);

        // Create causal trace for this session if enabled
        if config.enable_causal_trace {
            let causal_trace = CausalTrace::new(session_id);
            self.causal_traces
                .lock()
                .await
                .insert(session_id, causal_trace);
            tracing::debug!("Created causal trace for session {}", session_id);
        }

        // Create concept space for this session if needed
        let concept_config = ConceptSpaceConfig {
            lattice_type: config.concept_space_config.lattice_type.clone(),
            max_concepts: 10000,
            similarity_threshold: 0.999,
        };
        let concept_space = ConceptSpace::new(concept_config);
        self.concept_spaces
            .lock()
            .await
            .insert(session_id, concept_space);
        tracing::debug!("Created concept space for session {}", session_id);

        tracing::info!("Created session {} for problem: {}", session_id, problem);

        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: Uuid) -> Result<ReasoningSession> {
        self.sessions
            .lock()
            .await
            .get(&id)
            .cloned()
            .with_context(|| format!("Session {} not found", id))
    }

    /// Update a session
    pub async fn update_session(&self, session: ReasoningSession) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        if sessions.contains_key(&session.id) {
            sessions.insert(session.id, session);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session {} not found", session.id))
        }
    }

    /// Record a chunk in a session
    pub async fn record_chunk(
        &self,
        session_id: Uuid,
        prompt: String,
        output: String,
        tokens: usize,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        let session = sessions
            .get_mut(&session_id)
            .with_context(|| format!("Session {} not found", session_id))?;

        session.trace.add_chunk(prompt, output, tokens, 0); // latency_ms = 0 (not tracked in stateful mode)
        session.last_activity = Utc::now();

        Ok(())
    }

    /// Complete a session with a solution
    pub async fn complete_session(
        &self,
        session_id: Uuid,
        solution: Option<String>,
        reason: TerminationReason,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        let session = sessions
            .get_mut(&session_id)
            .with_context(|| format!("Session {} not found", session_id))?;

        session.trace.complete(solution, reason.clone());
        session.last_activity = Utc::now();

        tracing::info!("Session {} completed: {:?}", session_id, reason);

        Ok(())
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().await;

        sessions
            .values()
            .map(|session| SessionInfo {
                id: session.id,
                problem: session.trace.problem.clone(),
                iteration: session.state.iteration,
                total_tokens: session.trace.total_tokens,
                created_at: session.created_at,
                last_activity: session.last_activity,
                is_complete: session.trace.is_complete(),
            })
            .collect()
    }

    /// Remove a session
    pub async fn remove_session(&self, id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        sessions
            .remove(&id)
            .with_context(|| format!("Session {} not found", id))?;

        // Also remove storm mitigation, causal trace, and concept space
        self.storm_mitigations.lock().await.remove(&id);
        self.causal_traces.lock().await.remove(&id);
        self.concept_spaces.lock().await.remove(&id);

        tracing::info!("Removed session {}", id);

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self, max_age: Duration) -> usize {
        let mut sessions = self.sessions.lock().await;
        let now = Utc::now();

        let expired: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, session)| {
                let age = now.signed_duration_since(session.last_activity);
                age.num_seconds() > max_age.as_secs() as i64
            })
            .map(|(id, _)| *id)
            .collect();

        let count = expired.len();

        for id in &expired {
            sessions.remove(id);
        }

        // Release sessions lock before acquiring storm_mitigations lock
        drop(sessions);

        // Also remove storm mitigations, causal traces, and concept spaces for expired sessions
        if count > 0 {
            let mut mitigations = self.storm_mitigations.lock().await;
            let mut traces = self.causal_traces.lock().await;
            let mut spaces = self.concept_spaces.lock().await;
            for id in &expired {
                mitigations.remove(id);
                traces.remove(id);
                spaces.remove(id);
            }
            tracing::info!("Cleaned up {} expired sessions", count);
        }

        count
    }

    /// Get the number of active sessions
    pub async fn count(&self) -> usize {
        self.sessions.lock().await.len()
    }

    /// Check if storm mitigation allows processing a chunk for this session
    pub async fn check_storm_mitigation(
        &self,
        session_id: Uuid,
    ) -> Result<crate::storm_mitigation::MitigationDecision> {
        let mut mitigations = self.storm_mitigations.lock().await;

        let mitigation = mitigations
            .get_mut(&session_id)
            .with_context(|| format!("Session {} not found", session_id))?;

        Ok(mitigation.allow_event())
    }

    /// Record successful chunk processing for storm mitigation
    pub async fn record_storm_success(&self, session_id: Uuid) -> Result<()> {
        let mut mitigations = self.storm_mitigations.lock().await;

        if let Some(mitigation) = mitigations.get_mut(&session_id) {
            mitigation.record_success();
        }

        Ok(())
    }

    /// Record failed chunk processing for storm mitigation
    pub async fn record_storm_failure(&self, session_id: Uuid) -> Result<()> {
        let mut mitigations = self.storm_mitigations.lock().await;

        if let Some(mitigation) = mitigations.get_mut(&session_id) {
            mitigation.record_failure();
        }

        Ok(())
    }

    /// Get storm mitigation statistics for a session
    pub async fn get_storm_stats(
        &self,
        session_id: Uuid,
    ) -> Result<crate::storm_mitigation::StormMitigationStats> {
        let mitigations = self.storm_mitigations.lock().await;

        let mitigation = mitigations
            .get(&session_id)
            .with_context(|| format!("Session {} not found", session_id))?;

        Ok(mitigation.stats())
    }

    /// Get causal trace for a session
    pub async fn get_causal_trace(&self, session_id: Uuid) -> Result<CausalTrace> {
        let traces = self.causal_traces.lock().await;

        traces
            .get(&session_id)
            .cloned()
            .with_context(|| format!("Causal trace for session {} not found", session_id))
    }

    /// Query similar concepts in a session's concept space
    pub async fn query_concepts(
        &self,
        session_id: Uuid,
        embedding: Vec<f32>,
        k: usize,
    ) -> Result<Vec<crate::concept_space::Concept>> {
        let spaces = self.concept_spaces.lock().await;

        let space = spaces
            .get(&session_id)
            .with_context(|| format!("Concept space for session {} not found", session_id))?;

        // Find k most similar concepts
        let similar = space.find_similar(&embedding, k);

        // Clone the concepts to return owned values
        Ok(similar.into_iter().cloned().collect())
    }

    /// Get concept space statistics for a session
    pub async fn get_concept_stats(
        &self,
        session_id: Uuid,
    ) -> Result<crate::concept_space::ConceptSpaceStatistics> {
        let spaces = self.concept_spaces.lock().await;

        let space = spaces
            .get(&session_id)
            .with_context(|| format!("Concept space for session {} not found", session_id))?;

        Ok(space.statistics())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_session() {
        let manager = SessionManager::new();
        let config = StateConfig::default();
        let problem = "Test problem".to_string();

        let id = manager.create_session(problem.clone(), config).await.unwrap();
        let session = manager.get_session(id).await.unwrap();

        assert_eq!(session.id, id);
        assert_eq!(session.trace.problem, problem);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let manager = SessionManager::new();
        let config = StateConfig::default();

        let _id1 = manager.create_session("Problem 1".to_string(), config.clone()).await.unwrap();
        let _id2 = manager.create_session("Problem 2".to_string(), config.clone()).await.unwrap();

        let sessions = manager.list_sessions().await;
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_session() {
        let manager = SessionManager::new();
        let config = StateConfig::default();

        let id = manager.create_session("Test".to_string(), config).await.unwrap();
        assert_eq!(manager.count().await, 1);

        manager.remove_session(id).await.unwrap();
        assert_eq!(manager.count().await, 0);
    }
}
