---
description: "Fully implements features from description to production-ready code with tests and documentation"
---

# Feature Implementation Command

You are in feature implementation mode. The user will describe a feature they want implemented.

Your task:

1. **Analyze the Request**
   - Understand the feature requirements
   - Identify which parts of the codebase need changes
   - Determine if it's Rust backend, TypeScript frontend, or both

2. **Search the Codebase**
   - Find relevant files using Glob and Grep
   - Understand existing patterns and architecture
   - Identify similar implementations to match style

3. **Plan the Implementation**
   - Create a step-by-step plan
   - List all files to create/modify
   - Identify potential edge cases

4. **Implement the Feature**
   - Write production-ready code matching existing style
   - Include comprehensive error handling
   - Add appropriate logging and tracing
   - Use async/await patterns correctly
   - Follow Rust best practices (for .rs files)
   - Use TypeScript strict mode (for .ts files)

5. **Add Tests**
   - Write unit tests
   - Add integration tests if needed
   - Ensure edge cases are covered
   - Use existing test patterns

6. **Update Documentation**
   - Update README if needed
   - Add code comments
   - Update API documentation

7. **Verify Everything**
   - Run tests: `cargo test` for Rust
   - Check compilation: `cargo check`
   - Run linter: `cargo clippy`
   - Format code: `cargo fmt`
   - Build TypeScript: `npm run build` if applicable

8. **Create Commit**
   - Write clear commit message following conventional commits
   - Include what was implemented and why

**Important Guidelines**:
- Match the existing code style exactly
- Use the same error handling patterns
- Follow the project's architectural decisions
- Don't skip tests - they're critical
- Make it production-ready, not a prototype

**Expected Output**:
- Complete, working implementation
- All tests passing
- Code properly formatted
- Clear commit with changes

Now, what feature would you like me to implement?
