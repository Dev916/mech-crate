//! `mx docs` command - Portable Markdown to PDF conversion

use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::Args;
use console::style;

use mx_lib::{ensure_path, mech_crate_root};

/// Portable Markdown to PDF Compiler
#[derive(Args, Debug)]
pub struct DocsCommand {
    /// Markdown file (.md) or directory containing .md files
    input: Option<String>,

    /// Output directory for generated files
    #[arg(short, long)]
    output: Option<String>,

    /// Add prefix to output filenames
    #[arg(long)]
    prefix: Option<String>,

    /// Document author
    #[arg(long)]
    author: Option<String>,

    /// Document title
    #[arg(long)]
    title: Option<String>,

    /// Document subtitle
    #[arg(long)]
    subtitle: Option<String>,

    /// Mermaid theme: dark, light, forest, neutral
    #[arg(long, default_value = "light")]
    theme: String,

    /// Comma-separated file order for directories
    #[arg(long)]
    order: Option<String>,

    /// Only generate processed markdown
    #[arg(long)]
    markdown_only: bool,

    /// Only generate HTML (no PDF)
    #[arg(long)]
    html_only: bool,

    /// Don't scan subfolders (for directories)
    #[arg(long)]
    no_recursive: bool,

    /// Disable table of contents
    #[arg(long)]
    no_toc: bool,

    /// Show detailed progress
    #[arg(short, long)]
    verbose: bool,

    /// List available documents from docs.json config
    #[arg(long)]
    list: bool,

    /// Compile all documents from docs.json config
    #[arg(long)]
    all: bool,

    /// Compile a specific document by key name from docs.json config
    #[arg(long)]
    doc: Option<String>,

    /// Path to docs.json config or directory containing it
    #[arg(short, long)]
    config: Option<String>,

    /// Logo image for cover page (PNG, JPG, SVG)
    #[arg(long)]
    logo: Option<String>,

    /// Company name for cover page
    #[arg(long)]
    company_name: Option<String>,

    /// Disable logo on cover page
    #[arg(long)]
    no_logo: bool,
}

impl DocsCommand {
    pub async fn run(&self) -> Result<()> {
        // Check for Node.js
        self.check_node()?;

        // Find scripts/docs directory
        let root = mech_crate_root().context(
            "Could not find MechCrate root. Set MECH_CRATE_ROOT environment variable.",
        )?;
        let docs_script_dir = root.join("scripts").join("docs");

        if !docs_script_dir.exists() {
            anyhow::bail!(
                "Documentation scripts not found at: {}\nEnsure MechCrate is properly installed.",
                docs_script_dir.display()
            );
        }

        // Ensure npm dependencies are installed
        if !docs_script_dir.join("node_modules").exists() {
            println!(
                "{} Installing documentation dependencies (first run)...",
                style("→").cyan()
            );
            let status = Command::new("npm")
                .args(["install", "--silent"])
                .current_dir(&docs_script_dir)
                .env("PATH", ensure_path())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run 'npm install'")?;

            if !status.success() {
                anyhow::bail!("Failed to install documentation dependencies");
            }
            println!("{} Dependencies installed", style("✓").green().bold());
        }

        // Build arguments for the TypeScript compiler
        let mut args: Vec<String> = vec!["tsx".into(), "compile.ts".into()];

        // Handle special modes first
        if self.list {
            args.push("--list".into());
            if let Some(ref config) = self.config {
                args.push(format!("--config={}", config));
            }
            return self.run_npx(&docs_script_dir, &args);
        }

        if self.all {
            args.push("--all".into());
            self.add_common_args(&mut args)?;
            return self.run_npx(&docs_script_dir, &args);
        }

        if let Some(ref doc) = self.doc {
            args.push(format!("--doc={}", doc));
            self.add_common_args(&mut args)?;
            return self.run_npx(&docs_script_dir, &args);
        }

        // Handle positional input
        if let Some(ref input) = self.input {
            let input_path = if std::path::Path::new(input).is_absolute() {
                input.clone()
            } else {
                std::env::current_dir()
                    .context("Failed to get current directory")?
                    .join(input)
                    .to_string_lossy()
                    .to_string()
            };
            args.push(input_path);
            self.add_common_args(&mut args)?;
            return self.run_npx(&docs_script_dir, &args);
        }

        // No action specified — show help
        self.print_help();
        Ok(())
    }

    fn add_common_args(&self, args: &mut Vec<String>) -> Result<()> {
        if let Some(ref output) = self.output {
            let abs = std::fs::canonicalize(output).unwrap_or_else(|_| output.into());
            args.push(format!("--output={}", abs.display()));
        }
        if let Some(ref prefix) = self.prefix {
            args.push(format!("--prefix={}", prefix));
        }
        if let Some(ref author) = self.author {
            args.push(format!("--author={}", author));
        }
        if let Some(ref title) = self.title {
            args.push(format!("--title={}", title));
        }
        if let Some(ref subtitle) = self.subtitle {
            args.push(format!("--subtitle={}", subtitle));
        }
        args.push(format!("--theme={}", self.theme));
        if let Some(ref order) = self.order {
            args.push(format!("--order={}", order));
        }
        if let Some(ref config) = self.config {
            args.push(format!("--config={}", config));
        }
        if let Some(ref logo) = self.logo {
            let abs = std::fs::canonicalize(logo).unwrap_or_else(|_| logo.into());
            args.push(format!("--logo={}", abs.display()));
        }
        if let Some(ref company_name) = self.company_name {
            args.push(format!("--company-name={}", company_name));
        }
        if self.no_logo {
            args.push("--no-logo".into());
        }
        if self.verbose {
            args.push("--verbose".into());
        }
        if self.markdown_only {
            args.push("--markdown-only".into());
        }
        if self.html_only {
            args.push("--html-only".into());
        }
        if self.no_recursive {
            args.push("--no-recursive".into());
        }
        if self.no_toc {
            args.push("--no-toc".into());
        }
        Ok(())
    }

    fn run_npx(&self, docs_script_dir: &std::path::Path, args: &[String]) -> Result<()> {
        if self.verbose {
            println!(
                "{} Running: npx {}",
                style("→").cyan(),
                args.join(" ")
            );
        }

        let status = Command::new("npx")
            .args(args)
            .current_dir(docs_script_dir)
            .env("PATH", ensure_path())
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run 'npx tsx compile.ts'")?;

        if !status.success() {
            anyhow::bail!("Documentation compilation failed");
        }

        Ok(())
    }

    fn check_node(&self) -> Result<()> {
        // Check for Node.js
        which::which("node").context(
            "Node.js not found.\n  Install with: brew install node",
        )?;

        // Check Node.js version
        let output = Command::new("node")
            .args(["-v"])
            .output()
            .context("Failed to get Node.js version")?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let major: u32 = version_str
            .trim()
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        if major < 18 {
            anyhow::bail!(
                "Node.js 18+ required. Current version: {}",
                version_str.trim()
            );
        }

        // Check for npm
        which::which("npm").context(
            "npm not found. Install Node.js to get npm.",
        )?;

        Ok(())
    }

    fn print_help(&self) {
        println!();
        println!("{} - Portable Markdown to PDF Compiler", style("mx docs").bold());
        println!();
        println!("  Just needs Node.js - no other system dependencies required!");
        println!("  PDF generation via bundled Chromium - always works!");
        println!();
        println!("{}", style("USAGE").bold());
        println!("    mx docs <input>              Convert file or folder to PDF");
        println!("    mx docs --all                Compile all docs from config");
        println!("    mx docs --doc=<name>         Compile specific doc from config");
        println!("    mx docs --list               List available documents");
        println!();
        println!("{}", style("ARGUMENTS").bold());
        println!("    <input>    Markdown file (.md) or directory containing .md files");
        println!();
        println!("{}", style("OPTIONS").bold());
        println!("    -o, --output <path>     Output directory for generated files");
        println!("    -c, --config <path>     Path to docs.json or directory containing it");
        println!("    --title <title>         Document title");
        println!("    --subtitle <subtitle>   Document subtitle");
        println!("    --author <author>       Document author");
        println!("    --prefix <string>       Add prefix to output filenames");
        println!("    --theme <theme>         Mermaid theme: dark, light, forest, neutral");
        println!("    --order <files>         Comma-separated file order for directories");
        println!("    --logo <path>           Logo image for cover page (PNG, JPG, SVG)");
        println!("    --company-name <name>   Company name for cover page");
        println!("    --no-logo               Disable logo on cover page");
        println!("    --markdown-only         Only generate processed markdown");
        println!("    --html-only             Only generate HTML (no PDF attempt)");
        println!("    --no-toc                Disable table of contents");
        println!("    --no-recursive          Don't scan subfolders (for directories)");
        println!("    -v, --verbose           Show detailed progress");
        println!("    -h, --help              Show this help");
        println!();
        println!("{}", style("CONFIG COMMANDS").bold());
        println!("    --all                   Compile all docs defined in docs.json");
        println!("    --doc=<name>            Compile a specific document by key name");
        println!("    --list                  List all available documents from config");
        println!();
        println!("{}", style("FRONTMATTER").bold());
        println!("    Documents can include YAML frontmatter for metadata:");
        println!();
        println!("    ---");
        println!("    title: My Document");
        println!("    subtitle: Optional Subtitle");
        println!("    author: Author Name");
        println!("    toc: true");
        println!("    ---");
        println!();
        println!("{}", style("OUTPUT").bold());
        println!("    artifacts/<name>/");
        println!("    ├── <name>.pdf      # PDF (always generated)");
        println!("    ├── <name>.html     # HTML version");
        println!("    ├── <name>.md       # Processed markdown");
        println!("    └── diagrams/       # Rendered Mermaid PNGs");
        println!();
        println!("{}", style("EXAMPLES").bold());
        println!();
        println!("    # Single file");
        println!("    mx docs docs/README.md");
        println!("    mx docs docs/spec.md -o artifacts/");
        println!();
        println!("    # Folder (all .md files)");
        println!("    mx docs docs/guides/");
        println!("    mx docs docs/api/ --title \"API Documentation\"");
        println!();
        println!("    # Config-based documents");
        println!("    mx docs --all                    # Compile all");
        println!("    mx docs --doc=whitepaper         # Compile specific");
        println!("    mx docs --list                   # List available");
        println!("    mx docs -c docs/myproject/ --list # Config from specific dir");
        println!();
        println!("{}", style("DEPENDENCIES").bold());
        println!("    Required: Node.js 18+ (npm)");
        println!();
        println!("    That's it! PDF generation uses bundled Chromium.");
        println!("    No Pandoc, LaTeX, or other system tools needed.");
        println!();
    }
}
