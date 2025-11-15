// ============================================
// Nilesoft Shell Configuration
// Icarus MCP Server Integration
// ============================================

// Add this configuration to your Nilesoft Shell config file
// Default location: C:\Program Files\Nilesoft Shell\shell.nss

// ============================================
// OPTION 1: File/Directory Context Menu
// Launch Icarus MCP with a file or directory as context
// ============================================

menu(mode="multiple" type="file|dir" where=sel.count>0 title="Icarus AI" image=icon.ai)
{
	item(title="Analyze with Icarus MCP"
	     image=icon.analyze
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --context "@sel.path""')

	item(title="Start Icarus Autonomous Mode"
	     image=icon.robot
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-autonomous.exe" --workspace "@sel.path""')

	separator

	item(title="Open Icarus Config"
	     image=icon.settings
	     cmd='notepad.exe'
	     args='"%USERPROFILE%\.icarus\icarus-config.toml"')
}

// ============================================
// OPTION 2: Background Context Menu
// Launch Icarus in the current directory
// ============================================

menu(mode="multiple" type="back" title="Icarus AI" image=icon.ai)
{
	item(title="Start Icarus MCP Server Here"
	     image=icon.ai
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "cd /d @sel.dir && %USERPROFILE%\.icarus\bin\icarus-mcp.exe""')

	item(title="Start Autonomous Mode Here"
	     image=icon.robot
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "cd /d @sel.dir && %USERPROFILE%\.icarus\bin\icarus-autonomous.exe --workspace @sel.dir""')

	separator

	item(title="Initialize Icarus Workspace"
	     image=icon.folder_open
	     cmd='cmd.exe'
	     args='/c "cd /d @sel.dir && echo Initializing Icarus workspace... && %USERPROFILE%\.icarus\bin\icarus-mcp.exe --init && pause"')
}

// ============================================
// OPTION 3: Simple Quick Launch
// Single menu items without submenu
// ============================================

/*
item(where=sel.count>0 type="dir" title="Icarus: Analyze Directory" image=icon.ai
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --analyze "@sel.path""')

item(where=sel.count>0 type="file" title="Icarus: Analyze File" image=icon.ai
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --file "@sel.path""')
*/

// ============================================
// OPTION 4: Desktop/Send To Integration
// Add Icarus to "Send To" menu
// ============================================

/*
item(type="file|dir" title="Send to Icarus MCP" image=icon.ai
     mode="multiple"
     cmd='%USERPROFILE%\.icarus\bin\icarus-mcp.exe'
     args='--process "@sel.path"')
*/

// ============================================
// OPTION 5: Advanced - Conditional Menu
// Show different options based on file type
// ============================================

/*
// For code files
item(where=sel.count>0 type="file"
     where=sel.ext=="rs" || sel.ext=="py" || sel.ext=="js" || sel.ext=="ts"
     title="Icarus: Code Analysis"
     image=icon.code
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --code-analysis "@sel.path""')

// For text/markdown files
item(where=sel.count>0 type="file"
     where=sel.ext=="md" || sel.ext=="txt"
     title="Icarus: Document Analysis"
     image=icon.document
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --document "@sel.path""')

// For project directories (has Cargo.toml, package.json, etc.)
item(where=sel.count>0 type="dir"
     where=io.exists(sel.path + "\Cargo.toml") || io.exists(sel.path + "\package.json")
     title="Icarus: Analyze Project"
     image=icon.project
     cmd='cmd.exe'
     args='/c "start "" cmd /k "cd /d @sel.path && %USERPROFILE%\.icarus\bin\icarus-mcp.exe --project-analysis""')
*/

// ============================================
// OPTION 6: Quick Actions Submenu
// Multiple quick actions in one menu
// ============================================

/*
menu(mode="multiple" type="dir|file" where=sel.count>0 title="Icarus Quick Actions" image=icon.ai)
{
	item(title="Quick Analyze" image=icon.search
	     cmd='%USERPROFILE%\.icarus\bin\icarus-mcp.exe'
	     args='--quick "@sel.path"')

	item(title="Deep Analysis" image=icon.analyze
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\.icarus\bin\icarus-mcp.exe" --deep "@sel.path""')

	item(title="Generate Summary" image=icon.document
	     cmd='%USERPROFILE%\.icarus\bin\icarus-mcp.exe'
	     args='--summary "@sel.path" --output "@sel.dir\@sel.name-summary.md"')

	separator

	item(title="View Logs" image=icon.log
	     cmd='notepad.exe'
	     args='"%USERPROFILE%\.icarus\logs\icarus.log"')
}
*/

// ============================================
// OPTION 7: PowerShell Integration
// Launch with PowerShell for better scripting
// ============================================

/*
item(where=sel.count>0 type="dir" title="Icarus MCP (PowerShell)" image=icon.powershell
     cmd='powershell.exe'
     args='-NoExit -Command "cd \"@sel.path\"; & \"$env:USERPROFILE\.icarus\bin\icarus-mcp.exe\""')
*/

// ============================================
// Installation Instructions
// ============================================

/*
To install this configuration:

1. Build and deploy Icarus MCP Server:
   - Run: build-and-deploy.bat
   - This installs to: %USERPROFILE%\.icarus\bin\

2. Open Nilesoft Shell configuration:
   - Location: C:\Program Files\Nilesoft Shell\shell.nss
   - Or right-click desktop → Nilesoft Shell → Edit Config

3. Copy desired configuration blocks from this file

4. Save and reload Nilesoft Shell:
   - Right-click desktop → Nilesoft Shell → Reload
   - Or restart Windows Explorer

5. Test the integration:
   - Right-click any folder
   - Look for "Icarus AI" menu items

Note: Adjust paths if you installed Icarus to a different location
*/

// ============================================
// Troubleshooting
// ============================================

/*
If menu items don't appear:
1. Verify Icarus is installed: Check %USERPROFILE%\.icarus\bin\icarus-mcp.exe exists
2. Check Nilesoft Shell is running: Right-click desktop, look for Nilesoft menu
3. Reload configuration: Right-click desktop → Nilesoft Shell → Reload
4. Check syntax: Look for errors in Nilesoft Shell log

If Icarus doesn't launch:
1. Test manually: Open cmd and run %USERPROFILE%\.icarus\bin\icarus-mcp.exe
2. Check dependencies: Rust runtime libraries
3. Check PATH: Ensure .icarus\bin is accessible
4. Review logs: %USERPROFILE%\.icarus\logs\
*/
