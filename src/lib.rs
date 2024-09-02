use crate::model::{Example, ExampleMetadata, ExampleName, ExampleParameters, GuestLanguage};
use include_dir::{include_dir, Dir, DirEntry};
use std::collections::HashSet;
use std::convert::identity;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[cfg(feature = "cli")]
pub mod cli;
pub mod model;

pub trait Examples {
    fn list_all_examples() -> Vec<Example>;
    fn instantiate(example: &Example, parameters: &ExampleParameters) -> io::Result<String>;
    fn instructions(example: &Example, parameters: &ExampleParameters) -> String;
}

pub struct GolemExamples {}

static EXAMPLES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/examples");
static ADAPTERS: Dir<'_> = include_dir!("$OUT_DIR/golem-wit/adapters");
static WIT: Dir<'_> = include_dir!("$OUT_DIR/golem-wit/wit/deps");

impl Examples for GolemExamples {
    fn list_all_examples() -> Vec<Example> {
        let mut result: Vec<Example> = vec![];
        for entry in EXAMPLES.entries() {
            if let Some(lang_dir) = entry.as_dir() {
                let lang_dir_name = lang_dir.path().file_name().unwrap().to_str().unwrap();
                if let Some(lang) = GuestLanguage::from_string(lang_dir_name) {
                    let adapters_path =
                        Path::new(lang.tier().name()).join("wasi_snapshot_preview1.wasm");

                    for sub_entry in lang_dir.entries() {
                        if let Some(example_dir) = sub_entry.as_dir() {
                            let example_dir_name =
                                example_dir.path().file_name().unwrap().to_str().unwrap();
                            if example_dir_name != "INSTRUCTIONS"
                                && !example_dir_name.starts_with('.')
                            {
                                let example = parse_example(
                                    &lang,
                                    lang_dir.path(),
                                    Path::new("INSTRUCTIONS"),
                                    &adapters_path,
                                    example_dir.path(),
                                );
                                result.push(example);
                            }
                        }
                    }
                } else {
                    panic!("Invalid guest language name: {lang_dir_name}");
                }
            }
        }
        result
    }

    fn instantiate(example: &Example, parameters: &ExampleParameters) -> io::Result<String> {
        instantiate_directory(
            &EXAMPLES,
            &example.example_path,
            &parameters
                .target_path
                .join(parameters.component_name.as_string()),
            parameters,
            &example.exclude,
            &example.transform_exclude,
            true,
        )?;
        if let Some(adapter_path) = &example.adapter {
            copy(
                &ADAPTERS,
                adapter_path,
                &parameters
                    .target_path
                    .join(parameters.component_name.as_string())
                    .join("adapters")
                    .join(example.language.tier().name())
                    .join(adapter_path.file_name().unwrap().to_str().unwrap()),
            )?;
        }
        let wit_deps_targets = {
            match &example.wit_deps_targets {
                Some(paths) => paths
                    .iter()
                    .map(|path| {
                        parameters
                            .target_path
                            .join(parameters.component_name.as_string())
                            .join(path)
                    })
                    .collect(),
                None => vec![parameters
                    .target_path
                    .join(parameters.component_name.as_string())
                    .join("wit")
                    .join("deps")],
            }
        };
        for wit_dep in &example.wit_deps {
            for target_wit_deps in &wit_deps_targets {
                let target = target_wit_deps.join(wit_dep.file_name().unwrap().to_str().unwrap());
                copy_all(&WIT, wit_dep, &target)?;
            }
        }
        Ok(Self::instructions(example, parameters))
    }

    fn instructions(example: &Example, parameters: &ExampleParameters) -> String {
        transform(&example.instructions, parameters)
    }
}

fn instantiate_directory(
    catalog: &Dir<'_>,
    source: &Path,
    target: &Path,
    parameters: &ExampleParameters,
    excludes: &HashSet<String>,
    transform_excludes: &HashSet<String>,
    filter_metadata: bool,
) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in catalog
        .get_dir(source)
        .unwrap_or_else(|| panic!("Could not find entry {source:?}"))
        .entries()
    {
        let name = entry.path().file_name().unwrap().to_str().unwrap();
        if !excludes.contains(name) && (!filter_metadata || name != "metadata.json") {
            let name = file_name_transform(name, parameters);
            match entry {
                DirEntry::Dir(dir) => {
                    instantiate_directory(
                        catalog,
                        dir.path(),
                        &target.join(&name),
                        parameters,
                        excludes,
                        transform_excludes,
                        false,
                    )?;
                }
                DirEntry::File(file) => {
                    instantiate_file(
                        catalog,
                        file.path(),
                        &target.join(&name),
                        parameters,
                        !transform_excludes.contains(&name),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn instantiate_file(
    catalog: &Dir<'_>,
    source: &Path,
    target: &Path,
    parameters: &ExampleParameters,
    transform_contents: bool,
) -> io::Result<()> {
    let raw_contents = catalog
        .get_file(source)
        .unwrap_or_else(|| panic!("Could not find entry {source:?}"))
        .contents();
    let mut file = File::create(target)?;

    let transformed_contents = transform_contents
        .then(|| String::from_utf8(raw_contents.to_vec()).ok())
        .and_then(identity)
        .map(|contents| transform(contents, parameters));

    if let Some(transformed_contents) = transformed_contents {
        file.write_all(transformed_contents.as_bytes())?;
    } else {
        file.write_all(raw_contents)?;
    }

    Ok(())
}

fn copy(catalog: &Dir<'_>, source: &Path, target: &Path) -> io::Result<()> {
    let contents = catalog
        .get_file(source)
        .unwrap_or_else(|| panic!("Could not find entry {source:?}"))
        .contents();

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(target)?;
    file.write_all(contents)?;
    Ok(())
}

fn copy_all(catalog: &Dir<'_>, source_path: &Path, target_path: &Path) -> io::Result<()> {
    fs::create_dir_all(target_path)?;

    let source_dir = catalog
        .get_dir(source_path)
        .unwrap_or_else(|| panic!("Could not find entry {source_path:?}"));
    for file in source_dir.files() {
        let contents = file.contents();
        let mut file =
            File::create(target_path.join(file.path().file_name().unwrap().to_str().unwrap()))?;
        file.write_all(contents)?;
    }

    Ok(())
}

fn transform(str: impl AsRef<str>, parameters: &ExampleParameters) -> String {
    str.as_ref()
        .replace("component-name", &parameters.component_name.to_kebab_case())
        .replace("ComponentName", &parameters.component_name.to_pascal_case())
        .replace("component_name", &parameters.component_name.to_snake_case())
        .replace(
            "pack::name",
            &parameters.package_name.to_string_with_double_colon(),
        )
        .replace("pack:name", &parameters.package_name.to_string_with_colon())
        .replace("pack_name", &parameters.package_name.to_snake_case())
        .replace("pack-name", &parameters.package_name.to_kebab_case())
        .replace("pack/name", &parameters.package_name.to_string_with_slash())
        .replace("PackName", &parameters.package_name.to_pascal_case())
        .replace("pack-ns", &parameters.package_name.namespace())
        .replace("PackNs", &parameters.package_name.namespace_title_case())
}

fn file_name_transform(str: impl AsRef<str>, parameters: &ExampleParameters) -> String {
    transform(str, parameters).replace("Cargo.toml._", "Cargo.toml") // HACK because cargo package ignores every subdirectory containing a Cargo.toml
}

fn parse_example(
    lang: &GuestLanguage,
    lang_path: &Path,
    default_instructions_file_name: &Path,
    adapters_path: &Path,
    example_root: &Path,
) -> Example {
    let raw_metadata = EXAMPLES
        .get_file(example_root.join("metadata.json"))
        .expect("Failed to read metadata JSON")
        .contents();
    let metadata = serde_json::from_slice::<ExampleMetadata>(raw_metadata)
        .expect("Failed to parse metadata JSON");
    let instructions_path = match metadata.instructions {
        Some(instructions_file_name) => lang_path.join(instructions_file_name),
        None => lang_path.join(default_instructions_file_name),
    };
    let raw_instructions = EXAMPLES
        .get_file(instructions_path)
        .expect("Failed to read instructions")
        .contents();
    let instructions =
        String::from_utf8(raw_instructions.to_vec()).expect("Failed to decode instructions");
    let name = ExampleName::from_string(example_root.file_name().unwrap().to_str().unwrap());

    let mut wit_deps: Vec<PathBuf> = vec![];
    if metadata.requires_golem_host_wit.unwrap_or(false) {
        wit_deps.push(Path::new("golem").to_path_buf());
        wit_deps.push(Path::new("wasm-rpc").to_path_buf());
    }
    if metadata.requires_wasi.unwrap_or(false) {
        wit_deps.push(Path::new("blobstore").to_path_buf());
        wit_deps.push(Path::new("cli").to_path_buf());
        wit_deps.push(Path::new("clocks").to_path_buf());
        wit_deps.push(Path::new("filesystem").to_path_buf());
        wit_deps.push(Path::new("http").to_path_buf());
        wit_deps.push(Path::new("io").to_path_buf());
        wit_deps.push(Path::new("keyvalue").to_path_buf());
        wit_deps.push(Path::new("logging").to_path_buf());
        wit_deps.push(Path::new("random").to_path_buf());
        wit_deps.push(Path::new("sockets").to_path_buf());
    }

    Example {
        name,
        language: lang.clone(),
        description: metadata.description,
        example_path: example_root.to_path_buf(),
        instructions,
        adapter: if metadata.requires_adapter.unwrap_or(true) {
            Some(adapters_path.to_path_buf())
        } else {
            None
        },
        wit_deps,
        wit_deps_targets: metadata
            .wit_deps_paths
            .map(|dirs| dirs.iter().map(PathBuf::from).collect()),
        exclude: metadata.exclude.iter().cloned().collect(),
        transform_exclude: metadata
            .transform_exclude
            .map(|te| te.iter().cloned().collect())
            .unwrap_or_default(),
    }
}
