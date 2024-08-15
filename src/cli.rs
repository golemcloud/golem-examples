use clap::*;

use crate::model::{ComponentName, ExampleName, GuestLanguage, GuestLanguageTier, PackageName};

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct NameOrLanguage {
    /// Name of the example to use
    #[arg(short, long, group = "ex")]
    pub example: Option<ExampleName>,

    /// Language to use for it's default example
    #[arg(short, long, alias = "lang", group = "ex")]
    pub language: Option<GuestLanguage>,
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

    // Generates and builds the examples components
    #[command()]
    TestExamples {
        // Filter examples by name, checks if the example name contains the filter string
        #[arg(short, long)]
        filter: Option<String>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, rename_all = "kebab-case")]
pub struct GolemCommand {
    #[command(subcommand)]
    pub command: Command,
}
