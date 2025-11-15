//! Example: Teaching Icarus through knowledge distillation
//!
//! This demonstrates how to use the learning system to teach Icarus
//! new skills based on Claude Code's problem-solving approaches.

use icarus_core::learning::{Interaction, SkillLibrary, StrategyExtractor};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧠 Icarus Knowledge Distillation Demo\n");
    println!("Teaching Icarus skills from Claude Code's session...\n");

    let library = SkillLibrary::new();

    // ========================================================================
    // SKILL 1: Integrating External Libraries into Rust MCP Server
    // ========================================================================
    println!("📚 Teaching Skill 1: Integrating External Libraries");
    println!("─────────────────────────────────────────────────────");

    let skill1 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: "Need to integrate SkillLibrary from learning module into existing MCP server".to_string(),
        claude_reasoning: vec![
            "Import the necessary types from the learning module at the top of the file".to_string(),
            "Add a new field to the server struct to hold the library instance".to_string(),
            "Initialize the library in the server's constructor using Arc for thread-safety".to_string(),
            "Use the library in handler methods where needed".to_string(),
            "Ensure async/await patterns are properly used with RwLock".to_string(),
        ],
        solution: "Successfully integrated SkillLibrary into IcarusMCPServer with proper Arc<SkillLibrary> wrapper for thread-safe access across async handlers".to_string(),
        context: HashMap::from([
            ("domain".to_string(), "architecture".to_string()),
            ("language".to_string(), "rust".to_string()),
            ("pattern".to_string(), "dependency_injection".to_string()),
        ]),
    };

    match StrategyExtractor::extract_skill(&skill1) {
        Ok(Some(skill)) => {
            println!("✅ Extracted skill: {}", skill.name);
            println!("   Domain: {:?}", skill.domain);
            println!("   Pattern: {}", skill.pattern);
            println!("   Steps: {} actions", skill.steps.len());
            library.add_skill(skill).await?;
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // ========================================================================
    // SKILL 2: Implementing MCP Tool Handlers
    // ========================================================================
    println!("📚 Teaching Skill 2: Implementing MCP Tool Handlers");
    println!("─────────────────────────────────────────────────────");

    let skill2 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: "Implement new MCP tool handler for icarus_learn_from_interaction with proper validation and error handling".to_string(),
        claude_reasoning: vec![
            "Define the tool schema in handle_list_tools with all required parameters and their types".to_string(),
            "Add the tool name to the match statement in handle_call_tool".to_string(),
            "Create dedicated handler function that parses arguments from serde_json::Value".to_string(),
            "Validate all required fields with clear error messages for missing data".to_string(),
            "Handle optional fields with .unwrap_or_default() patterns".to_string(),
            "Process the data through the appropriate business logic (StrategyExtractor in this case)".to_string(),
            "Return CallToolResult with success/failure status and relevant data".to_string(),
        ],
        solution: "Created complete handle_learn_from_interaction that validates input, extracts skills, stores them in library, and returns detailed response with statistics".to_string(),
        context: HashMap::from([
            ("domain".to_string(), "refactoring".to_string()),
            ("language".to_string(), "rust".to_string()),
            ("pattern".to_string(), "handler_pattern".to_string()),
            ("file".to_string(), "src/mcp/server.rs".to_string()),
        ]),
    };

    match StrategyExtractor::extract_skill(&skill2) {
        Ok(Some(skill)) => {
            println!("✅ Extracted skill: {}", skill.name);
            println!("   Domain: {:?}", skill.domain);
            println!("   Pattern: {}", skill.pattern);
            println!("   Heuristics: {}", skill.heuristics.len());
            library.add_skill(skill).await?;
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // ========================================================================
    // SKILL 3: Debugging Rust Compilation Issues
    // ========================================================================
    println!("📚 Teaching Skill 3: Debugging Rust Compilation Issues");
    println!("─────────────────────────────────────────────────────");

    let skill3 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: "cargo build failing - need to verify code compiles before committing changes".to_string(),
        claude_reasoning: vec![
            "First run 'cargo check' instead of full build for faster feedback".to_string(),
            "Check returns compilation errors without doing codegen/optimization".to_string(),
            "Read error messages carefully - Rust errors are usually very helpful".to_string(),
            "Fix any compilation errors found".to_string(),
            "Once cargo check passes, run cargo build --release for production binary".to_string(),
            "Verify binary was created and check its size to ensure build completed".to_string(),
        ],
        solution: "Used cargo check to verify compilation (exit code 0 with only warnings), then ran release build successfully, confirming 2.0MB binary created".to_string(),
        context: HashMap::from([
            ("domain".to_string(), "debugging".to_string()),
            ("language".to_string(), "rust".to_string()),
            ("tool".to_string(), "cargo".to_string()),
        ]),
    };

    match StrategyExtractor::extract_skill(&skill3) {
        Ok(Some(skill)) => {
            println!("✅ Extracted skill: {}", skill.name);
            println!("   Domain: {:?}", skill.domain);
            println!("   Pattern: {}", skill.pattern);
            library.add_skill(skill).await?;
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // ========================================================================
    // SKILL 4: Git Workflow for Feature Development
    // ========================================================================
    println!("📚 Teaching Skill 4: Git Workflow for Feature Development");
    println!("─────────────────────────────────────────────────────");

    let skill4 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: "Need to commit and push new feature implementation with clear history".to_string(),
        claude_reasoning: vec![
            "Stage specific files with 'git add' for logical commits".to_string(),
            "Review changes with 'git diff --cached' before committing".to_string(),
            "Write descriptive commit messages with context and impact".to_string(),
            "Include co-authorship attribution for pair programming or AI assistance".to_string(),
            "Use heredoc syntax for multi-line commit messages in scripts".to_string(),
            "Push to feature branch, not main directly".to_string(),
        ],
        solution: "Created two logical commits: one for code changes, one for documentation. Each with detailed messages and co-authorship. Pushed to feature branch successfully.".to_string(),
        context: HashMap::from([
            ("domain".to_string(), "general".to_string()),
            ("tool".to_string(), "git".to_string()),
            ("pattern".to_string(), "version_control".to_string()),
        ]),
    };

    match StrategyExtractor::extract_skill(&skill4) {
        Ok(Some(skill)) => {
            println!("✅ Extracted skill: {}", skill.name);
            println!("   Domain: {:?}", skill.domain);
            library.add_skill(skill).await?;
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // ========================================================================
    // Display Library Statistics
    // ========================================================================
    println!("\n📊 Icarus Skill Library Statistics");
    println!("═══════════════════════════════════════════════════════");

    let stats = library.get_statistics().await;

    println!("Total Skills: {}", stats.get("total_skills").unwrap());
    println!("Average Success Rate: {:.2}",
        stats.get("avg_success_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    );

    println!("\nSkills by Domain:");
    if let Some(domain_counts) = stats.get("domain_counts") {
        if let Some(obj) = domain_counts.as_object() {
            for (domain, count) in obj {
                println!("  - {}: {}", domain, count);
            }
        }
    }

    println!("\n🎓 Knowledge Distillation Complete!");
    println!("Icarus has learned {} new skills from this session.",
        stats.get("total_skills").unwrap()
    );

    Ok(())
}
