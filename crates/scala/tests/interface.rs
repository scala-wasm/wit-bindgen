use wit_bindgen_scala::{Opts, ScalaContext};
use wit_bindgen_scala::interface::{get_package_path, get_interface_file_path};

#[test]
fn test_get_package_path_import() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_package_path(&ctx, "wasi:io/streams@0.2.0", true);
    assert_eq!(path, "com.example.wasi.io");
}

#[test]
fn test_get_package_path_import_kebab() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_package_path(&ctx, "scala-wasm:scala-wasm/foo-bar@0.2.0", true);
    assert_eq!(path, "com.example.scala_wasm.scala_wasm");
}

#[test]
fn test_get_package_path_export() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_package_path(&ctx, "my:app/handler@1.0.0", false);
    assert_eq!(path, "com.example.exports.my.app");
}

#[test]
fn test_get_interface_file_path_import() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_interface_file_path(&ctx, "wasi:io/streams@0.2.0", "streams", true);
    assert_eq!(path, "com/example/wasi/io/streams.scala");
}

#[test]
fn test_get_interface_file_path_export() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_interface_file_path(&ctx, "my:app/handler@1.0.0", "handler", false);
    assert_eq!(path, "com/example/exports/my/app/handler.scala");
}

#[test]
fn test_get_interface_file_path_with_kebab_case() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "com.example".to_string(),
        binding_root: None,
    });

    let path = get_interface_file_path(&ctx, "my-org:my-app/my-handler@1.0.0", "my-handler", true);
    assert_eq!(path, "com/example/my_org/my_app/my_handler.scala");
}

#[test]
fn test_get_package_path_no_version() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    let path = get_package_path(&ctx, "example:api/basic", true);
    assert_eq!(path, "test.example.api");
}
