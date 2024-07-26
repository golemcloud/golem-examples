use clap::*;
use golem_examples::model::{
    ComponentName, ExampleName, ExampleParameters, GuestLanguage, GuestLanguageTier, PackageName,
};
use golem_examples::*;
use std::env;


#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct NameOrLanguage {
    #[arg(short, long, group = "ex")]
    example: Option<ExampleName>,

    #[arg(short, long, alias = "lang", group = "ex")]
    language: Option<GuestLanguage>,
}

#[derive(Subcommand, Debug)]
#[command()]
enum Command {
    #[command()]
    New {
        #[command(flatten)]
        name_or_language: NameOrLanguage,

        #[arg(short, long)]
        package_name: Option<PackageName>,

        component_name: ComponentName,
    },
    #[command()]
    ListExamples {
        #[arg(short, long)]
        min_tier: Option<GuestLanguageTier>,

        #[arg(short, long, alias = "lang")]
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
            name_or_language,
            component_name,
            package_name,
        } => {
            let example_name = name_or_language.example.clone().unwrap_or(
                ExampleName::from_string(format!("{}-default", name_or_language.language.clone().unwrap_or(GuestLanguage::Rust).id()))
            );
            let examples = GolemExamples::list_all_examples();
            let example = examples
                .iter()
                .find(|example| example.name == example_name);
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
    }
}
