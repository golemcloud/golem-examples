use cargo_metadata::MetadataCommand;
use copy_dir::copy_dir;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let golem_wit_root = PathBuf::from(
        env::var("GOLEM_WIT_ROOT").unwrap_or_else(|_| find_package_root("golem-wit")),
    );

    println!("cargo:warning=Output dir: {out_dir:?}");
    println!("cargo:warning=Golem WIT root: {golem_wit_root:?}");

    let target = out_dir.join("golem-wit");
    if target.exists() {
        if dir_diff::is_different(&golem_wit_root, &target).unwrap_or(true) {
            std::fs::remove_dir_all(&target).unwrap();
            copy_dir(golem_wit_root, target).unwrap();
        } else {
            println!("cargo:warning=Golem WIT is up to date in {target:?}");
        }
    } else {
        copy_dir(golem_wit_root, target).unwrap();
    }
}

fn find_package_root(name: &str) -> String {
    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap();
    let package = metadata.packages.iter().find(|p| p.name == name).unwrap();
    package.manifest_path.parent().unwrap().to_string()
}
