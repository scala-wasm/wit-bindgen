use wit_bindgen_core::{Files, wit_parser::Resolve};
use wit_bindgen_scala::{Opts, WithOption};

fn generate_scala(wit: &str) -> Files {
    generate_scala_with(wit, vec![])
}

fn generate_scala_with(wit: &str, with: Vec<(String, WithOption)>) -> Files {
    let mut resolve = Resolve::default();
    let pkg = resolve.push_str("test.wit", wit).unwrap();
    let world = resolve.select_world(&[pkg], None).unwrap();

    let opts = Opts {
        base_package: "com.example.test".to_string(),
        binding_root: None,
        generate_unapply: false,
        with,
    };
    let mut generator = opts.build();
    let mut files = Files::default();

    generator.generate(&resolve, world, &mut files).unwrap();

    files
}

#[test]
fn test_simple_types() {
    let wit = r#"
        package test:types;

        interface simple {
            record point {
                x: s32,
                y: s32,
            }

            add: func(a: s32, b: s32) -> s32;
        }

        world test {
            import simple;
        }
    "#;

    let files = generate_scala(wit);
    assert!(!files.iter().collect::<Vec<_>>().is_empty());

    // Check that the generated file contains expected content
    let contents: Vec<_> = files.iter().collect();
    let scala_content = std::str::from_utf8(contents[0].1).unwrap();

    assert!(scala_content.contains("package com.example.test"));
    assert!(scala_content.contains("final class Point"));
    assert!(scala_content.contains("def add"));
    assert!(scala_content.contains("@scala.scalajs.wit.annotation.WitImport"));
}

#[test]
fn test_variants() {
    let wit = r#"
        package test:variants;

        interface types {
            variant outcome {
                ok(string),
                err(string),
            }

            enum color {
                red,
                green,
                blue,
            }
        }

        world test {
            import types;
        }
    "#;

    let files = generate_scala(wit);
    let contents: Vec<_> = files.iter().collect();
    let scala_content = std::str::from_utf8(contents[0].1).unwrap();

    assert!(scala_content.contains("sealed trait Outcome"));
    assert!(scala_content.contains("sealed trait Color"));
    assert!(scala_content.contains("@scala.scalajs.wit.annotation.WitVariant"));
}

#[test]
fn test_lists_and_options() {
    let wit = r#"
        package test:collections;

        interface data {
            process: func(items: list<u32>) -> option<string>;
        }

        world test {
            import data;
        }
    "#;

    let files = generate_scala(wit);
    let contents: Vec<_> = files.iter().collect();
    let scala_content = std::str::from_utf8(contents[0].1).unwrap();

    assert!(scala_content.contains("Array["));
    assert!(scala_content.contains("java.util.Optional["));
    assert!(scala_content.contains("scala.scalajs.wit.unsigned.UInt"));
}

#[test]
fn test_resources() {
    let wit = r#"
        package test:resources;

        interface counters {
            resource counter {
                constructor(initial: s32);
                increment: func();
                value: func() -> s32;
            }
        }

        world test {
            import counters;
        }
    "#;

    let files = generate_scala(wit);
    let contents: Vec<_> = files.iter().collect();
    let scala_content = std::str::from_utf8(contents[0].1).unwrap();

    assert!(scala_content.contains("trait Counter"));
    assert!(scala_content.contains("object Counter"));
    assert!(scala_content.contains("@scala.scalajs.wit.annotation.WitResourceImport"));
    assert!(
        scala_content.contains("@scala.scalajs.wit.annotation.WitResourceConstructor")
    );
    assert!(scala_content.contains("def apply(initial: Int): Counter"));
}

#[test]
fn test_import_export() {
    let wit = r#"
        package test:both;

        interface math {
            add: func(a: s32, b: s32) -> s32;
        }

        world test {
            import math;
            export math;
        }
    "#;

    let files = generate_scala(wit);
    let contents: Vec<_> = files.iter().collect();

    // Should generate 2 files: one for import, one for export
    assert_eq!(contents.len(), 2);

    let import_file = contents
        .iter()
        .find(|(path, _)| !path.contains("exports"))
        .unwrap();
    let export_file = contents
        .iter()
        .find(|(path, _)| path.contains("exports"))
        .unwrap();

    let import_content = std::str::from_utf8(import_file.1).unwrap();
    let export_content = std::str::from_utf8(export_file.1).unwrap();

    // Import should have native marker
    assert!(import_content.contains("= scala.scalajs.wit.native"));
    assert!(import_content.contains("@scala.scalajs.wit.annotation.WitImport"));

    // Export should be abstract (no native marker)
    assert!(!export_content.contains("= scala.scalajs.wit.native"));
    assert!(export_content.contains("@scala.scalajs.wit.annotation.WitExport"));

    // Export should have package object structure with trait inside
    assert!(export_content.contains("package object math"));
    assert!(export_content.contains("trait Math"));
}

#[test]
fn test_flags() {
    let wit = r#"
        package test:perms;

        interface permissions {
            flags file-perms {
                read,
                write,
                execute,
            }
        }

        world test {
            import permissions;
        }
    "#;

    let files = generate_scala(wit);
    let contents: Vec<_> = files.iter().collect();
    let scala_content = std::str::from_utf8(contents[0].1).unwrap();

    assert!(scala_content.contains("final class FilePerms"));
    assert!(scala_content.contains("@scala.scalajs.wit.annotation.WitFlags"));
    assert!(scala_content.contains("val read ="));
    assert!(scala_content.contains("val write ="));
    assert!(scala_content.contains("val execute ="));
    assert!(scala_content.contains("def |"));
    assert!(scala_content.contains("def &"));
}

#[test]
fn test_with_remap_skips_import() {
    let wit = r#"
        package test:types;

        interface simple {
            record point {
                x: s32,
                y: s32,
            }

            add: func(a: s32, b: s32) -> s32;
        }

        world test {
            import simple;
        }
    "#;

    let files = generate_scala_with(
        wit,
        vec![(
            "test:types/simple".to_string(),
            WithOption::Path("external.types.simple".to_string()),
        )],
    );

    // Remapped import should produce no files
    let contents: Vec<_> = files.iter().collect();
    assert!(contents.is_empty(), "Remapped import should produce no files");
}

#[test]
fn test_with_generate_produces_output() {
    let wit = r#"
        package test:types;

        interface simple {
            record point {
                x: s32,
                y: s32,
            }

            add: func(a: s32, b: s32) -> s32;
        }

        world test {
            import simple;
        }
    "#;

    let files = generate_scala_with(
        wit,
        vec![(
            "test:types/simple".to_string(),
            WithOption::Generate,
        )],
    );

    // Explicit `generate` should still produce output
    let contents: Vec<_> = files.iter().collect();
    assert!(!contents.is_empty(), "=generate should produce files");

    let scala_content = std::str::from_utf8(contents[0].1).unwrap();
    assert!(scala_content.contains("final class Point"));
    assert!(scala_content.contains("def add"));
}

#[test]
fn test_cross_interface_type_uses_external_path() {
    let wit = r#"
        package test:types;

        interface base {
            record point {
                x: s32,
                y: s32,
            }
        }

        interface consumer {
            use base.{point};

            process: func(p: point) -> point;
        }

        world test {
            import base;
            import consumer;
        }
    "#;

    let files = generate_scala_with(
        wit,
        vec![(
            "test:types/base".to_string(),
            WithOption::Path("external.base.types".to_string()),
        )],
    );

    let contents: Vec<_> = files.iter().collect();
    // Only consumer should be generated (base is remapped)
    assert_eq!(contents.len(), 1);

    let scala_content = std::str::from_utf8(contents[0].1).unwrap();
    // Type reference should use the external path
    assert!(
        scala_content.contains("external.base.types.Point"),
        "Cross-interface type should use external path, got:\n{}",
        scala_content
    );
}

#[test]
fn test_export_remapped_interface_trait_only() {
    let wit = r#"
        package test:types;

        interface handler {
            record point {
                x: s32,
                y: s32,
            }

            process: func(p: point) -> point;
        }

        world test {
            export handler;
        }
    "#;

    let files = generate_scala_with(
        wit,
        vec![(
            "test:types/handler".to_string(),
            WithOption::Path("external.types.handler".to_string()),
        )],
    );

    let contents: Vec<_> = files.iter().collect();
    // Export of remapped interface should still produce a file (trait only)
    assert_eq!(contents.len(), 1);

    let export_content = std::str::from_utf8(contents[0].1).unwrap();

    // Should have trait
    assert!(
        export_content.contains("trait Handler"),
        "Remapped export should generate trait, got:\n{}",
        export_content
    );

    // Types should resolve to external path
    assert!(
        export_content.contains("external.types.handler.Point"),
        "Remapped export type refs should use external path, got:\n{}",
        export_content
    );

    // Should NOT have package object (trait-only mode)
    assert!(
        !export_content.contains("package object"),
        "Remapped export should not have package object, got:\n{}",
        export_content
    );
}

#[test]
fn test_with_package_level_type_resolution() {
    let wit = r#"
        package test:types;

        interface base {
            record point {
                x: s32,
                y: s32,
            }
        }

        interface consumer {
            use base.{point};

            process: func(p: point) -> point;
        }

        world test {
            import base;
            export consumer;
        }
    "#;

    // Package-level remap: base import skipped, consumer export uses external types
    let files = generate_scala_with(
        wit,
        vec![(
            "test:types".to_string(),
            WithOption::Path("external.types".to_string()),
        )],
    );

    let contents: Vec<_> = files.iter().collect();
    // consumer export generates trait-only (it's also matched by package-level)
    assert_eq!(contents.len(), 1);

    let export_content = std::str::from_utf8(contents[0].1).unwrap();

    // base types resolve through package-level match: external.types + base → external.types.base
    assert!(
        export_content.contains("external.types.base.Point"),
        "Package-level remap should resolve cross-interface types as external.types.<iface>.Type, got:\n{}",
        export_content
    );
}

