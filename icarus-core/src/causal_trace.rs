// Markovian Thinker: Causal Set Trace Management
// Partially ordered set of reasoning events inspired by Icarus TIC and causal set theory

use crate::events::{ReasoningEvent, ReasoningLevel};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Causal trace: partially ordered set of reasoning events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalTrace {
    /// All events indexed by ID
    events: HashMap<Uuid, CausalEvent>,

    /// Causal edges: (cause_id, effect_id)
    /// Represents "cause → effect" relationship
    causal_edges: Vec<(Uuid, Uuid)>,

    /// Root events (no predecessors)
    roots: Vec<Uuid>,

    /// Leaf events (no successors)
    leaves: Vec<Uuid>,

    /// Reasoning branches
    branches: Vec<ReasoningBranch>,

    /// Trace metadata
    metadata: CausalTraceMetadata,
}

/// Event with causal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvent {
    /// Event ID
    pub id: Uuid,

    /// The reasoning event
    pub event: ReasoningEvent,

    /// Direct predecessors (causes)
    pub predecessors: Vec<Uuid>,

    /// Direct successors (effects)
    pub successors: Vec<Uuid>,

    /// Depth in causal graph (distance from root)
    pub depth: usize,

    /// Reasoning level
    pub level: ReasoningLevel,

    /// Branch this event belongs to
    pub branch_id: Option<Uuid>,
}

/// Reasoning branch (forked execution path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningBranch {
    /// Branch ID
    pub id: Uuid,

    /// Parent branch (if forked)
    pub parent: Option<Uuid>,

    /// Fork point event
    pub fork_point: Uuid,

    /// Events in this branch
    pub events: Vec<Uuid>,

    /// Branch state
    pub state: BranchState,

    /// Branch priority
    pub priority: f32,
}

/// Branch state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchState {
    /// Active and processing
    Active,

    /// Completed successfully
    Completed,

    /// Pruned (lower priority)
    Pruned,

    /// Failed with error
    Failed,
}

/// Causal trace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalTraceMetadata {
    /// Trace creation time
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update time
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Total events
    pub total_events: usize,

    /// Total branches
    pub total_branches: usize,

    /// Session ID
    pub session_id: Uuid,
}

impl CausalTrace {
    /// Create new causal trace
    pub fn new(session_id: Uuid) -> Self {
        Self {
            events: HashMap::new(),
            causal_edges: Vec::new(),
            roots: Vec::new(),
            leaves: Vec::new(),
            branches: Vec::new(),
            metadata: CausalTraceMetadata {
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                total_events: 0,
                total_branches: 0,
                session_id,
            },
        }
    }

    /// Add event to trace
    pub fn add_event(
        &mut self,
        event: ReasoningEvent,
        level: ReasoningLevel,
        predecessors: Vec<Uuid>,
    ) -> Uuid {
        let event_id = event.event_id();

        // Create causal event
        let causal_event = CausalEvent {
            id: event_id,
            event,
            predecessors: predecessors.clone(),
            successors: Vec::new(),
            depth: self.calculate_depth(&predecessors),
            level,
            branch_id: None,
        };

        // Add causal edges
        for pred_id in &predecessors {
            self.causal_edges.push((*pred_id, event_id));

            // Update predecessor's successors
            if let Some(pred) = self.events.get_mut(pred_id) {
                pred.successors.push(event_id);
            }
        }

        // Update roots/leaves
        if predecessors.is_empty() {
            self.roots.push(event_id);
        } else {
            // Remove predecessors from leaves
            self.leaves.retain(|id| !predecessors.contains(id));
        }
        self.leaves.push(event_id);

        // Insert event
        self.events.insert(event_id, causal_event);

        // Update metadata
        self.metadata.total_events = self.events.len();
        self.metadata.updated_at = chrono::Utc::now();

        event_id
    }

    /// Check if event_a causally precedes event_b
    pub fn precedes(&self, a: Uuid, b: Uuid) -> bool {
        if a == b {
            return false;
        }

        // Use BFS to check if there's a path from a to b
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(a);

        while let Some(current) = queue.pop_front() {
            if current == b {
                return true;
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(event) = self.events.get(&current) {
                for succ in &event.successors {
                    queue.push_back(*succ);
                }
            }
        }

        false
    }

    /// Get causal past (all events that causally precede given event)
    pub fn causal_past(&self, event_id: Uuid) -> Vec<Uuid> {
        let mut past = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(event) = self.events.get(&event_id) {
            for pred in &event.predecessors {
                queue.push_back(*pred);
            }
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            past.push(current);

            if let Some(event) = self.events.get(&current) {
                for pred in &event.predecessors {
                    queue.push_back(*pred);
                }
            }
        }

        past
    }

    /// Get causal future (all events that causally follow given event)
    pub fn causal_future(&self, event_id: Uuid) -> Vec<Uuid> {
        let mut future = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(event) = self.events.get(&event_id) {
            for succ in &event.successors {
                queue.push_back(*succ);
            }
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            future.push(current);

            if let Some(event) = self.events.get(&current) {
                for succ in &event.successors {
                    queue.push_back(*succ);
                }
            }
        }

        future
    }

    /// Detect cycles in causal graph
    pub fn detect_cycles(&self) -> Vec<Vec<Uuid>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut current_path = Vec::new();

        for root in &self.roots {
            self.detect_cycles_dfs(
                *root,
                &mut visited,
                &mut rec_stack,
                &mut current_path,
                &mut cycles,
            );
        }

        cycles
    }

    /// DFS helper for cycle detection
    fn detect_cycles_dfs(
        &self,
        node: Uuid,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        current_path: &mut Vec<Uuid>,
        cycles: &mut Vec<Vec<Uuid>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        current_path.push(node);

        if let Some(event) = self.events.get(&node) {
            for succ in &event.successors {
                if !visited.contains(succ) {
                    self.detect_cycles_dfs(*succ, visited, rec_stack, current_path, cycles);
                } else if rec_stack.contains(succ) {
                    // Found cycle
                    let cycle_start = current_path.iter().position(|&id| id == *succ).unwrap();
                    cycles.push(current_path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(&node);
        current_path.pop();
    }

    /// Calculate depth (distance from root)
    fn calculate_depth(&self, predecessors: &[Uuid]) -> usize {
        predecessors
            .iter()
            .filter_map(|id| self.events.get(id))
            .map(|e| e.depth + 1)
            .max()
            .unwrap_or(0)
    }

    /// Create new branch
    pub fn create_branch(&mut self, fork_point: Uuid, priority: f32) -> Uuid {
        let branch_id = Uuid::new_v4();

        let parent_branch = self
            .events
            .get(&fork_point)
            .and_then(|e| e.branch_id);

        let branch = ReasoningBranch {
            id: branch_id,
            parent: parent_branch,
            fork_point,
            events: Vec::new(),
            state: BranchState::Active,
            priority,
        };

        self.branches.push(branch);
        self.metadata.total_branches = self.branches.len();

        branch_id
    }

    /// Assign event to branch
    pub fn assign_to_branch(&mut self, event_id: Uuid, branch_id: Uuid) {
        if let Some(event) = self.events.get_mut(&event_id) {
            event.branch_id = Some(branch_id);
        }

        if let Some(branch) = self.branches.iter_mut().find(|b| b.id == branch_id) {
            branch.events.push(event_id);
        }
    }

    /// Get all events in chronological order
    pub fn chronological_order(&self) -> Vec<Uuid> {
        let mut events: Vec<_> = self.events.values().collect();
        events.sort_by_key(|e| e.event.timestamp());
        events.iter().map(|e| e.id).collect()
    }

    /// Get events by level
    pub fn events_by_level(&self, level: ReasoningLevel) -> Vec<Uuid> {
        self.events
            .values()
            .filter(|e| e.level == level)
            .map(|e| e.id)
            .collect()
    }

    /// Export to GraphViz DOT format
    pub fn to_graphviz(&self) -> String {
        let mut dot = String::from("digraph CausalTrace {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box];\n\n");

        // Add nodes with labels
        for (id, event) in &self.events {
            let label = self.event_label(&event.event);
            let color = match event.level {
                ReasoningLevel::Micro => "lightblue",
                ReasoningLevel::Meso => "lightgreen",
                ReasoningLevel::Macro => "lightyellow",
            };

            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor={}, style=filled];\n",
                id, label, color
            ));
        }

        dot.push_str("\n");

        // Add edges
        for (cause, effect) in &self.causal_edges {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", cause, effect));
        }

        // Add branch subgraphs
        for (i, branch) in self.branches.iter().enumerate() {
            dot.push_str(&format!("\n  subgraph cluster_{} {{\n", i));
            dot.push_str(&format!("    label=\"Branch {}\";\n", i));
            dot.push_str("    color=gray;\n");

            for event_id in &branch.events {
                dot.push_str(&format!("    \"{}\";\n", event_id));
            }

            dot.push_str("  }\n");
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate human-readable event label
    fn event_label(&self, event: &ReasoningEvent) -> String {
        match event {
            ReasoningEvent::ChunkRequest { level, .. } => {
                format!("ChunkReq\\n{:?}", level)
            }
            ReasoningEvent::ChunkComplete { tokens, .. } => {
                format!("ChunkDone\\n{} tok", tokens)
            }
            ReasoningEvent::VerificationRequest { .. } => "Verify\\nReq".to_string(),
            ReasoningEvent::VerificationComplete { .. } => "Verify\\nDone".to_string(),
            ReasoningEvent::ConceptCrystallization { concept, .. } => {
                format!("Crystal\\n{}", concept.chars().take(10).collect::<String>())
            }
            ReasoningEvent::ConceptCrystallized { .. } => "Crystalized".to_string(),
            ReasoningEvent::TerminationCheck { .. } => "Term\\nCheck".to_string(),
            ReasoningEvent::SessionTerminated { .. } => "Session\\nEnd".to_string(),
        }
    }

    /// Get statistics
    pub fn statistics(&self) -> TraceStatistics {
        let micro_count = self.events_by_level(ReasoningLevel::Micro).len();
        let meso_count = self.events_by_level(ReasoningLevel::Meso).len();
        let macro_count = self.events_by_level(ReasoningLevel::Macro).len();

        let avg_depth = if !self.events.is_empty() {
            self.events.values().map(|e| e.depth).sum::<usize>() as f32
                / self.events.len() as f32
        } else {
            0.0
        };

        let max_depth = self.events.values().map(|e| e.depth).max().unwrap_or(0);

        TraceStatistics {
            total_events: self.events.len(),
            micro_events: micro_count,
            meso_events: meso_count,
            macro_events: macro_count,
            total_branches: self.branches.len(),
            active_branches: self
                .branches
                .iter()
                .filter(|b| b.state == BranchState::Active)
                .count(),
            total_edges: self.causal_edges.len(),
            avg_depth,
            max_depth,
            has_cycles: !self.detect_cycles().is_empty(),
        }
    }

    /// Get event by ID
    pub fn get_event(&self, id: Uuid) -> Option<&CausalEvent> {
        self.events.get(&id)
    }

    /// Get all events
    pub fn all_events(&self) -> Vec<&CausalEvent> {
        self.events.values().collect()
    }

    /// Get metadata
    pub fn metadata(&self) -> &CausalTraceMetadata {
        &self.metadata
    }
}

/// Trace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStatistics {
    pub total_events: usize,
    pub micro_events: usize,
    pub meso_events: usize,
    pub macro_events: usize,
    pub total_branches: usize,
    pub active_branches: usize,
    pub total_edges: usize,
    pub avg_depth: f32,
    pub max_depth: usize,
    pub has_cycles: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(session_id: Uuid, timestamp: u64) -> ReasoningEvent {
        ReasoningEvent::ChunkRequest {
            session_id,
            prompt: "Test".to_string(),
            priority: 0.5,
            timestamp,
            level: ReasoningLevel::Macro,
        }
    }

    #[test]
    fn test_causal_trace_creation() {
        let session_id = Uuid::new_v4();
        let trace = CausalTrace::new(session_id);

        assert_eq!(trace.metadata.session_id, session_id);
        assert_eq!(trace.metadata.total_events, 0);
        assert!(trace.roots.is_empty());
        assert!(trace.leaves.is_empty());
    }

    #[test]
    fn test_add_event() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        let event = create_test_event(session_id, 1000);
        let event_id = trace.add_event(event, ReasoningLevel::Macro, vec![]);

        assert_eq!(trace.metadata.total_events, 1);
        assert_eq!(trace.roots.len(), 1);
        assert_eq!(trace.leaves.len(), 1);
        assert_eq!(trace.roots[0], event_id);
    }

    #[test]
    fn test_causal_chain() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        // Create chain: e1 → e2 → e3
        let e1 = trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Macro,
            vec![],
        );

        let e2 = trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Macro,
            vec![e1],
        );

        let e3 = trace.add_event(
            create_test_event(session_id, 1002),
            ReasoningLevel::Macro,
            vec![e2],
        );

        assert!(trace.precedes(e1, e2));
        assert!(trace.precedes(e2, e3));
        assert!(trace.precedes(e1, e3)); // Transitive
        assert!(!trace.precedes(e3, e1)); // Not reverse
    }

    #[test]
    fn test_causal_past_future() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        // Create chain: e1 → e2 → e3
        let e1 = trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Macro,
            vec![],
        );

        let e2 = trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Macro,
            vec![e1],
        );

        let e3 = trace.add_event(
            create_test_event(session_id, 1002),
            ReasoningLevel::Macro,
            vec![e2],
        );

        let past = trace.causal_past(e3);
        assert_eq!(past.len(), 2);
        assert!(past.contains(&e1));
        assert!(past.contains(&e2));

        let future = trace.causal_future(e1);
        assert_eq!(future.len(), 2);
        assert!(future.contains(&e2));
        assert!(future.contains(&e3));
    }

    #[test]
    fn test_branching() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        // Create fork: e1 → e2
        //                  → e3
        let e1 = trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Macro,
            vec![],
        );

        let e2 = trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Macro,
            vec![e1],
        );

        let e3 = trace.add_event(
            create_test_event(session_id, 1002),
            ReasoningLevel::Macro,
            vec![e1],
        );

        assert!(trace.precedes(e1, e2));
        assert!(trace.precedes(e1, e3));
        assert!(!trace.precedes(e2, e3)); // Parallel branches
        assert!(!trace.precedes(e3, e2));
    }

    #[test]
    fn test_cycle_detection() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        // Create acyclic graph first
        let e1 = trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Macro,
            vec![],
        );

        let _e2 = trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Macro,
            vec![e1],
        );

        let cycles = trace.detect_cycles();
        assert!(cycles.is_empty());

        // Note: In real implementation, cycles shouldn't be possible
        // as we prevent adding edges that would create cycles
    }

    #[test]
    fn test_branch_creation() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        let fork_point = trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Macro,
            vec![],
        );

        let branch_id = trace.create_branch(fork_point, 0.8);

        assert_eq!(trace.branches.len(), 1);
        assert_eq!(trace.metadata.total_branches, 1);

        let e2 = trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Macro,
            vec![fork_point],
        );

        trace.assign_to_branch(e2, branch_id);

        let branch = &trace.branches[0];
        assert_eq!(branch.events.len(), 1);
        assert_eq!(branch.events[0], e2);
    }

    #[test]
    fn test_statistics() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        // Add events at different levels
        trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Micro,
            vec![],
        );

        trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Meso,
            vec![],
        );

        trace.add_event(
            create_test_event(session_id, 1002),
            ReasoningLevel::Macro,
            vec![],
        );

        let stats = trace.statistics();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.micro_events, 1);
        assert_eq!(stats.meso_events, 1);
        assert_eq!(stats.macro_events, 1);
        assert!(!stats.has_cycles);
    }

    #[test]
    fn test_graphviz_export() {
        let session_id = Uuid::new_v4();
        let mut trace = CausalTrace::new(session_id);

        let e1 = trace.add_event(
            create_test_event(session_id, 1000),
            ReasoningLevel::Macro,
            vec![],
        );

        let _e2 = trace.add_event(
            create_test_event(session_id, 1001),
            ReasoningLevel::Macro,
            vec![e1],
        );

        let dot = trace.to_graphviz();

        assert!(dot.contains("digraph CausalTrace"));
        assert!(dot.contains("ChunkReq"));
        assert!(dot.contains("->"));
    }
}
