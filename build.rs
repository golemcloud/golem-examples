use cargo_metadata::MetadataCommand;
use copy_dir::copy_dir;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let golem_wit_root = find_package_root("golem-wit");

    println!("Output dir: {out_dir:?}");
    println!("Golem WIT root: {golem_wit_root:?}");

    copy_dir(golem_wit_root, out_dir.join("golem-wit")).unwrap();
}

fn find_package_root(name: &str) -> String {
    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap();
    let package = metadata.packages.iter().find(|p| p.name == name).unwrap();
    package.manifest_path.parent().unwrap().to_string()
}
