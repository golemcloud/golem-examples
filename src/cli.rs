use clap::*;
use golem_examples::model::{
    ComponentName, ExampleName, ExampleParameters, GuestLanguage, GuestLanguageTier, PackageName,
};
use golem_examples::*;
use std::env;

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct NameOrLanguage {
    /// Name of the example to use
    #[arg(short, long, group = "ex")]
    example: Option<ExampleName>,

    /// Language to use for it's default example
    #[arg(short, long, alias = "lang", group = "ex")]
    language: Option<GuestLanguage>,
}

impl NameOrLanguage {
    /// Gets the selected example's name
    pub fn example_name(&self) -> ExampleName {
        self.example
            .clone()
            .unwrap_or(ExampleName::from_string(format!(
                "{}-default",
                self.language.clone().unwrap_or(GuestLanguage::Rust).id()
            )))
    }
}

#[derive(Subcommand, Debug)]
#[command()]
pub enum Command {
    /// Create a new Golem component from built-in examples
    #[command()]
    New {
        #[command(flatten)]
        name_or_language: NameOrLanguage,

        /// The package name of the generated component (in namespace:name format)
        #[arg(short, long)]
        package_name: Option<PackageName>,

        /// The new component's name
        component_name: ComponentName,
    },

    /// Lists the built-in examples available for creating new components
    #[command()]
    ListExamples {
        /// The minimum language tier to include in the list
        #[arg(short, long)]
        min_tier: Option<GuestLanguageTier>,

        /// Filter examples by a given guest language
        #[arg(short, long, alias = "lang")]
        language: Option<GuestLanguage>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, rename_all = "kebab-case")]
pub struct GolemCommand {
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
    }
}
