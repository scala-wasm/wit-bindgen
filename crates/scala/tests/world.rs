use wit_bindgen_scala::{Opts, ScalaContext};
use wit_bindgen_scala::world::{get_world_package_path, get_world_file_path};

#[test]
fn test_get_world_package_path_import() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_package_path(&ctx, true);
    assert_eq!(path, "com.example");
}

#[test]
fn test_get_world_package_path_export() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_package_path(&ctx, false);
    assert_eq!(path, "com.example.exports");
}

#[test]
fn test_get_world_file_path_import() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_file_path(&ctx, true);
    assert_eq!(path, "com/example/package.scala");
}

#[test]
fn test_get_world_file_path_export() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_world_file_path(&ctx, false);
    assert_eq!(path, "com/example/exports/Root.scala");
}
