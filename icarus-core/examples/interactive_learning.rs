//! Interactive Learning Demo - Watch Icarus Learn in Real-Time
//!
//! This example provides detailed, real-time output of the knowledge
//! distillation process so you can see exactly what's happening.

use icarus_core::learning::{Interaction, SkillLibrary, StrategyExtractor};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn print_slow(text: &str, delay_ms: u64) {
    print!("{}", text);
    io::stdout().flush().unwrap();
    pause(delay_ms);
}

fn print_header(title: &str) {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║ {:<62} ║", title);
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    pause(500);
}

fn print_section(icon: &str, title: &str) {
    println!("\n{} {}", icon, title);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    pause(300);
}

fn print_step(step_num: usize, description: &str) {
    print_slow(&format!("  [{}] ", step_num), 100);
    println!("{}", description);
    pause(200);
}

fn print_detail(key: &str, value: &str) {
    println!("      • {}: {}", key, value);
    pause(150);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    print_header("🧠 ICARUS KNOWLEDGE DISTILLATION - LIVE DEMO");

    println!("Welcome to the Interactive Learning Demo!");
    println!("Watch as Icarus learns skills from Claude Code in real-time.\n");
    pause(1000);

    print_section("📚", "Initializing Skill Library");
    print_slow("Creating empty skill library... ", 500);
    let library = SkillLibrary::new();
    println!("✅ READY\n");
    pause(500);

    // ========================================================================
    // SKILL 1: Advanced Debugging Workflow
    // ========================================================================
    print_header("TEACHING SESSION #1: Advanced Debugging Workflow");

    print_section("📖", "Problem Context");
    let problem1 = "Production bug: API returning 500 errors intermittently for user authentication endpoint";
    println!("  Problem: {}", problem1);
    pause(800);

    print_section("🤔", "Claude's Reasoning Process");
    let reasoning1 = vec![
        "Check application logs for stack traces and error patterns".to_string(),
        "Reproduce the issue locally with the same user data".to_string(),
        "Add detailed logging around the authentication logic".to_string(),
        "Identify that the error occurs when JWT token is expired but cache isn't cleared".to_string(),
        "Implement proper cache invalidation on token expiry".to_string(),
        "Add monitoring alerts for 500 errors on auth endpoint".to_string(),
        "Deploy fix and verify error rate drops to zero".to_string(),
    ];

    for (i, step) in reasoning1.iter().enumerate() {
        print_step(i + 1, step);
    }
    pause(500);

    print_section("✅", "Solution Implemented");
    let solution1 = "Fixed intermittent 500 errors by adding cache invalidation on JWT expiry and implementing proper error monitoring";
    println!("  {}", solution1);
    pause(800);

    print_section("🔬", "Extracting Knowledge");
    print_slow("  Creating interaction record... ", 400);

    let interaction1 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: problem1.to_string(),
        claude_reasoning: reasoning1,
        solution: solution1.to_string(),
        context: HashMap::from([
            ("domain".to_string(), "debugging".to_string()),
            ("severity".to_string(), "production".to_string()),
            ("component".to_string(), "authentication".to_string()),
        ]),
    };
    println!("✅");
    pause(300);

    print_slow("  Analyzing reasoning patterns... ", 600);
    println!("✅");
    pause(300);

    print_slow("  Extracting reusable strategy... ", 700);
    match StrategyExtractor::extract_skill(&interaction1) {
        Ok(Some(skill)) => {
            println!("✅\n");
            pause(300);

            print_section("🎓", "Skill Learned!");
            print_detail("Skill ID", &skill.id);
            print_detail("Skill Name", &skill.name);
            print_detail("Domain", &format!("{:?}", skill.domain));
            print_detail("Pattern", &skill.pattern);
            print_detail("Steps Extracted", &skill.steps.len().to_string());
            print_detail("Heuristics", &skill.heuristics.len().to_string());
            print_detail("Initial Success Rate", &format!("{:.2}", skill.success_rate));

            pause(500);
            print_slow("\n  Storing in library... ", 400);
            library.add_skill(skill).await?;
            println!("✅ STORED");
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    pause(1000);

    // ========================================================================
    // SKILL 2: Performance Optimization
    // ========================================================================
    print_header("TEACHING SESSION #2: Performance Optimization");

    print_section("📖", "Problem Context");
    let problem2 = "Dashboard loading time increased from 2s to 15s after adding real-time data updates";
    println!("  Problem: {}", problem2);
    pause(800);

    print_section("🤔", "Claude's Reasoning Process");
    let reasoning2 = vec![
        "Profile the application to identify bottlenecks".to_string(),
        "Discover N+1 query problem: fetching related data in a loop".to_string(),
        "Implement database query optimization with JOIN instead of multiple queries".to_string(),
        "Add caching layer for frequently accessed data".to_string(),
        "Implement pagination to limit initial data load".to_string(),
        "Add lazy loading for non-critical dashboard widgets".to_string(),
        "Verify load time reduced to 2.5s with profiling tools".to_string(),
    ];

    for (i, step) in reasoning2.iter().enumerate() {
        print_step(i + 1, step);
    }
    pause(500);

    print_section("✅", "Solution Implemented");
    let solution2 = "Optimized dashboard from 15s to 2.5s by fixing N+1 queries, adding caching, and implementing pagination";
    println!("  {}", solution2);
    pause(800);

    print_section("🔬", "Extracting Knowledge");
    print_slow("  Creating interaction record... ", 400);

    let interaction2 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: problem2.to_string(),
        claude_reasoning: reasoning2,
        solution: solution2.to_string(),
        context: HashMap::from([
            ("domain".to_string(), "performance".to_string()),
            ("metric".to_string(), "load_time".to_string()),
            ("improvement".to_string(), "83%".to_string()),
        ]),
    };
    println!("✅");
    pause(300);

    print_slow("  Analyzing reasoning patterns... ", 600);
    println!("✅");
    pause(300);

    print_slow("  Extracting reusable strategy... ", 700);
    match StrategyExtractor::extract_skill(&interaction2) {
        Ok(Some(skill)) => {
            println!("✅\n");
            pause(300);

            print_section("🎓", "Skill Learned!");
            print_detail("Skill ID", &skill.id);
            print_detail("Skill Name", &skill.name);
            print_detail("Domain", &format!("{:?}", skill.domain));
            print_detail("Pattern", &skill.pattern);
            print_detail("Steps Extracted", &skill.steps.len().to_string());

            pause(500);
            print_slow("\n  Storing in library... ", 400);
            library.add_skill(skill).await?;
            println!("✅ STORED");
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    pause(1000);

    // ========================================================================
    // SKILL 3: Code Refactoring
    // ========================================================================
    print_header("TEACHING SESSION #3: Code Refactoring");

    print_section("📖", "Problem Context");
    let problem3 = "Legacy payment processing module has 800-line function with complex nested conditionals";
    println!("  Problem: {}", problem3);
    pause(800);

    print_section("🤔", "Claude's Reasoning Process");
    let reasoning3 = vec![
        "Analyze the function to understand all code paths and responsibilities".to_string(),
        "Identify 5 distinct concerns: validation, calculation, external API calls, database updates, notifications".to_string(),
        "Extract validation logic into PaymentValidator class with clear interface".to_string(),
        "Extract calculation into PaymentCalculator with testable methods".to_string(),
        "Extract API integration into PaymentGateway adapter".to_string(),
        "Extract database operations into PaymentRepository".to_string(),
        "Create PaymentOrchestrator to coordinate these components".to_string(),
        "Add comprehensive unit tests for each extracted component".to_string(),
        "Refactor original function to use orchestrator - now 50 lines".to_string(),
    ];

    for (i, step) in reasoning3.iter().enumerate() {
        print_step(i + 1, step);
    }
    pause(500);

    print_section("✅", "Solution Implemented");
    let solution3 = "Refactored 800-line function into 5 focused classes with single responsibilities, reduced to 50-line orchestrator";
    println!("  {}", solution3);
    pause(800);

    print_section("🔬", "Extracting Knowledge");
    print_slow("  Creating interaction record... ", 400);

    let interaction3 = Interaction {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        problem: problem3.to_string(),
        claude_reasoning: reasoning3,
        solution: solution3.to_string(),
        context: HashMap::from([
            ("domain".to_string(), "refactoring".to_string()),
            ("principle".to_string(), "single_responsibility".to_string()),
            ("before_lines".to_string(), "800".to_string()),
            ("after_lines".to_string(), "50".to_string()),
        ]),
    };
    println!("✅");
    pause(300);

    print_slow("  Analyzing reasoning patterns... ", 600);
    println!("✅");
    pause(300);

    print_slow("  Extracting reusable strategy... ", 700);
    match StrategyExtractor::extract_skill(&interaction3) {
        Ok(Some(skill)) => {
            println!("✅\n");
            pause(300);

            print_section("🎓", "Skill Learned!");
            print_detail("Skill ID", &skill.id);
            print_detail("Skill Name", &skill.name);
            print_detail("Domain", &format!("{:?}", skill.domain));
            print_detail("Pattern", &skill.pattern);
            print_detail("Steps Extracted", &skill.steps.len().to_string());
            print_detail("Heuristics", &skill.heuristics.len().to_string());

            pause(500);
            print_slow("\n  Storing in library... ", 400);
            library.add_skill(skill).await?;
            println!("✅ STORED");
        }
        Ok(None) => println!("❌ Could not extract skill"),
        Err(e) => println!("❌ Error: {}", e),
    }
    pause(1000);

    // ========================================================================
    // Final Statistics
    // ========================================================================
    print_header("📊 FINAL SKILL LIBRARY STATISTICS");

    let stats = library.get_statistics().await;

    pause(300);
    println!("╭─────────────────────────────────────────────────────╮");
    println!("│                  Library Summary                    │");
    println!("├─────────────────────────────────────────────────────┤");

    if let Some(total) = stats.get("total_skills") {
        println!("│  Total Skills Learned: {:<28} │", total);
    }

    if let Some(avg) = stats.get("avg_success_rate").and_then(|v| v.as_f64()) {
        println!("│  Average Success Rate: {:<28.2} │", avg);
    }

    println!("├─────────────────────────────────────────────────────┤");
    println!("│              Skills by Domain                       │");
    println!("├─────────────────────────────────────────────────────┤");

    if let Some(domain_counts) = stats.get("domain_counts") {
        if let Some(obj) = domain_counts.as_object() {
            for (domain, count) in obj {
                println!("│  {:<20} {:<28} │", domain, count);
            }
        }
    }

    println!("╰─────────────────────────────────────────────────────╯\n");
    pause(500);

    print_header("🎓 KNOWLEDGE DISTILLATION COMPLETE");

    println!("✨ Success! Icarus has learned new problem-solving strategies.");
    println!("📚 These skills are now available for application to similar problems.");
    println!("📈 Success rates will improve as skills are applied and validated.\n");

    pause(500);
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Icarus is ready to apply learned knowledge to new challenges!");
    println!("═══════════════════════════════════════════════════════════════\n");

    Ok(())
}
