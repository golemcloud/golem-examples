use derive_more::FromStr;
use fancy_regex::{Match, Regex};
use inflector::Inflector;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, FromStr, Serialize, Deserialize)]
pub struct TemplateName(String);

static TEMPLATE_NAME_SPLIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("(?=[A-Z\\-_])").unwrap());

impl TemplateName {
    pub fn new(name: impl AsRef<str>) -> TemplateName {
        TemplateName(name.as_ref().to_string())
    }

    pub fn as_string(&self) -> &str {
        &self.0
    }

    pub fn parts(&self) -> Vec<String> {
        let matches: Vec<Result<Match, fancy_regex::Error>> =
            TEMPLATE_NAME_SPLIT_REGEX.find_iter(&self.0).collect();
        let mut parts: Vec<&str> = vec![];
        let mut last = 0;
        for m in matches.into_iter().flatten() {
            let part = &self.0[last..m.start()];
            if !part.is_empty() {
                parts.push(part);
            }
            last = m.end();
        }
        parts.push(&self.0[last..]);

        let mut result: Vec<String> = Vec::with_capacity(parts.len());
        for part in parts {
            let s = part.to_lowercase();
            let s = s.strip_prefix('-').unwrap_or(&s);
            let s = s.strip_prefix('_').unwrap_or(s);
            result.push(s.to_string());
        }
        result
    }

    pub fn to_kebab_case(&self) -> String {
        self.parts().join("-")
    }

    pub fn to_snake_case(&self) -> String {
        self.parts().join("_")
    }

    pub fn to_pascal_case(&self) -> String {
        self.parts().iter().map(|s| s.to_title_case()).collect()
    }

    pub fn to_camel_case(&self) -> String {
        self.to_pascal_case().to_camel_case()
    }
}

impl fmt::Display for TemplateName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, Serialize, Deserialize)]
pub enum GuestLanguage {
    Rust,
    Go,
    C,
    Zig,
    JavaScript,
    CSharp,
    Swift,
    Grain,
    Python,
    Scala2,
}

impl GuestLanguage {
    pub fn from_string(s: impl AsRef<str>) -> Option<GuestLanguage> {
        match s.as_ref().to_lowercase().as_str() {
            "rust" => Some(GuestLanguage::Rust),
            "go" => Some(GuestLanguage::Go),
            "c" | "c++" | "cpp" => Some(GuestLanguage::C),
            "zig" => Some(GuestLanguage::Zig),
            "js" | "javascript" => Some(GuestLanguage::JavaScript),
            "c#" | "cs" | "csharp" => Some(GuestLanguage::CSharp),
            "swift" => Some(GuestLanguage::Swift),
            "grain" => Some(GuestLanguage::Grain),
            "py" | "python" => Some(GuestLanguage::Python),
            "scala2" => Some(GuestLanguage::Scala2),
            _ => None,
        }
    }

    pub fn tier(&self) -> GuestLanguageTier {
        match self {
            GuestLanguage::Rust => GuestLanguageTier::Tier2,
            GuestLanguage::Go => GuestLanguageTier::Tier2,
            GuestLanguage::C => GuestLanguageTier::Tier2,
            GuestLanguage::Zig => GuestLanguageTier::Tier3,
            GuestLanguage::JavaScript => GuestLanguageTier::Tier2,
            GuestLanguage::CSharp => GuestLanguageTier::Tier4,
            GuestLanguage::Swift => GuestLanguageTier::Tier3,
            GuestLanguage::Grain => GuestLanguageTier::Tier3,
            GuestLanguage::Python => GuestLanguageTier::Tier2,
            GuestLanguage::Scala2 => GuestLanguageTier::Tier2,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            GuestLanguage::Rust => "Rust",
            GuestLanguage::Go => "Go",
            GuestLanguage::C => "C",
            GuestLanguage::Zig => "Zig",
            GuestLanguage::JavaScript => "JavaScript",
            GuestLanguage::CSharp => "C#",
            GuestLanguage::Swift => "Swift",
            GuestLanguage::Grain => "Grain",
            GuestLanguage::Python => "Python",
            GuestLanguage::Scala2 => "Scala 2",
        }
    }
}

impl fmt::Display for GuestLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for GuestLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GuestLanguage::from_string(s).ok_or({
            let all = GuestLanguage::iter()
                .map(|x| format!("\"{x}\""))
                .collect::<Vec<String>>()
                .join(", ");
            format!("Unknown guest language: {s}. Expected one of {all}")
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, Serialize, Deserialize)]
pub enum GuestLanguageTier {
    Tier1,
    Tier2,
    Tier3,
    Tier4,
}

impl GuestLanguageTier {
    pub fn from_string(s: impl AsRef<str>) -> Option<GuestLanguageTier> {
        match s.as_ref().to_lowercase().as_str() {
            "tier1" | "1" => Some(GuestLanguageTier::Tier1),
            "tier2" | "2" => Some(GuestLanguageTier::Tier2),
            "tier3" | "3" => Some(GuestLanguageTier::Tier3),
            "tier4" | "4" => Some(GuestLanguageTier::Tier4),
            _ => None,
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            GuestLanguageTier::Tier1 => 1,
            GuestLanguageTier::Tier2 => 2,
            GuestLanguageTier::Tier3 => 3,
            GuestLanguageTier::Tier4 => 4,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            GuestLanguageTier::Tier1 => "tier1",
            GuestLanguageTier::Tier2 => "tier2",
            GuestLanguageTier::Tier3 => "tier3",
            GuestLanguageTier::Tier4 => "tier4",
        }
    }
}

impl fmt::Display for GuestLanguageTier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for GuestLanguageTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GuestLanguageTier::from_string(s).ok_or(format!("Unexpected guest language tier {s}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageName((String, String));

impl PackageName {
    pub fn from_string(s: impl AsRef<str>) -> Option<PackageName> {
        let parts: Vec<&str> = s.as_ref().split(':').collect();
        match parts.as_slice() {
            &[n1, n2] => Some(PackageName((n1.to_string(), n2.to_string()))),
            _ => None,
        }
    }

    pub fn to_pascal_case(&self) -> String {
        format!("{}{}", self.0 .0.to_title_case(), self.0 .1.to_title_case())
    }

    pub fn to_snake_case(&self) -> String {
        format!("{}_{}", self.0 .0, self.0 .1)
    }

    pub fn to_string_with_double_colon(&self) -> String {
        format!("{}::{}", self.0 .0, self.0 .1)
    }

    pub fn to_string_with_colon(&self) -> String {
        format!("{}:{}", self.0 .0, self.0 .1)
    }

    pub fn to_string_with_slash(&self) -> String {
        format!("{}/{}", self.0 .0, self.0 .1)
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_colon())
    }
}

impl FromStr for PackageName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PackageName::from_string(s).ok_or(format!(
            "Unexpected package name {s}. Must be in 'pack:name' format"
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, FromStr, Serialize, Deserialize)]
pub struct ExampleName(String);

impl ExampleName {
    pub fn from_string(s: impl AsRef<str>) -> ExampleName {
        ExampleName(s.as_ref().to_string())
    }

    pub fn as_string(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExampleName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub name: ExampleName,
    pub language: GuestLanguage,
    pub description: String,
    pub example_path: PathBuf,
    pub instructions: String,
    pub adapter: Option<PathBuf>,
    pub wit_deps: Vec<PathBuf>,
    pub exclude: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleParameters {
    pub template_name: TemplateName,
    pub package_name: PackageName,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExampleMetadata {
    pub description: String,
    #[serde(rename = "requiresAdapter")]
    pub requires_adapter: Option<bool>,
    #[serde(rename = "requiresGolemHostWIT")]
    pub requires_golem_host_wit: Option<bool>,
    #[serde(rename = "requiresWASI")]
    pub requires_wasi: Option<bool>,
    pub exclude: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::model::{PackageName, TemplateName};
    use once_cell::sync::Lazy;

    static N1: Lazy<TemplateName> = Lazy::new(|| TemplateName::new("my-test-template"));
    static N2: Lazy<TemplateName> = Lazy::new(|| TemplateName::new("MyTestTemplate"));
    static N3: Lazy<TemplateName> = Lazy::new(|| TemplateName::new("myTestTemplate"));
    static N4: Lazy<TemplateName> = Lazy::new(|| TemplateName::new("my_test_template"));

    #[test]
    pub fn template_name_to_pascal_case() {
        assert_eq!(N1.to_pascal_case(), "MyTestTemplate");
        assert_eq!(N2.to_pascal_case(), "MyTestTemplate");
        assert_eq!(N3.to_pascal_case(), "MyTestTemplate");
        assert_eq!(N4.to_pascal_case(), "MyTestTemplate");
    }

    #[test]
    pub fn template_name_to_camel_case() {
        assert_eq!(N1.to_camel_case(), "myTestTemplate");
        assert_eq!(N2.to_camel_case(), "myTestTemplate");
        assert_eq!(N3.to_camel_case(), "myTestTemplate");
        assert_eq!(N4.to_camel_case(), "myTestTemplate");
    }

    #[test]
    pub fn template_name_to_snake_case() {
        assert_eq!(N1.to_snake_case(), "my_test_template");
        assert_eq!(N2.to_snake_case(), "my_test_template");
        assert_eq!(N3.to_snake_case(), "my_test_template");
        assert_eq!(N4.to_snake_case(), "my_test_template");
    }

    #[test]
    pub fn template_name_to_kebab_case() {
        assert_eq!(N1.to_kebab_case(), "my-test-template");
        assert_eq!(N2.to_kebab_case(), "my-test-template");
        assert_eq!(N3.to_kebab_case(), "my-test-template");
        assert_eq!(N4.to_kebab_case(), "my-test-template");
    }

    static P1: Lazy<PackageName> = Lazy::new(|| PackageName::from_string("foo:bar").unwrap());

    #[test]
    pub fn package_name_to_pascal_case() {
        assert_eq!(P1.to_pascal_case(), "FooBar");
    }
}
