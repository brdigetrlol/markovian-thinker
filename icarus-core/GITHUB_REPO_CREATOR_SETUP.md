# GitHub Repository Creator - Nilesoft Shell Integration

A Windows batch script that creates GitHub repositories from any folder via right-click context menu using Nilesoft Shell.

## Features

- ✅ Create GitHub repositories from any folder with a right-click
- ✅ Interactive or quick selection (public/private)
- ✅ Beautiful dual progress bars showing current operation and overall progress
- ✅ Automatic git initialization, file staging, and commit
- ✅ Automatic push to GitHub
- ✅ Color-coded output with error handling
- ✅ File counting and staging feedback

## Prerequisites

Before you begin, ensure you have the following installed:

### 1. Git
Download and install from: https://git-scm.com/download/win

```bash
# Verify installation
git --version
```

Configure git if not already done:
```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

### 2. GitHub CLI (gh)
Download and install from: https://cli.github.com/

```bash
# Verify installation
gh --version

# Authenticate with GitHub
gh auth login
```

Follow the prompts to authenticate with your GitHub account.

### 3. Nilesoft Shell
Download and install from: https://nilesoft.org/

Nilesoft Shell is a powerful Windows context menu customizer.

## Installation

### Step 1: Copy the Script

1. Create a directory for your scripts (if it doesn't exist):
   ```
   C:\Users\<YourUsername>\scripts\
   ```

2. Copy `create-github-repo.bat` to this directory:
   ```
   C:\Users\<YourUsername>\scripts\create-github-repo.bat
   ```

### Step 2: Configure Nilesoft Shell

1. Open Nilesoft Shell configuration file:
   ```
   C:\Program Files\Nilesoft Shell\shell.nss
   ```

2. Add one of the following configurations to the file:

#### Option A: Full Menu with Multiple Choices
```nss
// Add this inside the main shell menu block
menu(mode="multiple" type="dir" where=sel.count>0 title="GitHub Repository Creator" image=inherit)
{
	item(title="Create Public Repo" image=icon.github
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path""')

	item(title="Create Private Repo" image=icon.github
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path""')

	separator

	item(title="Create Repo (Interactive)" image=icon.github
	     cmd='cmd.exe'
	     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path""')
}
```

#### Option B: Simple Single Item (Recommended)
```nss
// Add this inside the main shell menu block
item(where=sel.count>0 type="dir" title="Create GitHub Repo" image=icon.github
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path""')
```

#### Option C: Background Context Menu (Right-click in empty space)
```nss
// Add this for background context menu
item(mode="multiple" type="back" title="Create GitHub Repo Here" image=icon.github
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.dir""')
```

3. Save the configuration file

4. Restart Nilesoft Shell or reload the configuration:
   - Right-click on desktop → Nilesoft Shell → Reload

### Step 3: Verify Installation

1. Navigate to any folder in Windows Explorer
2. Right-click on the folder
3. You should see "Create GitHub Repo" (or your configured menu item)
4. Click it to test!

## Usage

### Method 1: Interactive Mode (Recommended)

1. Right-click on any folder in Windows Explorer
2. Select **"Create GitHub Repo"** from the context menu
3. A command window will open
4. Choose repository visibility:
   - Enter `1` for Public
   - Enter `2` for Private
5. Watch the progress bars as the script:
   - Initializes git repository
   - Counts and adds files
   - Creates initial commit
   - Creates GitHub repository
   - Pushes to GitHub
6. Press any key when complete

### Method 2: Direct Mode (if configured)

1. Right-click on any folder
2. Select **"Create Public Repo"** or **"Create Private Repo"**
3. The script runs automatically with your chosen visibility

## How It Works

The script performs the following steps:

1. **Validation**
   - Checks for git and gh CLI installation
   - Verifies GitHub CLI authentication
   - Validates directory access

2. **Initialization**
   - Removes existing .git directory if present
   - Initializes new git repository with 'main' branch

3. **File Management**
   - Counts all files (excluding .git)
   - Stages all files for commit
   - Creates README.md if no files exist

4. **Repository Creation**
   - Creates initial commit
   - Creates GitHub repository via gh CLI
   - Pushes to GitHub

5. **Progress Display**
   - Shows current operation progress
   - Displays overall progress across all stages
   - Color-coded output for easy reading

## Troubleshooting

### "git command not found"
- Install Git from https://git-scm.com/download/win
- Ensure Git is added to your PATH
- Restart your terminal/command prompt

### "gh command not found"
- Install GitHub CLI from https://cli.github.com/
- Ensure it's added to your PATH
- Run `gh auth login` to authenticate

### "GitHub CLI is not authenticated"
```bash
gh auth login
```
Follow the prompts to authenticate.

### "Failed to create GitHub repository"
Possible causes:
- Repository name already exists on your GitHub account
- Network connectivity issues
- GitHub API rate limits
- Invalid authentication token

Check authentication:
```bash
gh auth status
```

### Script not appearing in context menu
1. Verify Nilesoft Shell is installed and running
2. Check the configuration file path
3. Ensure the script path in the config is correct
4. Reload Nilesoft Shell configuration

### Permission Issues
Run the script as Administrator if you encounter permission errors.

## Customization

### Change Script Location

If you want to use a different location, update the path in the Nilesoft Shell config:

```nss
args='/c "start "" cmd /k "C:\path\to\your\create-github-repo.bat" "@sel.path""'
```

### Customize Menu Appearance

Modify the Nilesoft Shell configuration:
- `title`: Change the menu item text
- `image`: Change the icon (use `icon.github`, `icon.git`, or custom icons)
- Add `sep=top` or `sep=bottom` for separators

### Modify Progress Bar Colors

Edit the ANSI color codes in the batch script:
```batch
set "GREEN=[92m"
set "CYAN=[96m"
set "YELLOW=[93m"
set "RED=[91m"
set "BLUE=[94m"
set "MAGENTA=[95m"
```

## Advanced Configuration

### Add to Multiple Context Menu Locations

You can add the menu item to:
- **Folders**: `type="dir"`
- **Background**: `type="back"` (empty space in folders)
- **Desktop**: `type="desktop"`
- **This PC**: `type="desktop"`

Example for all locations:
```nss
item(where=sel.count>0 type="dir|back" title="Create GitHub Repo"
     image=icon.github
     cmd='cmd.exe'
     args='/c "start "" cmd /k "%USERPROFILE%\scripts\create-github-repo.bat" "@sel.path||@sel.dir""')
```

### Keyboard Shortcuts

Add keyboard shortcuts in Nilesoft Shell config:
```nss
item(title="Create GitHub Repo" keys="ctrl+shift+g" ...)
```

## Files Included

- `create-github-repo.bat` - Main batch script
- `nilesoft-shell-config.nss` - Example Nilesoft Shell configuration
- `GITHUB_REPO_CREATOR_SETUP.md` - This file

## License

Free to use and modify for personal and commercial projects.

## Contributing

Feel free to submit issues and enhancement requests!

## Credits

- Uses [Git](https://git-scm.com/) for version control
- Uses [GitHub CLI](https://cli.github.com/) for GitHub integration
- Uses [Nilesoft Shell](https://nilesoft.org/) for context menu integration

## Support

For issues or questions:
1. Verify all prerequisites are installed
2. Check the troubleshooting section
3. Ensure proper authentication with GitHub
4. Check Nilesoft Shell configuration

---

**Happy coding! 🚀**
