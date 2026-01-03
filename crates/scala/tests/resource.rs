use wit_bindgen_core::wit_parser::{Function, FunctionKind, Resolve, Type, TypeDef, TypeDefKind, TypeOwner};
use wit_bindgen_scala::{Opts, ScalaContext};
use wit_bindgen_scala::resource::{render_resource_method, render_resource_constructor, render_resource_drop_method};

#[test]
fn test_render_resource_method() {
    let mut ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    let mut resolve = Resolve::default();
    let dummy_resource_id = resolve.types.alloc(TypeDef {
        name: Some("DummyResource".to_string()),
        kind: TypeDefKind::Resource,
        owner: TypeOwner::None,
        docs: Default::default(),
        stability: Default::default(),
    });

    let func = Function {
        name: "read".to_string(),
        kind: FunctionKind::Method(dummy_resource_id),
        params: vec![("length".to_string(), Type::U32)],
        result: Some(Type::Bool),
        docs: Default::default(),
        stability: Default::default(),
    };

    let result = render_resource_method(&mut ctx, &resolve, "read", &func);

    assert!(
        result
            .contains("@scala.scalajs.wit.annotation.WitResourceMethod(\"read\")")
    );
    assert!(result.contains("def read("));
    assert!(result.contains("length: scala.scalajs.wit.unsigned.UInt"));
    assert!(result.contains("): Boolean"));
    assert!(result.contains("= scala.scalajs.wit.native"));
}

#[test]
fn test_render_resource_constructor() {
    let mut ctx = ScalaContext::new(&Opts {
        base_package: "test".to_string(),
        binding_root: None,
    });

    let mut resolve = Resolve::default();
    let dummy_resource_id = resolve.types.alloc(TypeDef {
        name: Some("Counter".to_string()),
        kind: TypeDefKind::Resource,
        owner: TypeOwner::None,
        docs: Default::default(),
        stability: Default::default(),
    });

    let func = Function {
        name: "constructor".to_string(),
        kind: FunctionKind::Constructor(dummy_resource_id),
        params: vec![("initial".to_string(), Type::S32)],
        result: None,
        docs: Default::default(),
        stability: Default::default(),
    };

    let result = render_resource_constructor(&mut ctx, &resolve, "Counter", &func);

    assert!(
        result.contains("@scala.scalajs.wit.annotation.WitResourceConstructor")
    );
    assert!(result.contains("def apply("));
    assert!(result.contains("initial: Int"));
    assert!(result.contains("): Counter"));
    assert!(result.contains("= scala.scalajs.wit.native"));
}

#[test]
fn test_render_resource_drop_method() {
    let result = render_resource_drop_method();
    assert!(result.contains("@scala.scalajs.wit.annotation.WitResourceDrop"));
    assert!(result.contains("def close(): Unit = scala.scalajs.wit.native"));
}
