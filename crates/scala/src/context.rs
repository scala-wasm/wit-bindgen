use crate::{Opts, annotations};
use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::collections::HashSet;
use std::fmt::Write as _;
use wit_bindgen_core::wit_parser::*;

/// Format WIT documentation as Scaladoc comments.
///
/// Converts WIT documentation strings into properly formatted Scaladoc with
/// the correct indentation and continuation markers.
pub fn format_docs(docs: &Docs) -> String {
    format_docs_with_indent(docs, 0)
}

/// Format WIT documentation as Scaladoc comments with custom indentation.
///
/// Converts WIT documentation strings into properly formatted Scaladoc with
/// the specified indentation level (number of spaces) and continuation markers.
pub fn format_docs_with_indent(docs: &Docs, indent: usize) -> String {
    let content = docs.contents.as_deref().unwrap_or("").trim();
    if content.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    let indent_str = " ".repeat(indent);

    // First line with opening /**
    writeln!(&mut output, "{}/** {}", indent_str, lines[0]).unwrap();

    // Subsequent lines with continuation marker
    for line in &lines[1..] {
        if line.trim().is_empty() {
            writeln!(&mut output, "{} *", indent_str).unwrap();
        } else {
            writeln!(&mut output, "{} *  {}", indent_str, line).unwrap();
        }
    }

    // Closing */
    writeln!(&mut output, "{} */", indent_str).unwrap();

    output
}

/// Context for Scala code generation, containing shared utilities and state.
pub struct ScalaContext {
    opts: Opts,
    keywords: ScalaKeywords,
    /// Current interface being rendered (for cross-interface type references)
    current_interface: Option<InterfaceId>,
}

impl ScalaContext {
    pub fn new(opts: &Opts) -> Self {
        Self {
            opts: opts.clone(),
            keywords: ScalaKeywords::new(),
            current_interface: None,
        }
    }

    /// Set the current interface being rendered (for cross-interface type references).
    pub fn set_current_interface(&mut self, interface_id: Option<InterfaceId>) {
        self.current_interface = interface_id;
    }

    /// Generate fully qualified package path for a type from another interface.
    fn get_qualified_type_name(&self, resolve: &Resolve, type_id: TypeId, type_name: &str) -> String {
        let ty = &resolve.types[type_id];

        // Check if this type is from a different interface
        if let TypeOwner::Interface(type_interface_id) = ty.owner {
            // If we're in an interface and the type is from a different interface, qualify it
            if let Some(current_interface_id) = self.current_interface {
                if type_interface_id != current_interface_id {
                    // Type is from a different interface - need fully qualified name
                    let type_interface = &resolve.interfaces[type_interface_id];
                    let interface_name = type_interface.name.as_ref().expect("Interface must have a name");

                    if let Some(package_id) = type_interface.package {
                        let package = &resolve.packages[package_id];
                        let pkg_name = &package.name;

                        // Build the fully qualified path
                        let mut segments = self.base_package_segments();
                        segments.push(self.to_snake_case(&pkg_name.namespace));
                        segments.push(self.to_snake_case(&pkg_name.name));
                        segments.push(self.to_snake_case(interface_name));
                        segments.push(self.to_pascal_case(type_name));

                        return segments.join(".");
                    }
                }
            }
        }

        // Same interface or no interface context - use simple name
        self.to_pascal_case(type_name)
    }

    /// Render a WIT type to its Scala equivalent with fully qualified names.
    pub fn render_type(&mut self, resolve: &Resolve, ty: &Type) -> String {
        match ty {
            // Primitive types - delegate to render_primitive_type
            Type::Bool
            | Type::S8
            | Type::U8
            | Type::S16
            | Type::U16
            | Type::S32
            | Type::U32
            | Type::S64
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Char
            | Type::String => self.render_primitive_type(ty).to_string(),
            Type::Id(id) => self.render_type_id(resolve, *id),
            Type::ErrorContext => panic!("ErrorContext type is not supported"),
        }
    }

    /// Render a type ID reference with fully qualified name.
    fn render_type_id(&mut self, resolve: &Resolve, id: TypeId) -> String {
        let ty = &resolve.types[id];

        // Check what kind of type this is
        match &ty.kind {
            TypeDefKind::List(inner) => {
                // list<T> maps to Array[T]
                format!("Array[{}]", self.render_type(resolve, inner))
            }
            TypeDefKind::Option(inner) => {
                // option<T> maps to java.util.Optional[T]
                format!("java.util.Optional[{}]", self.render_type(resolve, inner))
            }
            TypeDefKind::Result(result) => {
                // result<T, E> maps to scala.scalajs.wit.Result[T, E]
                let ok_type = result
                    .ok
                    .as_ref()
                    .map(|t| self.render_type(resolve, t))
                    .unwrap_or_else(|| "Unit".to_string());
                let err_type = result
                    .err
                    .as_ref()
                    .map(|t| self.render_type(resolve, t))
                    .unwrap_or_else(|| "Unit".to_string());
                format!("scala.scalajs.wit.Result[{}, {}]", ok_type, err_type)
            }
            TypeDefKind::Tuple(tuple) => {
                // tuple<T1, T2, ...> maps to scala.scalajs.wit.TupleN[...]
                let type_params: Vec<String> = tuple
                    .types
                    .iter()
                    .map(|t| self.render_type(resolve, t))
                    .collect();
                format!(
                    "scala.scalajs.wit.Tuple{}[{}]",
                    type_params.len(),
                    type_params.join(", ")
                )
            }
            TypeDefKind::Record(_)
            | TypeDefKind::Variant(_)
            | TypeDefKind::Enum(_)
            | TypeDefKind::Flags(_) => {
                // Named types - use qualified name if from different interface
                let type_name = ty.name.as_ref().expect("Named types must have a name");
                self.get_qualified_type_name(resolve, id, type_name)
            }
            TypeDefKind::Type(inner) => {
                // Type alias - render the underlying type
                self.render_type(resolve, inner)
            }
            TypeDefKind::Handle(handle) => {
                // Handle to a resource - follow the reference to get the resource name
                use wit_bindgen_core::wit_parser::Handle;
                let resource_id = match handle {
                    Handle::Own(id) | Handle::Borrow(id) => *id,
                };
                let resource_ty = &resolve.types[resource_id];
                let type_name = resource_ty
                    .name
                    .as_ref()
                    .expect("Resources must have a name");
                self.get_qualified_type_name(resolve, resource_id, type_name)
            }
            TypeDefKind::Resource => {
                // Resource definition - use qualified name if from different interface
                let type_name = ty.name.as_ref().expect("Resources must have a name");
                self.get_qualified_type_name(resolve, id, type_name)
            }
            TypeDefKind::FixedSizeList(inner, _size) => {
                // Fixed-size list also maps to Array[T]
                format!("Array[{}]", self.render_type(resolve, inner))
            }
            TypeDefKind::Future(_) | TypeDefKind::Stream(_) | TypeDefKind::Unknown => {
                "Unknown".to_string()
            }
        }
    }

    /// Render a WIT primitive type to its Scala equivalent.
    ///
    /// This returns non-fully qualified names for primitive types and fully qualified names
    /// for unsigned types from the scala.scalajs.component.unsigned package.
    pub fn render_primitive_type(&mut self, ty: &Type) -> &'static str {
        match ty {
            Type::Bool => "Boolean",
            Type::S8 => "Byte",
            Type::U8 => "scala.scalajs.wit.unsigned.UByte",
            Type::S16 => "Short",
            Type::U16 => "scala.scalajs.wit.unsigned.UShort",
            Type::S32 => "Int",
            Type::U32 => "scala.scalajs.wit.unsigned.UInt",
            Type::S64 => "Long",
            Type::U64 => "scala.scalajs.wit.unsigned.ULong",
            Type::F32 => "Float",
            Type::F64 => "Double",
            Type::Char => "Char",
            Type::String => "String",
            _ => unreachable!("Not a primitive type: {:?}", ty),
        }
    }

    /// Render a typedef (record, variant, enum, flags, etc.) to Scala code.
    pub fn render_typedef(&mut self, resolve: &Resolve, id: TypeId) -> String {
        let ty = &resolve.types[id];
        let name = ty.name.as_ref().expect("Type must have a name");
        let type_name = self.to_pascal_case(name);

        match &ty.kind {
            TypeDefKind::Record(record) => self.render_record(&type_name, record, resolve, &ty.docs),
            TypeDefKind::Variant(variant) => self.render_variant(&type_name, variant, resolve, &ty.docs),
            TypeDefKind::Enum(enum_) => self.render_enum(&type_name, enum_, &ty.docs),
            TypeDefKind::Flags(flags) => self.render_flags(&type_name, flags, &ty.docs),
            TypeDefKind::Tuple(tuple) => self.render_tuple_typedef(&type_name, tuple, resolve),
            TypeDefKind::Option(inner) => self.render_option_typedef(&type_name, inner, resolve),
            TypeDefKind::Result(result) => self.render_result_typedef(&type_name, result, resolve),
            TypeDefKind::List(inner) => self.render_list_typedef(&type_name, inner, resolve),
            TypeDefKind::Type(inner) => {
                // Type alias
                format!("type {} = {}", type_name, self.render_type(resolve, inner))
            }
            TypeDefKind::Handle(_handle) => {
                // Resources are handled separately
                format!("// Resource: {}", type_name)
            }
            TypeDefKind::Resource => {
                // Resources are handled separately
                format!("// Resource: {}", type_name)
            }
            TypeDefKind::FixedSizeList(inner, size) => {
                // Fixed-size lists map to Array
                format!(
                    "type {} = Array[{}] // Fixed size: {}",
                    type_name,
                    self.render_type(resolve, inner),
                    size
                )
            }
            TypeDefKind::Future(_) | TypeDefKind::Stream(_) | TypeDefKind::Unknown => {
                panic!("Unsupported type: {:?}", ty.kind)
            }
        }
    }

    /// Render a record type as a Scala case class.
    fn render_record(&mut self, name: &str, record: &Record, resolve: &Resolve, type_docs: &Docs) -> String {
        let mut output = String::new();

        // Generate scaladoc if docs exist
        let docs = format_docs(type_docs);
        if !docs.is_empty() {
            write!(&mut output, "{}", docs).unwrap();
        }

        writeln!(&mut output, "{}", annotations::component_record()).unwrap();
        write!(&mut output, "final case class {}(", name).unwrap();

        for (i, field) in record.fields.iter().enumerate() {
            if i > 0 {
                write!(&mut output, ", ").unwrap();
            }
            let field_name = self.to_camel_case(&field.name);
            let field_type = self.render_type(resolve, &field.ty);
            write!(&mut output, "{}: {}", field_name, field_type).unwrap();
        }

        writeln!(&mut output, ")").unwrap();
        output
    }

    /// Render a variant type as a Scala sealed trait with case classes.
    fn render_variant(&mut self, name: &str, variant: &Variant, resolve: &Resolve, type_docs: &Docs) -> String {
        let mut output = String::new();

        // Generate scaladoc if docs exist
        let docs = format_docs(type_docs);
        if !docs.is_empty() {
            write!(&mut output, "{}", docs).unwrap();
        }

        writeln!(&mut output, "{}", annotations::component_variant()).unwrap();
        writeln!(&mut output, "sealed trait {}", name).unwrap();
        writeln!(&mut output, "object {} {{", name).unwrap();

        for case in &variant.cases {
            let case_name = self.to_pascal_case(&case.name);
            match &case.ty {
                Some(ty) => {
                    let case_type = self.render_type(resolve, ty);
                    writeln!(
                        &mut output,
                        "  final case class {}(value: {}) extends {}",
                        case_name, case_type, name
                    )
                    .unwrap();
                }
                None => {
                    writeln!(&mut output, "  case object {} extends {}", case_name, name).unwrap();
                }
            }
        }

        writeln!(&mut output, "}}").unwrap();
        output
    }

    /// Render an enum type as a Scala sealed trait with case objects.
    fn render_enum(&mut self, name: &str, enum_: &Enum, type_docs: &Docs) -> String {
        let mut output = String::new();

        // Generate scaladoc if docs exist
        let docs = format_docs(type_docs);
        if !docs.is_empty() {
            write!(&mut output, "{}", docs).unwrap();
        }

        writeln!(&mut output, "{}", annotations::component_variant()).unwrap();
        writeln!(&mut output, "sealed trait {}", name).unwrap();
        writeln!(&mut output, "object {} {{", name).unwrap();

        for case in &enum_.cases {
            let case_name = self.to_pascal_case(&case.name);
            writeln!(&mut output, "  case object {} extends {}", case_name, name).unwrap();
        }

        writeln!(&mut output, "}}").unwrap();
        output
    }

    /// Render a flags type as a Scala case class with bitwise operators.
    fn render_flags(&mut self, name: &str, flags: &Flags, type_docs: &Docs) -> String {
        let mut output = String::new();

        // Generate scaladoc if docs exist
        let docs = format_docs(type_docs);
        if !docs.is_empty() {
            write!(&mut output, "{}", docs).unwrap();
        }

        writeln!(
            &mut output,
            "{}",
            annotations::component_flags(flags.flags.len())
        )
        .unwrap();
        writeln!(&mut output, "final case class {}(value: Int) {{", name).unwrap();
        writeln!(
            &mut output,
            "  def |(other: {}): {} = {}(value | other.value)",
            name, name, name
        )
        .unwrap();
        writeln!(
            &mut output,
            "  def &(other: {}): {} = {}(value & other.value)",
            name, name, name
        )
        .unwrap();
        writeln!(
            &mut output,
            "  def ^(other: {}): {} = {}(value ^ other.value)",
            name, name, name
        )
        .unwrap();
        writeln!(&mut output, "  def unary_~ : {} = {}(~value)", name, name).unwrap();
        writeln!(
            &mut output,
            "  def contains(other: {}): Boolean = (value & other.value) == other.value",
            name
        )
        .unwrap();
        writeln!(&mut output, "}}").unwrap();

        writeln!(&mut output, "object {} {{", name).unwrap();
        for (i, flag) in flags.flags.iter().enumerate() {
            let flag_name = self.to_camel_case(&flag.name);
            writeln!(&mut output, "  val {} = {}(1 << {})", flag_name, name, i).unwrap();
        }
        writeln!(&mut output, "}}").unwrap();

        output
    }

    /// Render a tuple type reference.
    fn render_tuple_typedef(&mut self, name: &str, tuple: &Tuple, resolve: &Resolve) -> String {
        let mut type_params = String::new();
        for (i, ty) in tuple.types.iter().enumerate() {
            if i > 0 {
                type_params.push_str(", ");
            }
            type_params.push_str(&self.render_type(resolve, ty));
        }
        format!(
            "type {} = scala.scalajs.wit.Tuple{}[{}]",
            name,
            tuple.types.len(),
            type_params
        )
    }

    /// Render an option type reference.
    fn render_option_typedef(&mut self, name: &str, inner: &Type, resolve: &Resolve) -> String {
        format!(
            "type {} = java.util.Optional[{}]",
            name,
            self.render_type(resolve, inner)
        )
    }

    /// Render a result type reference.
    fn render_result_typedef(&mut self, name: &str, result: &Result_, resolve: &Resolve) -> String {
        let ok_type = result
            .ok
            .as_ref()
            .map(|t| self.render_type(resolve, t))
            .unwrap_or_else(|| "Unit".to_string());
        let err_type = result
            .err
            .as_ref()
            .map(|t| self.render_type(resolve, t))
            .unwrap_or_else(|| "Unit".to_string());
        format!(
            "type {} = scala.scalajs.wit.Result[{}, {}]",
            name, ok_type, err_type
        )
    }

    /// Render a list type reference.
    fn render_list_typedef(&mut self, name: &str, inner: &Type, resolve: &Resolve) -> String {
        format!(
            "type {} = Array[{}]",
            name,
            self.render_type(resolve, inner)
        )
    }

    /// Escape Scala keywords by wrapping them in backticks.
    pub fn escape_keyword(&self, name: &str) -> String {
        if self.keywords.is_keyword(name) {
            format!("`{}`", name)
        } else {
            name.to_string()
        }
    }

    /// Convert a kebab-case name to camelCase (for method names, variables).
    pub fn to_camel_case(&self, name: &str) -> String {
        self.escape_keyword(&name.to_lower_camel_case())
    }

    /// Convert a kebab-case name to PascalCase (for type names, constructors).
    pub fn to_pascal_case(&self, name: &str) -> String {
        self.escape_keyword(&name.to_pascal_case())
    }

    /// Convert a kebab-case name to snake_case (for package names, file names).
    pub fn to_snake_case(&self, name: &str) -> String {
        name.to_snake_case()
    }

    /// Get the base package segments.
    pub fn base_package_segments(&self) -> Vec<String> {
        self.opts
            .base_package
            .split('.')
            .map(|s| s.to_string())
            .collect()
    }

    /// Render a function signature with annotation (import or export).
    pub fn render_function(
        &mut self,
        resolve: &Resolve,
        func: &Function,
        is_import: bool,
        namespace: &str,
    ) -> String {
        let func_name = self.to_camel_case(&func.name);
        let wit_name = &func.name;

        // Generate scaladoc if docs exist
        let docs = format_docs(&func.docs);

        // Collect parameters
        let mut params = Vec::new();
        for (param_name, param_ty) in &func.params {
            let scala_param_name = self.to_camel_case(param_name);
            let scala_param_type = self.render_type(resolve, param_ty);
            params.push((scala_param_name, scala_param_type));
        }

        // Render return type
        let return_type = func.result.as_ref().map(|ty| self.render_type(resolve, ty));

        if is_import {
            annotations::import_function(
                namespace,
                wit_name,
                &func_name,
                &params,
                return_type.as_deref(),
                &docs,
            )
        } else {
            annotations::export_function(
                namespace,
                wit_name,
                &func_name,
                &params,
                return_type.as_deref(),
                &docs,
            )
        }
    }
}

/// Scala keywords that need to be escaped.
struct ScalaKeywords {
    keywords: HashSet<&'static str>,
}

impl ScalaKeywords {
    fn new() -> Self {
        let mut keywords = HashSet::new();

        // Scala keywords (Scala 2 + future-proofing with Scala 3 keywords)
        keywords.extend([
            // Keywords
            "abstract",
            "case",
            "catch",
            "class",
            "def",
            "do",
            "else",
            "extends",
            "false",
            "final",
            "finally",
            "for",
            "forSome",
            "if",
            "implicit",
            "import",
            "lazy",
            "match",
            "new",
            "null",
            "object",
            "override",
            "package",
            "private",
            "protected",
            "return",
            "sealed",
            "super",
            "this",
            "throw",
            "trait",
            "true",
            "try",
            "type",
            "val",
            "var",
            "while",
            "with",
            "yield",
            // Scala 3 keywords (future-proofing)
            "enum",
            "export",
            "given",
            "then",
            // Soft keywords
            "as",
            "derives",
            "end",
            "extension",
            "infix",
            "inline",
            "opaque",
            "open",
            "transparent",
            "using",
            // Reserved words
            "_",
            ":",
            "=",
            "=>",
            "<-",
            "<:",
            "<%",
            ">:",
            "#",
            "@",
            // Common method names that might conflict
            "equals",
            "hashCode",
            "toString",
            "wait",
            "notify",
            "notifyAll",
            "clone",
            "finalize",
            "getClass",
        ]);

        Self { keywords }
    }

    fn is_keyword(&self, name: &str) -> bool {
        self.keywords.contains(name)
    }
}
