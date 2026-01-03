use wit_bindgen_scala::{Opts, ScalaContext};
use wit_bindgen_scala::world::{get_world_package_path, get_world_file_path};

#[test]
fn test_get_world_package_path_import() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_package_path(&ctx, "my-world", true);
    assert_eq!(path, "com.example.my_world");
}

#[test]
fn test_get_world_package_path_export() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_package_path(&ctx, "my-world", false);
    assert_eq!(path, "com.example.exports.my_world");
}

#[test]
fn test_get_world_file_path_import() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_file_path(&ctx, "my-world", true);
    assert_eq!(path, "com/example/my_world/package.scala");
}

#[test]
fn test_get_world_file_path_export() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_file_path(&ctx, "my-world", false);
    assert_eq!(path, "com/example/exports/my_world/package.scala");
}
