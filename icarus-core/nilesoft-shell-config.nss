// ============================================
// Nilesoft Shell Configuration
// GitHub Repository Creator Context Menu
// ============================================

// Add this configuration to your Nilesoft Shell config file
// Default location: C:\Program Files\Nilesoft Shell\shell.nss

// Add to the shell menu block for directories
menu(mode="multiple" type="dir" where=sel.count>0 title="GitHub Repository Creator" image=inherit)
{
	item(title="Create Public Repo" image=icon.github cmd='cmd.exe' args='/c "@ECHO OFF & SET SCRIPT_PATH=%USERPROFILE%\scripts\create-github-repo.bat & IF EXIST "!SCRIPT_PATH!" (start "" cmd /c ""!SCRIPT_PATH!"" "@sel.path" public) ELSE (echo Script not found at !SCRIPT_PATH! & pause)"')
	item(title="Create Private Repo" image=icon.github cmd='cmd.exe' args='/c "@ECHO OFF & SET SCRIPT_PATH=%USERPROFILE%\scripts\create-github-repo.bat & IF EXIST "!SCRIPT_PATH!" (start "" cmd /c ""!SCRIPT_PATH!"" "@sel.path" private) ELSE (echo Script not found at !SCRIPT_PATH! & pause)"')
	separator
	item(title="Create Repo (Interactive)" image=icon.github cmd='cmd.exe' args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path""')
}

// Alternative simpler version (single menu item with interactive prompt)
/*
item(where=sel.count>0 type="dir" title="Create GitHub Repo" image=icon.github
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path""')
*/

// For background context menu (when right-clicking empty space in folder)
/*
menu(mode="multiple" type="back" title="GitHub Repository Creator" image=inherit)
{
	item(title="Create Repo Here" image=icon.github
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.dir""')
}
*/
