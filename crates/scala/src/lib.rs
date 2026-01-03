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
    has_world_imports: bool,
    has_world_exports: bool,
}

impl Scala {
    fn new(opts: Opts) -> Self {
        Self {
            context: ScalaContext::new(&opts),
            imports: HashSet::new(),
            exports: HashSet::new(),
            has_world_imports: false,
            has_world_exports: false,
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
        // Mark that we have world-level imports (functions or types)
        if !funcs.is_empty() {
            self.has_world_imports = true;
        }
    }

    fn import_types(
        &mut self,
        _resolve: &Resolve,
        _world: WorldId,
        types: &[(&str, TypeId)],
        _files: &mut Files,
    ) {
        // Mark that we have world-level imports (functions or types)
        if !types.is_empty() {
            self.has_world_imports = true;
        }
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
        // Mark that we have world-level exports (functions or types)
        if !funcs.is_empty() {
            self.has_world_exports = true;
        }
        Ok(())
    }

    fn finish(&mut self, resolve: &Resolve, world_id: WorldId, files: &mut Files) -> Result<()> {
        let world = &resolve.worlds[world_id];
        let world_name = &world.name;
        let mut generated_count = self.imports.len() + self.exports.len();

        // Generate world-level import file if there are world-level imports
        if self.has_world_imports {
            if let Some(content) = world::render_world(
                &mut self.context,
                resolve,
                world_id,
                true, // is_import
            ) {
                let file_path = world::get_world_file_path(&self.context, world_name, true);
                files.push(&file_path, content.as_bytes());
                generated_count += 1;
            }
        }

        // Generate world-level export file if there are world-level exports
        if self.has_world_exports {
            if let Some(content) = world::render_world(
                &mut self.context,
                resolve,
                world_id,
                false, // is_import = false for exports
            ) {
                let file_path = world::get_world_file_path(&self.context, world_name, false);
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
