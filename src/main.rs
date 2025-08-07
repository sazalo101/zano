use anyhow::Result;
use clap::{Arg, Command};
use std::path::Path;
use tokio;

mod parser;
mod runtime;
mod package;

use parser::lexer::Lexer;
use parser::Parser;
use runtime::ZanoRuntime;
use package::PackageManager;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("zano")
        .version("0.1.0")
        .about("A Node.js-like backend runtime built in Rust")
        .arg(
            Arg::new("file")
                .help("The Zano script file to run")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("eval")
                .short('e')
                .long("eval")
                .value_name("CODE")
                .help("Evaluate code directly"),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Start interactive REPL")
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("init")
                .about("Initialize a new Zano project with package.json")
        )
        .subcommand(
            Command::new("install")
                .about("Install dependencies")
                .arg(
                    Arg::new("package")
                        .help("Package name to install")
                        .required(false)
                        .index(1),
                )
        )
        .subcommand(
            Command::new("run")
                .about("Run a script from package.json")
                .arg(
                    Arg::new("script")
                        .help("Script name to run")
                        .required(true)
                        .index(1),
                )
        )
        .get_matches();

    // Handle package management subcommands first
    match matches.subcommand() {
        Some(("init", _)) => {
            let pkg_manager = PackageManager::new(".");
            pkg_manager.init().await?;
            return Ok(());
        }
        Some(("install", sub_matches)) => {
            let pkg_manager = PackageManager::new(".");
            let package_name = sub_matches.get_one::<String>("package").cloned();
            pkg_manager.install(package_name).await?;
            return Ok(());
        }
        Some(("run", sub_matches)) => {
            let pkg_manager = PackageManager::new(".");
            let script_name = sub_matches.get_one::<String>("script").unwrap();
            pkg_manager.run_script(script_name).await?;
            return Ok(());
        }
        _ => {}
    }

    let runtime = ZanoRuntime::new().await;

    if matches.get_flag("interactive") {
        run_repl(runtime).await?;
    } else if let Some(code) = matches.get_one::<String>("eval") {
        execute_code(&runtime, code).await?;
    } else if let Some(file_path) = matches.get_one::<String>("file") {
        run_file(&runtime, file_path).await?;
    } else {
        println!("Zano v0.1.0 - A Node.js-like runtime in Rust");
        println!("Usage:");
        println!("  zano <file.zn>     # Run a Zano script");
        println!("  zano -e \"<code>\"   # Evaluate code directly");
        println!("  zano -i            # Start interactive REPL");
        println!("  zano init          # Initialize new project");
        println!("  zano install       # Install dependencies");
        println!("  zano run <script>  # Run npm script");
    }

    Ok(())
}

async fn run_file(runtime: &ZanoRuntime, file_path: &str) -> Result<()> {
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    let source = tokio::fs::read_to_string(file_path).await?;
    execute_code(runtime, &source).await
}

async fn execute_code(runtime: &ZanoRuntime, source: &str) -> Result<()> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    let result = runtime.execute(statements).await?;
    
    // Only print result if it's not undefined (like Node.js REPL)
    match result {
        parser::ZanoValue::Undefined => {},
        _ => println!("{:?}", result),
    }

    Ok(())
}

async fn run_repl(runtime: ZanoRuntime) -> Result<()> {
    println!("Zano REPL v0.1.0");
    println!("Type .exit to quit");

    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                
                if input == ".exit" {
                    break;
                }
                
                if input.is_empty() {
                    continue;
                }

                match execute_code(&runtime, input).await {
                    Ok(_) => {},
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}
