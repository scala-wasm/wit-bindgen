use wit_bindgen_scala::annotations::*;

#[test]
fn test_component_import() {
    assert_eq!(
        component_import("wasi:io/streams@0.2.0", "read"),
        "@scala.scalajs.wit.annotation.WitImport(\"wasi:io/streams@0.2.0\", \"read\")"
    );
}

#[test]
fn test_component_export() {
    assert_eq!(
        component_export("wasi:cli/run@0.2.0", "run"),
        "@scala.scalajs.wit.annotation.WitExport(\"wasi:cli/run@0.2.0\", \"run\")"
    );
}

#[test]
fn test_component_record() {
    assert_eq!(
        component_record(),
        "@scala.scalajs.wit.annotation.WitRecord"
    );
}

#[test]
fn test_component_variant() {
    assert_eq!(
        component_variant(),
        "@scala.scalajs.wit.annotation.WitVariant"
    );
}

#[test]
fn test_component_flags() {
    assert_eq!(
        component_flags(8),
        "@scala.scalajs.wit.annotation.WitFlags(8)"
    );
}

#[test]
fn test_component_resource_import() {
    assert_eq!(
        component_resource_import("wasi:io/streams@0.2.0", "input-stream"),
        "@scala.scalajs.wit.annotation.WitResourceImport(\"wasi:io/streams@0.2.0\", \"input-stream\")"
    );
}

#[test]
fn test_component_resource_method() {
    assert_eq!(
        component_resource_method("read"),
        "@scala.scalajs.wit.annotation.WitResourceMethod(\"read\")"
    );
}

#[test]
fn test_component_resource_static_method() {
    assert_eq!(
        component_resource_static_method("merge"),
        "@scala.scalajs.wit.annotation.WitResourceStaticMethod(\"merge\")"
    );
}

#[test]
fn test_component_resource_drop() {
    assert_eq!(
        component_resource_drop(),
        "@scala.scalajs.wit.annotation.WitResourceDrop"
    );
}

#[test]
fn test_component_export_interface() {
    assert_eq!(
        component_export_interface(),
        "@scala.scalajs.wit.annotation.WitExportInterface"
    );
}

#[test]
fn test_import_function() {
    let result = import_function(
        "wasi:io/streams@0.2.0",
        "read",
        "read",
        &[
            ("stream".to_string(), "InputStream".to_string()),
            ("len".to_string(), "Long".to_string()),
        ],
        Some("scala.scalajs.wit.Result[Array[Byte], StreamError]"),
        "",
    );

    assert!(result.contains("@scala.scalajs.wit.annotation.WitImport(\"wasi:io/streams@0.2.0\", \"read\")"));
    assert!(result.contains("def read(stream: InputStream, len: Long): scala.scalajs.wit.Result[Array[Byte], StreamError] = scala.scalajs.wit.native"));
}

#[test]
fn test_export_function() {
    let result = export_function(
        "my:app/handler@1.0.0",
        "handle-request",
        "handleRequest",
        &[("req".to_string(), "Request".to_string())],
        Some("Response"),
        "",
    );

    assert!(
        result.contains(
            "@scala.scalajs.wit.annotation.WitExport(\"my:app/handler@1.0.0\", \"handle-request\")"
        )
    );
    assert!(result.contains("def handleRequest(req: Request): Response"));
    assert!(!result.contains("native")); // Export functions don't have native marker
}
