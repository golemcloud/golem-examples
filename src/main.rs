use std::env;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use colored::{ColoredString, Colorize};

use golem_examples::cli::*;
use golem_examples::model::*;
use golem_examples::{Examples, GolemExamples};

pub fn main() {
    let command: GolemCommand = GolemCommand::parse();
    match &command.command {
        Command::New {
            name_or_language,
            component_name,
            package_name,
        } => {
            let example_name = name_or_language.example_name();
            let examples = GolemExamples::list_all_examples();
            let example = examples.iter().find(|example| example.name == example_name);
            match example {
                Some(example) => {
                    let cwd = env::current_dir().expect("Failed to get current working directory");
                    match GolemExamples::instantiate(
                        example,
                        ExampleParameters {
                            component_name: component_name.clone(),
                            package_name: package_name
                                .clone()
                                .unwrap_or(PackageName::from_string("golem:component").unwrap()),
                            target_path: cwd,
                        },
                    ) {
                        Ok(instructions) => println!("{instructions}"),
                        Err(err) => eprintln!("Failed to instantiate example: {err}"),
                    }
                }
                None => {
                    eprintln!("Unknown example {example_name}. Use the list-examples command to see the available commands.");
                }
            }
        }
        Command::ListExamples { min_tier, language } => {
            GolemExamples::list_all_examples()
                .iter()
                .filter(|example| match language {
                    Some(language) => example.language == *language,
                    None => true,
                })
                .filter(|example| match min_tier {
                    Some(min_tier) => example.language.tier() <= *min_tier,
                    None => true,
                })
                .for_each(|example| println!("{:?}", example));
        }
        Command::TestExamples { filter } => {
            let results: Vec<(Example, Result<(), String>)> = GolemExamples::list_all_examples()
                .iter()
                .filter(|example| match filter {
                    Some(filter) => example.name.as_string().contains(filter),
                    None => true,
                })
                .map(|example| {
                    let result = test_example(example);
                    if let Err(err) = &result {
                        println!("{}", err.bright_red())
                    }
                    (example.clone(), result)
                })
                .collect();

            println!();
            for result in &results {
                println!(
                    "{}: {}",
                    result.0.name.to_string().bold(),
                    match &result.1 {
                        Ok(_) => "OK".bright_green(),
                        Err(err) =>
                            ColoredString::from(format!("{}\n{}", "Failed".bright_red(), err.red())),
                    }
                )
            }
            println!();

            if results.iter().any(|r| r.1.is_err()) {
                exit(1)
            }
        }
    }
}

fn test_example(example: &Example) -> Result<(), String> {
    println!();
    println!(
        "{} {}",
        "Generating and testing:".bold().bright_white(),
        example.name.to_string().blue()
    );

    let target_path = PathBuf::from("examples-test");
    let component_name = ComponentName::new(example.name.as_string().to_string() + "-comp");
    let package_name =
        PackageName::from_string("golem:component").ok_or("failed to create package name")?;
    let component_path = target_path.join(component_name.as_string());

    println!("Target path: {}", target_path.display().to_string().blue());
    println!("Component name: {}", component_name.as_string().blue());
    println!("Package name: {}", package_name.to_string().blue());
    println!(
        "Component path: {}",
        component_path.display().to_string().blue()
    );

    let run = |command: &str, args: Vec<&str>| -> Result<(), String> {
        let command_formatted = format!("{} {}", command, args.join(" "));
        let run_failed = |e| format!("{} failed: {}", command_formatted, e);

        println!(
            "Running {} in {}",
            command_formatted.blue(),
            component_path.display().to_string().blue()
        );
        let status = std::process::Command::new(command)
            .args(args.clone())
            .current_dir(&component_path)
            .status()
            .map_err(|e| run_failed(e.to_string()))?;

        match status.code() {
            Some(code) if code == 0 => Ok(()),
            Some(code) => Err(run_failed(format!("non-zero exit code: {}", code))),
            None => Err(run_failed("terminated".to_string())),
        }
    };

    if component_path.exists() {
        println!("Deleting {}", component_path.display().to_string().blue());
        std::fs::remove_dir_all(&component_path)
            .map_err(|e| format!("remove dir all failed: {}", e))?;
    }

    println!("Instantiating");
    let _ = GolemExamples::instantiate(
        example,
        ExampleParameters {
            component_name,
            package_name,
            target_path,
        },
    )
    .map_err(|e| format!("instantiate failed: {}", e.to_string()))?;

    match &example.language {
        GuestLanguage::Go => run("make", vec!["build"]),
        GuestLanguage::TypeScript => {
            run("npm", vec!["install"])?;
            run("npm", vec!["run", "componentize"])
        }
        other => return Err(format!("build not implemented for {}", other.name())),
    }
}
