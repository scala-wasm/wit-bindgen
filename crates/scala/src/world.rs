/// World-level code generation for top-level functions and types.
///
/// Worlds can have top-level imports and exports that are not part of
/// any interface. These are generated in separate world files.
use crate::ScalaContext;
use std::fmt::Write as _;
use wit_bindgen_core::wit_parser::*;

/// Generate a world file for top-level imports or exports.
pub fn render_world(
    ctx: &mut ScalaContext,
    resolve: &Resolve,
    world_id: WorldId,
    is_import: bool,
) -> Option<String> {
    let world = &resolve.worlds[world_id];
    let world_name = &world.name;
    let package_name = ctx.to_snake_case(world_name);

    let mut has_content = false;
    let mut output = String::new();

    // Determine package path
    let package_path = get_world_package_path(ctx, world_name, is_import);
    writeln!(&mut output, "package {}", package_path).unwrap();
    writeln!(&mut output).unwrap();

    writeln!(&mut output, "package object {} {{", package_name).unwrap();
    writeln!(&mut output).unwrap();

    // Generate top-level types
    if is_import {
        for (_name, item) in &world.imports {
            if let WorldItem::Type(type_id) = item {
                let typedef = ctx.render_typedef(resolve, *type_id);
                if !typedef.is_empty() && !typedef.starts_with("//") {
                    has_content = true;
                    writeln!(&mut output, "  // Type definitions").unwrap();
                    for line in typedef.lines() {
                        if line.is_empty() {
                            writeln!(&mut output).unwrap();
                        } else {
                            writeln!(&mut output, "  {}", line).unwrap();
                        }
                    }
                    writeln!(&mut output).unwrap();
                }
            }
        }
    } else {
        for (_name, item) in &world.exports {
            if let WorldItem::Type(type_id) = item {
                let typedef = ctx.render_typedef(resolve, *type_id);
                if !typedef.is_empty() && !typedef.starts_with("//") {
                    has_content = true;
                    writeln!(&mut output, "  // Type definitions").unwrap();
                    for line in typedef.lines() {
                        if line.is_empty() {
                            writeln!(&mut output).unwrap();
                        } else {
                            writeln!(&mut output, "  {}", line).unwrap();
                        }
                    }
                    writeln!(&mut output).unwrap();
                }
            }
        }
    }

    writeln!(&mut output, "}}").unwrap();

    if has_content { Some(output) } else { None }
}

/// Get the package path for a world.
pub fn get_world_package_path(ctx: &ScalaContext, world_name: &str, is_import: bool) -> String {
    let mut segments = ctx.base_package_segments();

    if !is_import {
        segments.push("exports".to_string());
    }

    segments.push(ctx.to_snake_case(world_name));

    segments.join(".")
}

/// Get the file path for a world file.
pub fn get_world_file_path(ctx: &ScalaContext, world_name: &str, is_import: bool) -> String {
    let mut segments = ctx.base_package_segments();

    if !is_import {
        segments.push("exports".to_string());
    }

    segments.push(ctx.to_snake_case(world_name));

    let path = segments.join("/");
    format!("{}/package.scala", path)
}
