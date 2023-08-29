use clap::*;
use golem_examples::model::{
    ExampleName, ExampleParameters, GuestLanguage, GuestLanguageTier, PackageName, TemplateName,
};
use golem_examples::*;
use std::env;

#[derive(Subcommand, Debug)]
#[command()]
enum Command {
    #[command()]
    New {
        #[arg(short, long)]
        example: ExampleName,

        #[arg(short, long)]
        template_name: TemplateName,

        #[arg(short, long)]
        package_name: Option<PackageName>,
    },
    #[command()]
    ListTemplates {
        #[arg(short, long)]
        min_tier: Option<GuestLanguageTier>,

        #[arg(short, long)]
        language: Option<GuestLanguage>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, rename_all = "kebab-case")]
struct GolemCommand {
    #[command(subcommand)]
    command: Command,
}

pub fn main() {
    let command: GolemCommand = GolemCommand::parse();
    match &command.command {
        Command::New {
            example: example_name,
            template_name,
            package_name,
        } => {
            let examples = GolemExamples::list_all_examples();
            let example = examples
                .iter()
                .find(|example| example.name == *example_name);
            match example {
                Some(example) => {
                    let cwd = env::current_dir().expect("Failed to get current working directory");
                    match GolemExamples::instantiate(
                        example,
                        ExampleParameters {
                            template_name: template_name.clone(),
                            package_name: package_name
                                .clone()
                                .unwrap_or(PackageName::from_string("golem:template").unwrap()),
                            target_path: cwd,
                        },
                    ) {
                        Ok(instructions) => println!("{instructions}"),
                        Err(err) => eprintln!("Failed to instantiate template: {err}"),
                    }
                }
                None => {
                    eprintln!("Unknown template {example_name}. Use the list-templates command to see the available commands.");
                }
            }
        }
        Command::ListTemplates { min_tier, language } => {
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
    }
}
