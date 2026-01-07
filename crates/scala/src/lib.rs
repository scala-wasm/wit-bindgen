use anyhow::Result;
use std::collections::HashSet;
use wit_bindgen_core::{Files, WorldGenerator, wit_parser::*};

pub mod annotations;
pub mod context;
pub mod interface;
pub mod resource;
pub mod world;

pub use context::ScalaContext;

/// Configuration options for the Scala bindings generator.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Opts {
    /// Base package for generated bindings (e.g., "com.example.wasm")
    #[cfg_attr(feature = "clap", arg(long, default_value = "componentmodel"))]
    pub base_package: String,

    /// Output directory for bindings
    #[cfg_attr(feature = "clap", arg(long))]
    pub binding_root: Option<String>,
}

impl Opts {
    pub fn build(&self) -> Box<dyn WorldGenerator> {
        Box::new(Scala::new(self.clone()))
    }
}

/// Main Scala bindings generator.
pub struct Scala {
    context: ScalaContext,
    imports: HashSet<InterfaceId>,
    exports: HashSet<InterfaceId>,
    world_import_funcs: Vec<(String, Function)>,
    world_export_funcs: Vec<(String, Function)>,
}

impl Scala {
    fn new(opts: Opts) -> Self {
        Self {
            context: ScalaContext::new(&opts),
            imports: HashSet::new(),
            exports: HashSet::new(),
            world_import_funcs: Vec::new(),
            world_export_funcs: Vec::new(),
        }
    }
}

impl WorldGenerator for Scala {
    fn preprocess(&mut self, _resolve: &Resolve, _world: WorldId) {
        // No preprocessing needed
    }

    fn import_interface(
        &mut self,
        resolve: &Resolve,
        name: &WorldKey,
        id: InterfaceId,
        files: &mut Files,
    ) -> Result<()> {
        self.imports.insert(id);

        let interface = &resolve.interfaces[id];
        let interface_name = interface
            .name
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Interface must have a name"))?;

        // Build namespace string from package info
        let namespace = if let Some(package_id) = interface.package {
            let package = &resolve.packages[package_id];
            let pkg_name = &package.name;
            // Format: "namespace:name/interface@version"
            if let Some(version) = &pkg_name.version {
                format!(
                    "{}:{}/{}@{}",
                    pkg_name.namespace, pkg_name.name, interface_name, version
                )
            } else {
                format!(
                    "{}:{}/{}",
                    pkg_name.namespace, pkg_name.name, interface_name
                )
            }
        } else {
            // Fallback to using world key name
            resolve.name_world_key(name)
        };

        // Generate interface content
        let content = interface::render_interface(
            &mut self.context,
            resolve,
            id,
            &namespace,
            true, // is_import
        );

        // Get file path
        let file_path = interface::get_interface_file_path(
            &self.context,
            &namespace,
            interface_name,
            true, // is_import
        );

        files.push(&file_path, content.as_bytes());

        Ok(())
    }

    fn import_funcs(
        &mut self,
        _resolve: &Resolve,
        _world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) {
        for (name, func) in funcs {
            self.world_import_funcs.push((name.to_string(), (*func).clone()));
        }
    }

    fn import_types(
        &mut self,
        _resolve: &Resolve,
        _world: WorldId,
        _types: &[(&str, TypeId)],
        _files: &mut Files,
    ) {
        // World-level types are handled in finish()
    }

    fn export_interface(
        &mut self,
        resolve: &Resolve,
        name: &WorldKey,
        id: InterfaceId,
        files: &mut Files,
    ) -> Result<()> {
        self.exports.insert(id);

        let interface = &resolve.interfaces[id];
        let interface_name = interface
            .name
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Interface must have a name"))?;

        // Build namespace string from package info
        let namespace = if let Some(package_id) = interface.package {
            let package = &resolve.packages[package_id];
            let pkg_name = &package.name;
            // Format: "namespace:name/interface@version"
            if let Some(version) = &pkg_name.version {
                format!(
                    "{}:{}/{}@{}",
                    pkg_name.namespace, pkg_name.name, interface_name, version
                )
            } else {
                format!(
                    "{}:{}/{}",
                    pkg_name.namespace, pkg_name.name, interface_name
                )
            }
        } else {
            // Fallback to using world key name
            resolve.name_world_key(name)
        };

        // Generate interface content
        let content = interface::render_interface(
            &mut self.context,
            resolve,
            id,
            &namespace,
            false, // is_import = false for exports
        );

        // Get file path
        let file_path = interface::get_interface_file_path(
            &self.context,
            &namespace,
            interface_name,
            false, // is_import = false for exports
        );

        files.push(&file_path, content.as_bytes());

        Ok(())
    }

    fn export_funcs(
        &mut self,
        _resolve: &Resolve,
        _world: WorldId,
        funcs: &[(&str, &Function)],
        _files: &mut Files,
    ) -> Result<()> {
        for (name, func) in funcs {
            self.world_export_funcs.push((name.to_string(), (*func).clone()));
        }
        Ok(())
    }

    fn finish(&mut self, resolve: &Resolve, world_id: WorldId, files: &mut Files) -> Result<()> {
        let world = &resolve.worlds[world_id];
        let mut generated_count = self.imports.len() + self.exports.len();

        let has_world_imports = !self.world_import_funcs.is_empty()
            || world.imports.values().any(|item| matches!(item, WorldItem::Type(_)));

        if has_world_imports {
            if let Some(content) = world::render_world(
                &mut self.context,
                resolve,
                world_id,
                true, // is_import
                &self.world_import_funcs,
            ) {
                let file_path = world::get_world_file_path(&self.context, true);
                files.push(&file_path, content.as_bytes());
                generated_count += 1;
            }
        }

        let has_world_exports = !self.world_export_funcs.is_empty()
            || world.exports.values().any(|item| matches!(item, WorldItem::Type(_)));

        if has_world_exports {
            if let Some(content) = world::render_world(
                &mut self.context,
                resolve,
                world_id,
                false, // is_import = false for exports
                &self.world_export_funcs,
            ) {
                let file_path = world::get_world_file_path(&self.context, false);
                files.push(&file_path, content.as_bytes());
                generated_count += 1;
            }
        }

        eprintln!(
            "Generated {} Scala files ({} imports, {} exports)",
            generated_count,
            self.imports.len(),
            self.exports.len()
        );

        Ok(())
    }
}
