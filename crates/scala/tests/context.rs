use wit_bindgen_core::wit_parser::{Function, FunctionKind, Resolve, Type};
use wit_bindgen_scala::{Opts, ScalaContext};

#[test]
fn test_primitive_types() {
    let mut ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    // Test with fully qualified names
    assert_eq!(ctx.render_primitive_type(&Type::Bool), "Boolean");
    assert_eq!(ctx.render_primitive_type(&Type::S8), "Byte");
    assert_eq!(
        ctx.render_primitive_type(&Type::U8),
        "scala.scalajs.wit.unsigned.UByte"
    );
    assert_eq!(ctx.render_primitive_type(&Type::S16), "Short");
    assert_eq!(
        ctx.render_primitive_type(&Type::U16),
        "scala.scalajs.wit.unsigned.UShort"
    );
    assert_eq!(ctx.render_primitive_type(&Type::S32), "Int");
    assert_eq!(
        ctx.render_primitive_type(&Type::U32),
        "scala.scalajs.wit.unsigned.UInt"
    );
    assert_eq!(ctx.render_primitive_type(&Type::S64), "Long");
    assert_eq!(
        ctx.render_primitive_type(&Type::U64),
        "scala.scalajs.wit.unsigned.ULong"
    );
    assert_eq!(ctx.render_primitive_type(&Type::F32), "Float");
    assert_eq!(ctx.render_primitive_type(&Type::F64), "Double");
    assert_eq!(ctx.render_primitive_type(&Type::Char), "Char");
    assert_eq!(ctx.render_primitive_type(&Type::String), "String");
}

#[test]
fn test_keyword_escaping() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    assert_eq!(ctx.escape_keyword("type"), "`type`");
    assert_eq!(ctx.escape_keyword("class"), "`class`");
    assert_eq!(ctx.escape_keyword("val"), "`val`");
    assert_eq!(ctx.escape_keyword("normal"), "normal");
}

#[test]
fn test_name_conversions() {
    let ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    assert_eq!(ctx.to_camel_case("kebab-case-name"), "kebabCaseName");
    assert_eq!(ctx.to_pascal_case("kebab-case-name"), "KebabCaseName");
    assert_eq!(ctx.to_snake_case("kebab-case-name"), "kebab_case_name");

    // With keywords - "type" alone is a keyword, but "typeName" is not
    assert_eq!(ctx.to_camel_case("type-name"), "typeName");
    assert_eq!(ctx.to_pascal_case("class-name"), "ClassName");

    // Pure keywords need escaping (camelCase keeps it lowercase, PascalCase capitalizes it)
    assert_eq!(ctx.to_camel_case("type"), "`type`");
    assert_eq!(ctx.to_pascal_case("class"), "Class"); // "Class" is not a keyword
}

#[test]
fn test_render_function_import() {
    let mut ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    let resolve = Resolve::default();

    // Test function with parameters and return type
    let func = Function {
        name: "read-data".to_string(),
        kind: FunctionKind::Freestanding,
        params: vec![
            ("stream".to_string(), Type::String),
            ("length".to_string(), Type::U32),
        ],
        result: Some(Type::Bool),
        docs: Default::default(),
        stability: Default::default(),
    };

    let result = ctx.render_function(&resolve, &func, true, "test:example/api@1.0.0");

    assert!(result.contains("@scala.scalajs.wit.annotation.WitImport(\"test:example/api@1.0.0\", \"read-data\")"));
    assert!(result.contains("def readData("));
    assert!(result.contains("stream: String"));
    assert!(result.contains("length: scala.scalajs.wit.unsigned.UInt"));
    assert!(result.contains("): Boolean"));
    assert!(result.contains("= scala.scalajs.wit.native"));
}

#[test]
fn test_render_function_export() {
    let mut ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    let resolve = Resolve::default();

    // Test function with no return type
    let func = Function {
        name: "handle-request".to_string(),
        kind: FunctionKind::Freestanding,
        params: vec![("request".to_string(), Type::String)],
        result: None,
        docs: Default::default(),
        stability: Default::default(),
    };

    let result = ctx.render_function(&resolve, &func, false, "my:app/handler@1.0.0");

    assert!(
        result.contains(
            "@scala.scalajs.wit.annotation.WitExport(\"my:app/handler@1.0.0\", \"handle-request\")"
        )
    );
    assert!(result.contains("def handleRequest("));
    assert!(result.contains("request: String"));
    assert!(result.contains("): Unit"));
    assert!(!result.contains("native")); // Export functions don't have native marker
}
