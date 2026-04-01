use crate::ast::*;
use crate::desugaring::DesugarContext;
use crate::error::{CompilerError, Result, SemanticErrorInfo};
use crate::traits::TraitRegistry;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Capitalize the first letter of a string
fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Information about a pending Task (async/par) that hasn't been awaited yet
#[derive(Debug, Clone)]
struct TaskInfo {
    /// Whether this is error binding (two variables: value, err)
    is_error_binding: bool,
    /// The names of all variables in the binding (1 for simple, 2 for error binding)
    binding_names: Vec<String>,
    /// Whether the task has already been awaited
    awaited: bool,
    /// The execution policy (Async or Par)
    exec_policy: ExecPolicy,
    /// Whether the task returns a tuple directly (vs Result)
    returns_tuple: bool,
    /// Whether this is an HTTP call that returns (Option<T>, String) needing unwrap
    is_http_call: bool,
}

/// Tracks lifecycle hooks (beforeEach, afterEach, etc.) active at a given describe() scope.
#[derive(Debug, Clone, Default)]
struct TestHookScope {
    has_before_each: bool,
    has_after_each: bool,
    has_before_all: bool,
    has_after_all: bool,
    /// Whether beforeEach hook body contains async calls
    before_each_is_async: bool,
    /// Whether afterEach hook body contains async calls
    after_each_is_async: bool,
    /// Depth index — used to generate unique fn names for nested describes
    depth: usize,
}

pub struct CodeGenerator {
    pub(crate) output: String,
    indent_level: usize,
    ctx: DesugarContext,
    in_method: bool,
    current_class_name: Option<String>,
    current_method_is_mut: bool,
    /// B09: Pre-computed set of methods that need &mut self (direct + transitive)
    mut_self_methods: HashSet<String>,
    in_assignment_target: bool,
    in_fallible_function: bool,
    in_optional_function: bool, // BUG-006: Track if inside function returning T?
    in_test_block: bool,
    /// Stack of test lifecycle hooks per describe() scope (for auto-invocation)
    test_hooks_stack: Vec<TestHookScope>,
    in_string_template: bool, // Track if we're inside a string template
    bracket_notation_vars: std::collections::HashSet<String>,
    class_instance_vars: std::collections::HashSet<String>,
    array_vars: std::collections::HashSet<String>, // Track which variables are arrays
    map_vars: std::collections::HashSet<String>, // Track which variables are Map<K,V>
    set_vars: std::collections::HashSet<String>, // Track which variables are Set<T>
    json_value_vars: std::collections::HashSet<String>, // Track which variables are JsonValue
    string_vars: std::collections::HashSet<String>, // Track which variables are strings
    float_vars: std::collections::HashSet<String>, // Track which variables are floats (B32)
    date_vars: std::collections::HashSet<String>, // Track which variables are Date (chrono::NaiveDateTime)
    server_vars: std::collections::HashSet<String>, // Track which variables are HTTP Server (axum::Router)
    server_request_param: Option<String>, // Inside server handler, name of the request param (for req.params → __params)
    db_vars: std::collections::HashSet<String>, // Track which variables are DB connections (rusqlite::Connection)
    map_array_vars: std::collections::HashSet<String>, // Track Vec<HashMap<String,String>> vars (from DB.query, CSV.readTable)
    native_vec_string_vars: std::collections::HashSet<String>, // Track Vec<String> from Sys.args() - use direct indexing
    mutated_vars: std::collections::HashSet<String>, // Track variables that are assigned after declaration (need mut)
    // --- Class/type metadata (for field resolution)
    class_fields: std::collections::HashMap<String, std::collections::HashSet<String>>,
    class_optional_fields: std::collections::HashMap<String, std::collections::HashSet<String>>, // Track optional fields per class
    class_array_field_types: std::collections::HashMap<String, std::collections::HashMap<String, String>>, // className -> (fieldName -> elementType) for array fields
    var_types: std::collections::HashMap<String, String>, // var -> ClassName
    fallible_functions: std::collections::HashSet<String>, // Track which functions are fallible
    fallible_methods: std::collections::HashSet<String>, // B19 fix: Track which class methods are fallible (method_name, not qualified)
    array_returning_functions: std::collections::HashMap<String, String>, // Track functions that return [T] (Vec) -> elem type
    string_returning_functions: std::collections::HashSet<String>, // Track functions that return string (String)
    optional_returning_functions: std::collections::HashSet<String>, // BUG-007: Track functions that return T? (Option<T>)
    string_returning_methods: std::collections::HashSet<String>, // B100: Track class methods that return string (for VarDecl tracking)
    array_returning_methods: std::collections::HashMap<String, String>, // B100: Track class methods that return [T] (method_name -> elem_type)
    ref_lambda_params: std::collections::HashSet<String>, // Lambda params that are &T references (need *deref in comparisons)
    suppress_option_unwrap: bool, // When true, Option-returning methods (find/first/last/min/max) don't add .unwrap()
    suppress_map_get_unwrap: bool, // When true, Map.get() doesn't add .unwrap_or_default() (used in `or` path)
    // --- Type aliases (for expansion during codegen)
    type_aliases: std::collections::HashMap<String, (Vec<TypeParameter>, TypeRef)>,
    // --- Union types (for enum generation)
    union_types: std::collections::HashSet<Vec<String>>, // Track all union types used: [(i32, String), ...]
    // --- Phase 2: Lazy await/join tracking
    pending_tasks: std::collections::HashMap<String, TaskInfo>, // Variables that hold unawaited Tasks
    // --- Phase 3: Error binding variables (Option<String> type)
    error_binding_vars: std::collections::HashSet<String>, // Variables from error binding (second variable in let x, err = ...)
    error_binding_scope_stack: Vec<Vec<String>>, // B20: Stack of error binding vars per scope level (for fail scope checking)
    string_error_vars: std::collections::HashSet<String>, // String error variables from HTTP/File calls (for `if err` sugar)
    option_value_vars: std::collections::HashSet<String>, // Variables from error binding (first variable in let value, err = ..., which is Option<T>)
    struct_destructured_vars: std::collections::HashSet<String>, // Variables from struct destructuring (may be Option<T>)
    rust_struct_vars: std::collections::HashSet<String>, // Variables that are Rust structs (HTTP response, etc.), not JsonValue
    typed_array_vars: std::collections::HashMap<String, String>, // Track arrays with element type: var_name -> element_class_name (e.g., "posts" -> "Post")
    current_lambda_element_type: Option<String>, // Temporarily track element type when generating lambdas in forEach/map/etc
    // --- Constructor optional param tracking (for Some() wrapping at call sites)
    class_constructor_optionals: std::collections::HashMap<String, Vec<bool>>, // className -> [is_optional per field, in order]
    enum_variant_optionals: std::collections::HashMap<String, Vec<bool>>, // "EnumName::VariantName" -> [is_optional per field]
    // --- Phase 4: Join combining optimization
    #[allow(dead_code)]
    awaitable_tasks: Vec<String>, // Tasks that can be combined with tokio::join!
    // --- Phase 5: Generic constraints
    trait_registry: TraitRegistry, // Trait registry for constraint validation
    // --- Async user functions
    async_functions: std::collections::BTreeSet<String>, // User-defined async functions (BTreeSet from DesugarContext)
    // --- Phase 6: Interface method signatures (for type inference)
    interface_methods:
        std::collections::HashMap<String, std::collections::HashMap<String, TypeRef>>, // interface_name -> (method_name -> return_type)
    // --- Module aliases for wildcard imports (alias -> actual_module_name)
    module_aliases: std::collections::HashMap<String, String>,
    // --- Current function return type (for casting division results)
    current_return_type: Option<String>,
    // --- Enum metadata (enum_name -> variant_names with field info)
    enum_names: std::collections::HashSet<String>,
    enum_variants:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>, // enum_name -> (variant_name -> [field_names])
    /// Recursive enum fields that are auto-boxed: enum_name -> (variant_name -> set of boxed field names)
    boxed_enum_fields:
        std::collections::HashMap<String, std::collections::HashMap<String, std::collections::HashSet<String>>>,
    /// Enum variant field types: enum_name -> (variant_name -> [(field_name, type_ref)])
    enum_variant_field_types:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<(String, TypeRef)>>>,
    // --- B46: Classes that need serde derives (from JSON.stringify usage)
    serde_classes: std::collections::HashSet<String>,
    // --- Float literal suffix context (for f32 variable declarations)
    float_literal_suffix: String, // "f64" by default, set to "f32" when generating f32-typed expressions
    // --- Error trace context
    current_function_name: String,  // Current function/method name for error traces
    source_filename: String,        // Source filename for error traces
    /// Hoisted `use` statements extracted from `rust { }` blocks (emitted at top of file)
    rust_block_uses: Vec<String>,
    /// Counter for generating unique defer guard variable names
    defer_counter: usize,
    /// SH-002: When true, we're inside a constructor body — `this.field` maps to local vars
    in_constructor: bool,
    /// Default parameter values: function_name -> [(param_index, default_expr)]
    function_defaults: std::collections::HashMap<String, Vec<(usize, Expr)>>,
    /// B109: Track used test function names to avoid collisions
    used_test_names: std::collections::HashMap<String, usize>,
}

impl CodeGenerator {
    fn new(ctx: DesugarContext) -> Self {
        let async_funcs = ctx.async_functions.clone();
        let source_filename = ctx.source_filename.clone();
        Self {
            output: String::new(),
            indent_level: 0,
            ctx,
            in_method: false,
            current_class_name: None,
            current_method_is_mut: false,
            mut_self_methods: HashSet::new(),
            in_assignment_target: false,
            in_fallible_function: false,
            in_optional_function: false,
            in_test_block: false,
            in_string_template: false,
            bracket_notation_vars: std::collections::HashSet::new(),
            class_instance_vars: std::collections::HashSet::new(),
            array_vars: std::collections::HashSet::new(),
            map_vars: std::collections::HashSet::new(),
            set_vars: std::collections::HashSet::new(),
            json_value_vars: std::collections::HashSet::new(),
            string_vars: std::collections::HashSet::new(),
            float_vars: std::collections::HashSet::new(),
            date_vars: std::collections::HashSet::new(),
            server_vars: std::collections::HashSet::new(),
            server_request_param: None,
            db_vars: std::collections::HashSet::new(),
            map_array_vars: std::collections::HashSet::new(),
            native_vec_string_vars: std::collections::HashSet::new(),
            mutated_vars: std::collections::HashSet::new(),
            class_fields: std::collections::HashMap::new(),
            class_optional_fields: std::collections::HashMap::new(),
            class_array_field_types: std::collections::HashMap::new(),
            var_types: std::collections::HashMap::new(),
            fallible_functions: std::collections::HashSet::new(),
            fallible_methods: std::collections::HashSet::new(),
            array_returning_functions: std::collections::HashMap::new(),
            string_returning_functions: std::collections::HashSet::new(),
            optional_returning_functions: std::collections::HashSet::new(),
            string_returning_methods: std::collections::HashSet::new(),
            array_returning_methods: std::collections::HashMap::new(),
            ref_lambda_params: std::collections::HashSet::new(),
            suppress_option_unwrap: false,
            suppress_map_get_unwrap: false,
            type_aliases: std::collections::HashMap::new(),
            union_types: std::collections::HashSet::new(),
            pending_tasks: std::collections::HashMap::new(),
            error_binding_vars: std::collections::HashSet::new(),
            error_binding_scope_stack: vec![vec![]], // Start with one root scope
            string_error_vars: std::collections::HashSet::new(),
            option_value_vars: std::collections::HashSet::new(),
            struct_destructured_vars: std::collections::HashSet::new(),
            rust_struct_vars: std::collections::HashSet::new(),
            typed_array_vars: std::collections::HashMap::new(),
            current_lambda_element_type: None,
            class_constructor_optionals: std::collections::HashMap::new(),
            enum_variant_optionals: std::collections::HashMap::new(),
            awaitable_tasks: Vec::new(),
            trait_registry: TraitRegistry::new(),
            async_functions: async_funcs,
            interface_methods: std::collections::HashMap::new(),
            module_aliases: std::collections::HashMap::new(),
            current_return_type: None,
            test_hooks_stack: Vec::new(),
            enum_names: std::collections::HashSet::new(),
            enum_variants: std::collections::HashMap::new(),
            boxed_enum_fields: std::collections::HashMap::new(),
            enum_variant_field_types: std::collections::HashMap::new(),
            serde_classes: std::collections::HashSet::new(),
            float_literal_suffix: "f64".to_string(),
            current_function_name: String::new(),
            source_filename,
            rust_block_uses: Vec::new(),
            defer_counter: 0,
            in_constructor: false,
            function_defaults: std::collections::HashMap::new(),
            used_test_names: std::collections::HashMap::new(),
        }
    }

    fn is_class_instance(&self, var_name: &str) -> bool {
        // For method contexts (self), always use dot notation
        if var_name == "self" || var_name == "this" {
            return true;
        }

        let sanitized = self.sanitize_name(var_name);

        // Check if the variable was assigned from a class constructor
        self.class_instance_vars.contains(&sanitized)
        // B07 fix: Also check var_types — tracks variables with known class types
        || self.var_types.contains_key(&sanitized)
        // Temporary heuristic: single character variables are likely class instances
        || var_name.len() == 1
    }

    /// Register enum pattern bindings as class instances so member access generates
    /// `.field` instead of `get_field()`. Returns the bindings that were registered
    /// (for cleanup after the arm body).
    fn register_pattern_bindings(&mut self, pattern: &Pattern) -> Vec<String> {
        let mut registered = Vec::new();
        if let Pattern::EnumVariant {
            enum_name,
            variant_name,
            bindings,
        } = pattern
        {
            if let Some(variant_map) = self.enum_variant_field_types.get(enum_name) {
                if let Some(field_types) = variant_map.get(variant_name) {
                    for (i, binding) in bindings.iter().enumerate() {
                        if binding == "_" {
                            continue;
                        }
                        if let Some((_, type_ref)) = field_types.get(i) {
                            // Check if the field type is a class/struct (starts with uppercase
                            // and is known in class_fields or enum_names)
                            let type_name = match type_ref {
                                TypeRef::Simple(name) => Some(name.as_str()),
                                TypeRef::Optional(inner) => {
                                    if let TypeRef::Simple(name) = inner.as_ref() {
                                        Some(name.as_str())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                            if let Some(name) = type_name {
                                let first_char = name.chars().next().unwrap_or('a');
                                if first_char.is_uppercase()
                                    && (self.class_fields.contains_key(name)
                                        || self.enum_names.contains(name))
                                {
                                    let sanitized = self.sanitize_name(binding);
                                    self.class_instance_vars.insert(sanitized.clone());
                                    registered.push(sanitized);
                                }
                            }
                        }
                    }
                }
            }
        }
        registered
    }

    /// Unregister pattern bindings after an arm body to avoid polluting outer scope
    fn unregister_pattern_bindings(&mut self, bindings: &[String]) {
        for binding in bindings {
            self.class_instance_vars.remove(binding);
        }
    }

    /// Check if an expression is a JsonValue (for lambda pattern detection)
    /// Returns true for both JsonValue direct and Vec<JsonValue>
    fn is_json_value_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(var_name) => self.json_value_vars.contains(var_name),
            Expr::MethodCall(mc) => {
                // If the method returns a JsonValue (e.g., .get_field(), .get())
                matches!(mc.method.as_str(), "get" | "get_field")
                    || self.is_json_value_expr(&mc.object)
            }
            _ => false,
        }
    }

    /// Check if an expression is a DIRECT JsonValue (not Vec<JsonValue>)
    /// Direct means: from JSON.parse(), .get(), .get_field()
    /// Not from: .map(), .filter() (those return Vec<JsonValue>)
    fn is_direct_json_value(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(var_name) => {
                // Check if it's in json_value_vars AND not in array_vars
                // (array_vars includes Vec<JsonValue> from map/filter)
                self.json_value_vars.contains(var_name) && !self.array_vars.contains(var_name)
            }
            Expr::MethodCall(mc) => {
                // Only .get() and .get_field() return direct JsonValue
                matches!(mc.method.as_str(), "get" | "get_field")
            }
            _ => false,
        }
    }

    /// Check if expression is JSON.parse method call (Phase 1: JSON Typed Parsing)
    fn is_json_parse_call(&self, expr: &Expr) -> bool {
        match expr {
            // JSON.parse() call
            Expr::MethodCall(mc)
                if matches!(&*mc.object, Expr::Identifier(id) if id == "JSON")
                    && mc.method == "parse" =>
            {
                true
            }
            // response.json() or any object's .json() method
            Expr::MethodCall(mc) if mc.method == "json" => true,
            _ => false,
        }
    }

    /// Check if expression is JSON.stringify method call (returns tuple)
    fn is_json_stringify_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(mc)
                if matches!(&*mc.object, Expr::Identifier(id) if id == "JSON")
                    && mc.method == "stringify" =>
            {
                true
            }
            _ => false,
        }
    }

    /// Check if expression is an HTTP call (GET/POST/PUT/DELETE)
    fn is_http_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if callee is HTTP method call (async HTTP.get, etc.)
                if let Expr::MethodCall(mc) = call.callee.as_ref() {
                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                        return (obj == "HTTP" || obj == "Http")
                            && matches!(mc.method.as_str(), "get" | "post" | "put" | "delete");
                    }
                }
                false
            }
            Expr::MethodCall(mc) => {
                if let Expr::Identifier(obj) = mc.object.as_ref() {
                    return (obj == "HTTP" || obj == "Http")
                        && matches!(mc.method.as_str(), "get" | "post" | "put" | "delete");
                }
                false
            }
            _ => false,
        }
    }

    /// Check if expression is a File call (read/write/append/delete) or Dir.list call
    fn is_file_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(mc) => {
                if let Expr::Identifier(obj) = mc.object.as_ref() {
                    if obj == "File" {
                        return matches!(mc.method.as_str(), "read" | "write" | "append" | "delete" | "copy" | "move" | "size" | "readLines" | "writeLines");
                    }
                    if obj == "Dir" {
                        return matches!(mc.method.as_str(), "list" | "create" | "delete" | "listRecursive" | "walk");
                    }
                    if obj == "Config" {
                        return matches!(mc.method.as_str(), "get" | "getInt" | "getBool" | "load");
                    }
                    if obj == "Regex" {
                        return matches!(mc.method.as_str(), "match");
                    }
                    if obj == "Date" {
                        return matches!(mc.method.as_str(), "parse");
                    }
                    if obj == "CSV" {
                        return matches!(mc.method.as_str(), "read" | "write" | "readTable" | "writeTable");
                    }
                    if obj == "Process" {
                        return matches!(mc.method.as_str(), "exec" | "spawn");
                    }
                    if obj == "Crypto" {
                        return matches!(mc.method.as_str(), "base64Decode");
                    }
                    if obj == "DB" {
                        return matches!(mc.method.as_str(), "open" | "exec" | "query");
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if expression is an await of a pending HTTP task
    /// Returns the task variable name if it's an HTTP task await
    fn is_await_http_task(&self, expr: &Expr) -> Option<String> {
        if let Expr::Unary {
            op: crate::ast::UnOp::Await,
            operand,
        } = expr
        {
            if let Expr::Identifier(name) = operand.as_ref() {
                let sanitized = self.sanitize_name(name);
                if let Some(task_info) = self.pending_tasks.get(&sanitized) {
                    if task_info.is_http_call {
                        return Some(sanitized);
                    }
                }
            }
        }
        None
    }

    /// Check if an async expression contains an HTTP call
    /// e.g., async HTTP.get(url) -> true
    fn is_http_call_in_async(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if callee is HTTP method call
                if let Expr::MethodCall(mc) = call.callee.as_ref() {
                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                        return (obj == "HTTP" || obj == "Http")
                            && matches!(mc.method.as_str(), "get" | "post" | "put" | "delete");
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Bug #45-46: Detect which type parameters need Clone bound
    /// Returns set of type param names that need Clone bound based on usage
    fn infer_type_param_bounds(
        &self,
        class: &ClassDecl,
    ) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
        let mut bounds: std::collections::HashMap<String, std::collections::HashSet<String>> =
            std::collections::HashMap::new();

        // Initialize bounds for each type param
        for tp in &class.type_params {
            bounds.insert(tp.name.clone(), std::collections::HashSet::new());
        }

        // Get set of type param names for quick lookup
        let type_param_names: std::collections::HashSet<String> =
            class.type_params.iter().map(|tp| tp.name.clone()).collect();

        // Get field types to detect generic fields (only fields with explicit types)
        let generic_fields: std::collections::HashMap<String, &TypeRef> = class
            .members
            .iter()
            .filter_map(|m| {
                if let Member::Field(f) = m {
                    // Only include fields that have explicit type annotations
                    f.type_ref.as_ref().map(|t| (f.name.clone(), t))
                } else {
                    None
                }
            })
            .collect();

        // Analyze methods for patterns that need Clone/Display bounds
        for member in &class.members {
            if let Member::Method(method) = member {
                // Bug #45-46: If method returns T and accesses this.field or this.items[i]
                // where the field type involves T, we need Clone bound
                if let Some(return_type) = &method.return_type {
                    // Check if return type is or contains a type param
                    for tp_name in &type_param_names {
                        if self.type_contains_param(return_type, tp_name) {
                            // Check if method body accesses self fields that need clone
                            if self.method_returns_self_field_of_type(
                                method,
                                tp_name,
                                &generic_fields,
                            ) {
                                bounds.get_mut(tp_name).unwrap().insert("Clone".to_string());
                            }
                        }
                    }
                }

                // Bug #54: Check for string templates using type params
                if let Some(body) = &method.body {
                    for tp_name in &type_param_names {
                        if self.block_uses_type_in_template(body, tp_name, &generic_fields) {
                            bounds
                                .get_mut(tp_name)
                                .unwrap()
                                .insert("std::fmt::Display".to_string());
                        }
                    }
                }
                if let Some(expr_body) = &method.expr_body {
                    for tp_name in &type_param_names {
                        if self.expr_uses_type_in_template(expr_body, tp_name, &generic_fields) {
                            bounds
                                .get_mut(tp_name)
                                .unwrap()
                                .insert("std::fmt::Display".to_string());
                        }
                    }
                }
            }
        }

        bounds
    }

    /// Check if a TypeRef contains a type parameter
    fn type_contains_param(&self, type_ref: &TypeRef, param_name: &str) -> bool {
        match type_ref {
            TypeRef::Simple(name) => name == param_name,
            TypeRef::Array(inner) => self.type_contains_param(inner, param_name),
            TypeRef::Optional(inner) => self.type_contains_param(inner, param_name),
            TypeRef::Fallible(inner) => self.type_contains_param(inner, param_name),
            TypeRef::Tuple(elems) => elems
                .iter()
                .any(|e| self.type_contains_param(e, param_name)),
            TypeRef::Generic { base, args } => {
                base == param_name || args.iter().any(|a| self.type_contains_param(a, param_name))
            }
            TypeRef::Union(variants) => variants
                .iter()
                .any(|v| self.type_contains_param(v, param_name)),
            TypeRef::Map(key, value) => {
                self.type_contains_param(key, param_name)
                    || self.type_contains_param(value, param_name)
            }
            TypeRef::Set(inner) => self.type_contains_param(inner, param_name),
        }
    }

    /// Check if method returns from a self field that involves the type parameter
    fn method_returns_self_field_of_type(
        &self,
        method: &MethodDecl,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        if let Some(body) = &method.body {
            return self.block_returns_self_field_of_type(body, type_param, generic_fields);
        }
        if let Some(expr) = &method.expr_body {
            return self.expr_returns_self_field_of_type(expr, type_param, generic_fields);
        }
        false
    }

    fn block_returns_self_field_of_type(
        &self,
        block: &BlockStmt,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        for stmt in &block.stmts {
            if self.stmt_returns_self_field_of_type(stmt, type_param, generic_fields) {
                return true;
            }
        }
        false
    }

    fn stmt_returns_self_field_of_type(
        &self,
        stmt: &Stmt,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        match stmt {
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    return self.expr_returns_self_field_of_type(expr, type_param, generic_fields);
                }
                false
            }
            Stmt::If(if_stmt) => {
                let then_returns = match &if_stmt.then_branch {
                    IfBody::Block(b) => {
                        self.block_returns_self_field_of_type(b, type_param, generic_fields)
                    }
                    IfBody::Stmt(s) => {
                        self.stmt_returns_self_field_of_type(s, type_param, generic_fields)
                    }
                };
                let else_returns = if_stmt.else_branch.as_ref().map_or(false, |eb| match eb {
                    IfBody::Block(b) => {
                        self.block_returns_self_field_of_type(b, type_param, generic_fields)
                    }
                    IfBody::Stmt(s) => {
                        self.stmt_returns_self_field_of_type(s, type_param, generic_fields)
                    }
                });
                then_returns || else_returns
            }
            _ => false,
        }
    }

    fn expr_returns_self_field_of_type(
        &self,
        expr: &Expr,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        match expr {
            // Direct field access: this.value where value: T
            Expr::Member { object, property } => {
                if let Expr::Identifier(obj) = object.as_ref() {
                    if obj == "this" || obj == "self" {
                        // Check if this field has a type involving the type param
                        if let Some(field_type) = generic_fields.get(property) {
                            return self.type_contains_param(field_type, type_param);
                        }
                    }
                }
                false
            }
            // Array indexing: this.items[i] where items: [T]
            Expr::Index { object, .. } => {
                if let Expr::Member {
                    object: base,
                    property,
                } = object.as_ref()
                {
                    if let Expr::Identifier(obj) = base.as_ref() {
                        if obj == "this" || obj == "self" {
                            if let Some(field_type) = generic_fields.get(property) {
                                // For arrays, check the element type
                                if let TypeRef::Array(elem_type) = field_type {
                                    return self.type_contains_param(elem_type, type_param);
                                }
                                return self.type_contains_param(field_type, type_param);
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Bug #54: Check if block uses type param in string templates
    fn block_uses_type_in_template(
        &self,
        block: &BlockStmt,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        for stmt in &block.stmts {
            if self.stmt_uses_type_in_template(stmt, type_param, generic_fields) {
                return true;
            }
        }
        false
    }

    fn stmt_uses_type_in_template(
        &self,
        stmt: &Stmt,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.expr_uses_type_in_template(&expr_stmt.expr, type_param, generic_fields)
            }
            Stmt::Return(ret) => ret.expr.as_ref().map_or(false, |e| {
                self.expr_uses_type_in_template(e, type_param, generic_fields)
            }),
            Stmt::VarDecl(var_decl) => {
                self.expr_uses_type_in_template(&var_decl.init, type_param, generic_fields)
            }
            Stmt::If(if_stmt) => {
                let cond =
                    self.expr_uses_type_in_template(&if_stmt.condition, type_param, generic_fields);
                let then_branch = match &if_stmt.then_branch {
                    IfBody::Block(b) => {
                        self.block_uses_type_in_template(b, type_param, generic_fields)
                    }
                    IfBody::Stmt(s) => {
                        self.stmt_uses_type_in_template(s, type_param, generic_fields)
                    }
                };
                let else_branch = if_stmt.else_branch.as_ref().map_or(false, |eb| match eb {
                    IfBody::Block(b) => {
                        self.block_uses_type_in_template(b, type_param, generic_fields)
                    }
                    IfBody::Stmt(s) => {
                        self.stmt_uses_type_in_template(s, type_param, generic_fields)
                    }
                });
                cond || then_branch || else_branch
            }
            Stmt::While(while_stmt) => {
                self.expr_uses_type_in_template(&while_stmt.condition, type_param, generic_fields)
                    || self.block_uses_type_in_template(
                        &while_stmt.body,
                        type_param,
                        generic_fields,
                    )
            }
            Stmt::For(for_stmt) => {
                self.expr_uses_type_in_template(&for_stmt.iterable, type_param, generic_fields)
                    || self.block_uses_type_in_template(&for_stmt.body, type_param, generic_fields)
            }
            _ => false,
        }
    }

    fn expr_uses_type_in_template(
        &self,
        expr: &Expr,
        type_param: &str,
        generic_fields: &std::collections::HashMap<String, &TypeRef>,
    ) -> bool {
        match expr {
            Expr::StringTemplate { parts } => {
                // Check if any part references a generic field
                for part in parts {
                    if let StringTemplatePart::Expr(inner_expr) = part {
                        if let Expr::Member { object, property } = inner_expr.as_ref() {
                            if let Expr::Identifier(obj) = object.as_ref() {
                                if obj == "this" || obj == "self" {
                                    if let Some(field_type) = generic_fields.get(property.as_str())
                                    {
                                        if self.type_contains_param(field_type, type_param) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                        // Also check for array indexing in template
                        if let Expr::Index { object, .. } = inner_expr.as_ref() {
                            if let Expr::Member {
                                object: base,
                                property,
                            } = object.as_ref()
                            {
                                if let Expr::Identifier(obj) = base.as_ref() {
                                    if obj == "this" || obj == "self" {
                                        if let Some(field_type) =
                                            generic_fields.get(property.as_str())
                                        {
                                            if let TypeRef::Array(elem_type) = field_type {
                                                if self.type_contains_param(elem_type, type_param) {
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }
            Expr::Call(call) => {
                self.expr_uses_type_in_template(&call.callee, type_param, generic_fields)
                    || call
                        .args
                        .iter()
                        .any(|a| self.expr_uses_type_in_template(a, type_param, generic_fields))
            }
            Expr::MethodCall(mc) => {
                self.expr_uses_type_in_template(&mc.object, type_param, generic_fields)
                    || mc
                        .args
                        .iter()
                        .any(|a| self.expr_uses_type_in_template(a, type_param, generic_fields))
            }
            Expr::Binary { left, right, .. } => {
                self.expr_uses_type_in_template(left, type_param, generic_fields)
                    || self.expr_uses_type_in_template(right, type_param, generic_fields)
            }
            _ => false,
        }
    }

    /// Check if a method modifies self fields (requires &mut self)
    fn method_modifies_self(&self, method: &MethodDecl) -> bool {
        // Check pre-computed transitive set first
        if self.mut_self_methods.contains(&method.name) {
            return true;
        }
        if let Some(body) = &method.body {
            return self.block_modifies_self(body);
        }
        false
    }

    /// B09: Pre-compute which methods need &mut self, including transitive calls.
    /// B46: Scan all function/method bodies for JSON.stringify(arg) to identify classes needing serde derives.
    /// Builds var→class map from constructor calls, then finds JSON.stringify(var) usage.
    fn scan_json_stringify_classes(&mut self, program: &Program) {
        // Collect known class names
        let class_names: std::collections::HashSet<String> = program.items.iter().filter_map(|item| {
            if let TopLevel::Class(cls) = item { Some(cls.name.clone()) } else { None }
        }).collect();

        // Scan all function and method bodies
        for item in &program.items {
            match item {
                TopLevel::Function(func) => {
                    if let Some(body) = &func.body {
                        self.scan_stmts_for_stringify(&body.stmts, &class_names);
                    }
                }
                TopLevel::Class(cls) => {
                    for member in &cls.members {
                        if let Member::Method(method) = member {
                            if let Some(body) = &method.body {
                                self.scan_stmts_for_stringify(&body.stmts, &class_names);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Scan statements for JSON.stringify calls with class-typed arguments
    fn scan_stmts_for_stringify(&mut self, stmts: &[Stmt], class_names: &std::collections::HashSet<String>) {
        // Build local var→class map from this scope
        let mut var_class: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for stmt in stmts {
            // Track constructor assignments: let x = ClassName(...)
            if let Stmt::VarDecl(var) = stmt {
                if let Expr::Call(call) = &var.init {
                    if let Expr::Identifier(name) = call.callee.as_ref() {
                        if class_names.contains(name) {
                            for binding in &var.bindings {
                                if let BindingPattern::Identifier(var_name) = &binding.pattern {
                                    var_class.insert(var_name.clone(), name.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Find JSON.stringify(arg) calls
            self.scan_expr_for_stringify_in_stmt(stmt, &var_class, class_names);
        }
    }

    fn scan_expr_for_stringify_in_stmt(&mut self, stmt: &Stmt, var_class: &std::collections::HashMap<String, String>, class_names: &std::collections::HashSet<String>) {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.scan_expr_for_stringify(&expr_stmt.expr, var_class, class_names);
            }
            Stmt::Return(ret_stmt) => {
                if let Some(expr) = &ret_stmt.expr {
                    self.scan_expr_for_stringify(expr, var_class, class_names);
                }
            }
            Stmt::VarDecl(var) => {
                self.scan_expr_for_stringify(&var.init, var_class, class_names);
            }
            Stmt::Assign(assign) => {
                self.scan_expr_for_stringify(&assign.value, var_class, class_names);
            }
            Stmt::If(if_stmt) => {
                self.scan_expr_for_stringify(&if_stmt.condition, var_class, class_names);
                match &if_stmt.then_branch {
                    IfBody::Block(block) => {
                        for s in &block.stmts {
                            self.scan_expr_for_stringify_in_stmt(s, var_class, class_names);
                        }
                    }
                    IfBody::Stmt(s) => {
                        self.scan_expr_for_stringify_in_stmt(s, var_class, class_names);
                    }
                }
                if let Some(else_branch) = &if_stmt.else_branch {
                    match else_branch {
                        IfBody::Block(block) => {
                            for s in &block.stmts {
                                self.scan_expr_for_stringify_in_stmt(s, var_class, class_names);
                            }
                        }
                        IfBody::Stmt(s) => {
                            self.scan_expr_for_stringify_in_stmt(s, var_class, class_names);
                        }
                    }
                }
            }
            Stmt::For(for_stmt) => {
                for s in &for_stmt.body.stmts {
                    self.scan_expr_for_stringify_in_stmt(s, var_class, class_names);
                }
            }
            _ => {}
        }
    }

    fn scan_expr_for_stringify(&mut self, expr: &Expr, var_class: &std::collections::HashMap<String, String>, class_names: &std::collections::HashSet<String>) {
        match expr {
            Expr::MethodCall(mc) => {
                if mc.method == "stringify" {
                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                        if obj == "JSON" {
                            // Found JSON.stringify(arg) — resolve arg's class type
                            for arg in &mc.args {
                                if let Expr::Identifier(var_name) = arg {
                                    if let Some(class_name) = var_class.get(var_name) {
                                        self.serde_classes.insert(class_name.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                // Recurse into subexpressions
                self.scan_expr_for_stringify(&mc.object, var_class, class_names);
                for arg in &mc.args {
                    self.scan_expr_for_stringify(arg, var_class, class_names);
                }
            }
            Expr::Call(call) => {
                self.scan_expr_for_stringify(&call.callee, var_class, class_names);
                for arg in &call.args {
                    self.scan_expr_for_stringify(arg, var_class, class_names);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.scan_expr_for_stringify(left, var_class, class_names);
                self.scan_expr_for_stringify(right, var_class, class_names);
            }
            _ => {}
        }
    }

    /// Phase 1: Detect direct &mut self (assignments to this.field, mutating methods on this.field)
    /// Phase 2: Propagate — if method A calls this.B() and B is &mut self, then A is too
    fn compute_mut_self_methods(&mut self, class: &ClassDecl) {
        self.mut_self_methods.clear();

        // Phase 1: Find directly mutating methods
        let methods: Vec<&MethodDecl> = class
            .members
            .iter()
            .filter_map(|m| {
                if let Member::Method(method) = m {
                    if method.name != "constructor" {
                        return Some(method);
                    }
                }
                None
            })
            .collect();

        for method in &methods {
            let is_setter = method.name.starts_with("set");
            let directly_modifies = if let Some(body) = &method.body {
                self.block_modifies_self(body)
            } else {
                false
            };
            if is_setter || directly_modifies {
                self.mut_self_methods.insert(method.name.clone());
            }
        }

        // Phase 2: Transitive propagation — iterate until no more changes
        loop {
            let mut changed = false;
            for method in &methods {
                if self.mut_self_methods.contains(&method.name) {
                    continue; // already marked
                }
                if let Some(body) = &method.body {
                    if self.block_calls_mut_self_method(body) {
                        self.mut_self_methods.insert(method.name.clone());
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// Check if a block calls any method on `this` that's in `mut_self_methods`
    fn block_calls_mut_self_method(&self, block: &BlockStmt) -> bool {
        for stmt in &block.stmts {
            if self.stmt_calls_mut_self_method(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_calls_mut_self_method(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => self.expr_calls_mut_self_method(&expr_stmt.expr),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map_or(false, |e| self.expr_calls_mut_self_method(e)),
            Stmt::VarDecl(var) => self.expr_calls_mut_self_method(&var.init),
            Stmt::Assign(assign) => {
                self.expr_calls_mut_self_method(&assign.value)
            }
            Stmt::If(if_stmt) => {
                let cond_calls = self.expr_calls_mut_self_method(&if_stmt.condition);
                let then_calls = match &if_stmt.then_branch {
                    IfBody::Block(b) => self.block_calls_mut_self_method(b),
                    IfBody::Stmt(s) => self.stmt_calls_mut_self_method(s),
                };
                let else_calls = if_stmt.else_branch.as_ref().map_or(false, |eb| match eb {
                    IfBody::Block(b) => self.block_calls_mut_self_method(b),
                    IfBody::Stmt(s) => self.stmt_calls_mut_self_method(s),
                });
                cond_calls || then_calls || else_calls
            }
            Stmt::While(w) => {
                self.expr_calls_mut_self_method(&w.condition)
                    || self.block_calls_mut_self_method(&w.body)
            }
            Stmt::For(f) => {
                self.expr_calls_mut_self_method(&f.iterable)
                    || self.block_calls_mut_self_method(&f.body)
            }
            Stmt::Switch(sw) => {
                self.expr_calls_mut_self_method(&sw.discriminant)
                    || sw.cases.iter().any(|case| case.body.iter().any(|s| self.stmt_calls_mut_self_method(s)))
                    || sw.default.as_ref().map_or(false, |d| d.iter().any(|s| self.stmt_calls_mut_self_method(s)))
            }
            Stmt::Defer(defer_stmt) => self.stmt_calls_mut_self_method(&defer_stmt.body),
            _ => false,
        }
    }

    fn expr_calls_mut_self_method(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(mc) => {
                // Check if this is this.someMethod() where someMethod is in mut_self_methods
                if let Expr::Identifier(obj) = mc.object.as_ref() {
                    if obj == "this" && self.mut_self_methods.contains(&mc.method) {
                        return true;
                    }
                }
                // Recurse into args
                mc.args.iter().any(|a| self.expr_calls_mut_self_method(a))
                    || self.expr_calls_mut_self_method(&mc.object)
            }
            Expr::Call(call) => {
                self.expr_calls_mut_self_method(&call.callee)
                    || call.args
                    .iter()
                    .any(|a| self.expr_calls_mut_self_method(a))
            }
            Expr::Binary { left, right, .. } => {
                self.expr_calls_mut_self_method(left)
                    || self.expr_calls_mut_self_method(right)
            }
            Expr::Unary { operand, .. } => self.expr_calls_mut_self_method(operand),
            Expr::Member { object, .. } => self.expr_calls_mut_self_method(object),
            Expr::Index { object, index, .. } => {
                self.expr_calls_mut_self_method(object)
                    || self.expr_calls_mut_self_method(index)
            }
            Expr::Ternary { condition, then_expr, else_expr } => {
                self.expr_calls_mut_self_method(condition)
                    || self.expr_calls_mut_self_method(then_expr)
                    || self.expr_calls_mut_self_method(else_expr)
            }
            Expr::Switch(sw) => {
                self.expr_calls_mut_self_method(&sw.discriminant)
                    || sw.arms.iter().any(|arm| match &arm.body {
                        SwitchBody::Block(b) => b.iter().any(|s| self.stmt_calls_mut_self_method(s)),
                        SwitchBody::Expr(e) => self.expr_calls_mut_self_method(&*e),
                    })
            }
            Expr::ArrayLiteral(elems) => elems.iter().any(|e| self.expr_calls_mut_self_method(e)),
            Expr::StringTemplate { parts } => parts.iter().any(|p| {
                if let StringTemplatePart::Expr(e) = p {
                    self.expr_calls_mut_self_method(e)
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    /// Check if a block contains assignments to self fields
    fn block_modifies_self(&self, block: &BlockStmt) -> bool {
        for stmt in &block.stmts {
            if self.stmt_modifies_self(stmt) {
                return true;
            }
        }
        false
    }

    /// Check if a statement modifies self fields
    fn stmt_modifies_self(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => self.expr_modifies_self(&expr_stmt.expr),
            Stmt::Return(return_stmt) => return_stmt
                .expr
                .as_ref()
                .map_or(false, |e| self.expr_modifies_self(e)),
            Stmt::VarDecl(var_decl) => self.expr_modifies_self(&var_decl.init),
            Stmt::Assign(assign_stmt) => {
                // B08 fix: Check if assignment target involves this/self at any depth
                // Catches: this.field = x, this.items[i].field = x, this.a.b.c = x
                if self.target_involves_self(&assign_stmt.target) {
                    return true;
                }
                self.expr_modifies_self(&assign_stmt.value)
            }
            Stmt::If(if_stmt) => {
                let cond_modifies = self.expr_modifies_self(&if_stmt.condition);
                let then_modifies = match &if_stmt.then_branch {
                    IfBody::Block(b) => self.block_modifies_self(b),
                    IfBody::Stmt(s) => self.stmt_modifies_self(s),
                };
                let else_modifies = if_stmt.else_branch.as_ref().map_or(false, |eb| match eb {
                    IfBody::Block(b) => self.block_modifies_self(b),
                    IfBody::Stmt(s) => self.stmt_modifies_self(s),
                });
                cond_modifies || then_modifies || else_modifies
            }
            Stmt::While(while_stmt) => {
                self.expr_modifies_self(&while_stmt.condition)
                    || self.block_modifies_self(&while_stmt.body)
            }
            Stmt::For(for_stmt) => {
                self.expr_modifies_self(&for_stmt.iterable)
                    || self.block_modifies_self(&for_stmt.body)
                    // B45 fix: If iterating over this.field and body mutates loop var, that's transitive self mutation
                    || self.for_mutates_self_transitively(for_stmt)
            }
            Stmt::Defer(defer_stmt) => self.stmt_modifies_self(&defer_stmt.body),
            // BUG-002 fix: Recurse into switch/case arms for self-mutation detection
            Stmt::Switch(switch_stmt) => {
                if self.expr_modifies_self(&switch_stmt.discriminant) {
                    return true;
                }
                for case in &switch_stmt.cases {
                    for s in &case.body {
                        if self.stmt_modifies_self(s) {
                            return true;
                        }
                    }
                }
                if let Some(default) = &switch_stmt.default {
                    for s in default {
                        if self.stmt_modifies_self(s) {
                            return true;
                        }
                    }
                }
                false
            }
            Stmt::TryCatch(tc) => {
                self.block_modifies_self(&tc.try_block)
                    || self.block_modifies_self(&tc.catch_block)
            }
            Stmt::Block(block) => self.block_modifies_self(block),
            _ => false,
        }
    }

    /// Check if an expression modifies self fields (assignment to this.field)
    fn expr_modifies_self(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(mc) => {
                // Bug #20 fix: Check if calling mutating methods on self fields
                // e.g., self.notes.push(note) means we modify self
                let is_mutating_method = matches!(
                    mc.method.as_str(),
                    "push"
                        | "pop"
                        | "remove"
                        | "clear"
                        | "insert"
                        | "sort"
                        | "reverse"
                        | "extend"
                        | "retain"
                        | "truncate"
                        // Bug #76 fix: Liva Map/Set method names that mutate (before codegen translation)
                        | "set"      // Map.set → HashMap.insert
                        | "add"      // Set.add → HashSet.insert
                        | "delete"   // Map/Set.delete → HashMap/HashSet.remove
                );

                if is_mutating_method {
                    // Check if the base is this.something
                    if let Some(base_name) = self.get_base_var_name(&mc.object) {
                        if base_name == "this" || base_name == "self" {
                            return true;
                        }
                    }
                    // Check if it's this.field.method()
                    if let Expr::Member { object, .. } = mc.object.as_ref() {
                        if let Expr::Identifier(obj_name) = object.as_ref() {
                            if obj_name == "this" || obj_name == "self" {
                                return true;
                            }
                        }
                    }
                }

                self.expr_modifies_self(&mc.object)
                    || mc.args.iter().any(|a| self.expr_modifies_self(a))
            }
            Expr::Call(call) => {
                self.expr_modifies_self(&call.callee)
                    || call.args.iter().any(|a| self.expr_modifies_self(a))
            }
            Expr::Binary { left, right, .. } => {
                self.expr_modifies_self(left) || self.expr_modifies_self(right)
            }
            // BUG-002 fix: Recurse into switch expression arms for self-mutation detection
            Expr::Switch(switch_expr) => {
                if self.expr_modifies_self(&switch_expr.discriminant) {
                    return true;
                }
                for arm in &switch_expr.arms {
                    match &arm.body {
                        SwitchBody::Expr(e) => {
                            if self.expr_modifies_self(e) {
                                return true;
                            }
                        }
                        SwitchBody::Block(stmts) => {
                            for s in stmts {
                                if self.stmt_modifies_self(s) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Extract the base variable name from an expression
    /// e.g., posts.parvec() -> "posts", myArray -> "myArray", this.items -> "items"
    fn get_base_var_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier(name) => Some(name.clone()),
            Expr::MethodCall(mc) => self.get_base_var_name(&mc.object),
            // Handle this.field (Member expression) — return the property name
            // so that typed_array_vars lookup works for class field arrays
            Expr::Member { object, property } => {
                if let Expr::Identifier(name) = object.as_ref() {
                    if name == "this" || name == "self" {
                        return Some(property.clone());
                    }
                }
                // For other member expressions, try the property name
                Some(property.clone())
            }
            _ => None,
        }
    }

    /// B08 fix: Recursively check if an assignment target involves `this`/`self` at any depth.
    /// Catches: this.field, this.items[i].field, this.a.b.c, etc.
    fn target_involves_self(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(name) => name == "this" || name == "self",
            Expr::Member { object, .. } => self.target_involves_self(object),
            Expr::Index { object, .. } => self.target_involves_self(object),
            _ => false,
        }
    }

    /// B45 fix: Check if a for-loop transitively modifies self
    /// Pattern: `for item in this.field { item.prop = value }`
    /// The loop variable `item` aliases elements of `this.field`, so mutating it modifies self
    fn for_mutates_self_transitively(&self, for_stmt: &crate::ast::ForStmt) -> bool {
        // Check if iterable is this.<field>
        let iterates_self_field = matches!(&for_stmt.iterable,
            Expr::Member { object, .. } if matches!(object.as_ref(), Expr::Identifier(name) if name == "this")
        );
        if !iterates_self_field {
            return false;
        }
        // Check if body assigns to loop_var.<anything>
        let loop_var = &for_stmt.var;
        self.block_assigns_to_var_field(&for_stmt.body, loop_var)
    }

    /// Check if a block contains assignments to `var_name.field = value`
    fn block_assigns_to_var_field(&self, block: &BlockStmt, var_name: &str) -> bool {
        for stmt in &block.stmts {
            if let Stmt::Assign(assign) = stmt {
                if self.target_starts_with_var(&assign.target, var_name) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if an assignment target starts with a specific variable name
    fn target_starts_with_var(&self, expr: &Expr, var_name: &str) -> bool {
        match expr {
            Expr::Member { object, .. } => {
                if let Expr::Identifier(name) = object.as_ref() {
                    name == var_name
                } else {
                    self.target_starts_with_var(object, var_name)
                }
            }
            Expr::Index { object, .. } => self.target_starts_with_var(object, var_name),
            _ => false,
        }
    }

    /// Collect all variables that are mutated (assigned after declaration) in a block
    fn collect_mutated_vars_in_block(
        &self,
        block: &BlockStmt,
        mutated: &mut std::collections::HashSet<String>,
    ) {
        for stmt in &block.stmts {
            self.collect_mutated_vars_in_stmt(stmt, mutated);
        }
    }

    /// Collect mutated variables from a statement
    fn collect_mutated_vars_in_stmt(
        &self,
        stmt: &Stmt,
        mutated: &mut std::collections::HashSet<String>,
    ) {
        match stmt {
            Stmt::Assign(assign) => {
                // This is an assignment - the target variable is mutated
                // Use sanitize_name to match snake_case used at VarDecl generation
                if let Expr::Identifier(name) = &assign.target {
                    mutated.insert(self.sanitize_name(name));
                }
                // Also check for compound assignments like arr[i] = x
                if let Expr::Index { object, .. } = &assign.target {
                    if let Expr::Identifier(name) = object.as_ref() {
                        mutated.insert(self.sanitize_name(name));
                    }
                }
            }
            // Bug #41 fix: Check VarDecl for mutating method calls in initializer
            // e.g., let x = arr.pop() should mark arr as mutated
            Stmt::VarDecl(var_decl) => {
                self.collect_mutated_vars_in_expr(&var_decl.init, mutated);
            }
            Stmt::If(if_stmt) => {
                // Recurse into branches
                match &if_stmt.then_branch {
                    IfBody::Block(b) => self.collect_mutated_vars_in_block(b, mutated),
                    IfBody::Stmt(s) => self.collect_mutated_vars_in_stmt(s, mutated),
                }
                if let Some(else_branch) = &if_stmt.else_branch {
                    match else_branch {
                        IfBody::Block(b) => self.collect_mutated_vars_in_block(b, mutated),
                        IfBody::Stmt(s) => self.collect_mutated_vars_in_stmt(s, mutated),
                    }
                }
            }
            Stmt::While(while_stmt) => {
                self.collect_mutated_vars_in_block(&while_stmt.body, mutated);
            }
            Stmt::For(for_stmt) => {
                self.collect_mutated_vars_in_block(&for_stmt.body, mutated);
            }
            Stmt::Switch(switch_stmt) => {
                for case in &switch_stmt.cases {
                    // case.body is Vec<Stmt>, not BlockStmt
                    for s in &case.body {
                        self.collect_mutated_vars_in_stmt(s, mutated);
                    }
                }
                if let Some(default) = &switch_stmt.default {
                    for s in default {
                        self.collect_mutated_vars_in_stmt(s, mutated);
                    }
                }
            }
            Stmt::Block(block) => {
                self.collect_mutated_vars_in_block(block, mutated);
            }
            Stmt::Defer(defer_stmt) => {
                self.collect_mutated_vars_in_stmt(&defer_stmt.body, mutated);
            }
            Stmt::Expr(expr_stmt) => {
                self.collect_mutated_vars_in_expr(&expr_stmt.expr, mutated);
            }
            // B100 fix: Analyze return expressions for mutated vars
            // e.g., `return lexer.tokenize()` should mark `lexer` as mutated
            Stmt::Return(ret_stmt) => {
                if let Some(expr) = &ret_stmt.expr {
                    self.collect_mutated_vars_in_expr(expr, mutated);
                }
            }
            _ => {}
        }
    }

    /// Collect mutated variables from an expression (for compound ops like i += 1)
    fn collect_mutated_vars_in_expr(
        &self,
        expr: &Expr,
        mutated: &mut std::collections::HashSet<String>,
    ) {
        match expr {
            Expr::MethodCall(mc) => {
                // Mutating methods like push, pop, etc. - for arrays AND class instances
                // Bug #43 fix: These methods mutate the object regardless of whether it's an array or class
                let is_mutating_method = matches!(
                    mc.method.as_str(),
                    "push"
                        | "pop"
                        | "remove"
                        | "clear"
                        | "insert"
                        | "sort"
                        | "reverse"
                        | "extend"
                        | "retain"
                        | "truncate"
                        | "set"
                        | "add"
                        | "delete"
                        | "update"
                        | "reset"
                        | "increment"
                        | "decrement"
                );
                if is_mutating_method {
                    if let Expr::Identifier(name) = mc.object.as_ref() {
                        // Bug #43 fix: Sanitize name to match how VarDecl lookup works
                        mutated.insert(self.sanitize_name(name));
                    }
                }

                // For class instances, any method call could potentially mutate
                // We use a heuristic: if the method name doesn't start with "get", "is", "has", "to[A-Z]"
                // it's likely a mutating method
                let is_likely_getter = mc.method.starts_with("get")
                    || mc.method.starts_with("is")
                    || mc.method.starts_with("has")
                    // B100 fix: Use "to" + uppercase check to avoid matching "tokenize" etc.
                    || (mc.method.starts_with("to") && mc.method.chars().nth(2).map_or(false, |c| c.is_uppercase()))
                    || mc.method == "length"
                    || mc.method == "size"
                    || mc.method == "count"
                    || mc.method == "clone"
                    || mc.method == "toString"
                    || mc.method == "describe"
                    || mc.method == "display"
                    // Non-mutating functional/iterator methods
                    || mc.method == "filter"
                    || mc.method == "map"
                    || mc.method == "forEach"
                    || mc.method == "find"
                    || mc.method == "some"
                    || mc.method == "every"
                    || mc.method == "reduce"
                    || mc.method == "includes"
                    || mc.method == "contains"
                    || mc.method == "join"
                    || mc.method == "slice"
                    || mc.method == "indexOf"
                    || mc.method == "lastIndexOf"
                    || mc.method == "flat"
                    || mc.method == "flatMap"
                    || mc.method == "entries"
                    || mc.method == "keys"
                    || mc.method == "values"
                    // v1.4 String methods (non-mutating)
                    || mc.method == "padStart"
                    || mc.method == "padEnd"
                    || mc.method == "repeat"
                    || mc.method == "replaceAll"
                    || mc.method == "chars"
                    || mc.method == "capitalize"
                    || mc.method == "isEmpty"
                    || mc.method == "reverse"
                    || mc.method == "truncate"
                    || mc.method == "countMatches"
                    || mc.method == "removePrefix"
                    || mc.method == "removeSuffix"
                    || mc.method == "toInt"
                    || mc.method == "toFloat"
                    // v1.4 Array methods (non-mutating)
                    || mc.method == "findIndex"
                    || mc.method == "first"
                    || mc.method == "last"
                    || mc.method == "distinct"
                    || mc.method == "zip"
                    || mc.method == "take"
                    || mc.method == "drop"
                    || mc.method == "chunks"
                    || mc.method == "sortBy"
                    || mc.method == "groupBy"
                    || mc.method == "reversed"
                    || mc.method == "sum"
                    || mc.method == "min"
                    || mc.method == "max";

                if !is_likely_getter && !is_mutating_method {
                    // This could be a mutating method on a class instance
                    if let Expr::Identifier(name) = mc.object.as_ref() {
                        // Skip class/module names (start with uppercase) — they're not variables
                        if !name.chars().next().map_or(false, |c| c.is_uppercase()) {
                            // Mark as potentially mutated (sanitized to match VarDecl lookup)
                            mutated.insert(self.sanitize_name(name));
                        }
                    }
                }

                // Recurse into args
                for arg in &mc.args {
                    self.collect_mutated_vars_in_expr(arg, mutated);
                }
            }
            Expr::Lambda(lambda) => {
                // Recurse into lambda body
                match &lambda.body {
                    LambdaBody::Block(block) => self.collect_mutated_vars_in_block(block, mutated),
                    LambdaBody::Expr(e) => self.collect_mutated_vars_in_expr(e, mutated),
                }
            }
            // SH-011 fix: Descend into switch expression arms to find mutations
            // e.g., `let _ = switch x { Variant => { arr.push(...); 0 } }` should mark arr as mutated
            Expr::Switch(switch_expr) => {
                for arm in &switch_expr.arms {
                    match &arm.body {
                        SwitchBody::Block(stmts) => {
                            for s in stmts {
                                self.collect_mutated_vars_in_stmt(s, mutated);
                            }
                        }
                        SwitchBody::Expr(e) => {
                            self.collect_mutated_vars_in_expr(e, mutated);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Generate typed JSON parsing code (Phase 1: JSON Typed Parsing)
    /// Generates: serde_json::from_str::<Type>(&json_string)
    fn generate_typed_json_parse(
        &mut self,
        method_call: &MethodCallExpr,
        type_ref: &TypeRef,
    ) -> Result<()> {
        // Convert Liva type to Rust type
        let rust_type = type_ref.to_rust_type();

        // Generate: serde_json::from_str::<RustType>(&json_arg)
        self.output.push_str("serde_json::from_str::<");
        self.output.push_str(&rust_type);
        self.output.push_str(">(&");

        // Check if this is JSON.parse() or response.json()
        if method_call.method == "json"
            && !matches!(&*method_call.object, Expr::Identifier(id) if id == "JSON")
        {
            // This is response.json() - use the response body
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".body");
        } else {
            // This is JSON.parse() - use the argument
            if let Some(arg) = method_call.args.first() {
                self.generate_expr(arg)?;
            } else {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3001",
                    "JSON.parse requires a string argument",
                    "JSON.parse must be called with a JSON string",
                )));
            }
        }

        self.output.push(')');
        Ok(())
    }

    fn indent(&mut self) {
        self.indent_level += 1;
        // B20: Push new scope level for error binding var tracking
        self.error_binding_scope_stack.push(vec![]);
    }

    fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
        // B20: Pop scope level — vars declared in this scope are no longer visible to find_error_var_in_scope
        self.error_binding_scope_stack.pop();
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn generate_program(&mut self, program: &Program) -> Result<()> {
        // Generate use statements for Rust crates
        // Rust identifiers cannot contain hyphens, so convert them to underscores
        for dep in &self.ctx.rust_crates {
            let rust_name = dep.name.replace('-', "_");
            if let Some(alias_name) = &dep.alias {
                writeln!(self.output, "use {} as {};", rust_name, alias_name).unwrap();
            } else {
                writeln!(self.output, "use {};", rust_name).unwrap();
            }
        }

        if !self.ctx.rust_crates.is_empty() {
            self.output.push('\n');
        }

        // Build class metadata maps
        // Note: class_fields/class_optional_fields are NOT fully cleared here because
        // they may contain pre-populated data from imported modules
        // (generate_entry_point/generate_module_code) — BUG-003 fix
        self.class_array_field_types.clear();
        self.class_constructor_optionals.clear();
        // Note: enum_variant_optionals is NOT cleared here because it may contain
        // pre-populated data from imported modules (generate_entry_point/generate_module_code)
        for item in &program.items {
            if let TopLevel::Class(cls) = item {
                let mut fields = std::collections::HashSet::new();
                let mut optional_fields = std::collections::HashSet::new();
                let mut array_field_types = std::collections::HashMap::new();
                let mut constructor_optionals: Vec<bool> = Vec::new();
                for m in &cls.members {
                    if let Member::Field(f) = m {
                        fields.insert(f.name.clone());
                        let is_opt = f.is_optional || matches!(&f.type_ref, Some(TypeRef::Optional(_)));
                        if is_opt {
                            optional_fields.insert(f.name.clone());
                        }
                        constructor_optionals.push(is_opt);
                        // Track array field types: fieldName -> elementType
                        if let Some(TypeRef::Array(element_type)) = &f.type_ref {
                            if let TypeRef::Simple(type_name) = element_type.as_ref() {
                                array_field_types.insert(f.name.clone(), type_name.clone());
                            }
                        }
                    }
                }
                self.class_fields.insert(cls.name.clone(), fields);
                self.class_optional_fields
                    .insert(cls.name.clone(), optional_fields);
                self.class_constructor_optionals.insert(cls.name.clone(), constructor_optionals);
                if !array_field_types.is_empty() {
                    self.class_array_field_types.insert(cls.name.clone(), array_field_types);
                }

                // B100 fix: Scan class methods for return types (string, [T])
                for m in &cls.members {
                    if let Member::Method(method) = m {
                        if let Some(ret_type) = &method.return_type {
                            if matches!(ret_type, TypeRef::Simple(name) if name == "string") {
                                self.string_returning_methods.insert(method.name.clone());
                            }
                            if let TypeRef::Array(elem) = ret_type {
                                let elem_type = match elem.as_ref() {
                                    TypeRef::Simple(name) => name.clone(),
                                    _ => String::new(),
                                };
                                self.array_returning_methods.insert(method.name.clone(), elem_type);
                            }
                        }
                    }
                }
            }
        }

        // Build interface method signatures map (for type inference in implementing classes)
        // Note: Due to parser design, interfaces may be parsed as Class without constructor
        self.interface_methods.clear();
        for item in &program.items {
            // Check TopLevel::Type (explicit interface syntax via 'type' keyword)
            if let TopLevel::Type(type_decl) = item {
                let mut methods: std::collections::HashMap<String, TypeRef> =
                    std::collections::HashMap::new();
                for member in &type_decl.members {
                    if let Member::Method(m) = member {
                        if let Some(ret_type) = &m.return_type {
                            methods.insert(m.name.clone(), ret_type.clone());
                        }
                    }
                }
                if !methods.is_empty() {
                    self.interface_methods
                        .insert(type_decl.name.clone(), methods);
                }
            }
            // Also check Class that is really an interface (no constructor, only method signatures)
            if let TopLevel::Class(class) = item {
                let has_constructor = class
                    .members
                    .iter()
                    .any(|m| matches!(m, Member::Method(method) if method.name == "constructor"));
                let has_method_bodies = class.members.iter().any(|m| {
                    matches!(m, Member::Method(method) if method.body.is_some() || method.expr_body.is_some())
                });
                // If no constructor and no method bodies, it's an interface
                if !has_constructor && !has_method_bodies {
                    let mut methods: std::collections::HashMap<String, TypeRef> =
                        std::collections::HashMap::new();
                    for member in &class.members {
                        if let Member::Method(m) = member {
                            if let Some(ret_type) = &m.return_type {
                                methods.insert(m.name.clone(), ret_type.clone());
                            }
                        }
                    }
                    if !methods.is_empty() {
                        self.interface_methods.insert(class.name.clone(), methods);
                    }
                }
            }
        }

        // Build enum metadata maps (merge with pre-loaded imported enums)
        for item in &program.items {
            if let TopLevel::Enum(enum_decl) = item {
                self.enum_names.insert(enum_decl.name.clone());
                let mut variants_map = std::collections::HashMap::new();
                let mut variants_type_map = std::collections::HashMap::new();
                for variant in &enum_decl.variants {
                    let field_names: Vec<String> =
                        variant.fields.iter().map(|f| f.name.clone()).collect();
                    let field_types: Vec<(String, TypeRef)> =
                        variant.fields.iter().map(|f| (f.name.clone(), f.type_ref.clone())).collect();
                    variants_map.insert(variant.name.clone(), field_names);
                    variants_type_map.insert(variant.name.clone(), field_types);

                    // Track optional fields per variant for Some() wrapping
                    let variant_optionals: Vec<bool> = variant.fields.iter()
                        .map(|f| matches!(&f.type_ref, TypeRef::Optional(_)))
                        .collect();
                    if variant_optionals.iter().any(|&o| o) {
                        let key = format!("{}::{}", enum_decl.name, variant.name);
                        self.enum_variant_optionals.insert(key, variant_optionals);
                    }
                }
                self.enum_variants
                    .insert(enum_decl.name.clone(), variants_map);
                self.enum_variant_field_types
                    .insert(enum_decl.name.clone(), variants_type_map);

                // Pre-detect recursive fields for auto-boxing
                let mut boxed_fields_for_enum: std::collections::HashMap<
                    String,
                    std::collections::HashSet<String>,
                > = std::collections::HashMap::new();
                for variant in &enum_decl.variants {
                    for field in &variant.fields {
                        if Self::is_recursive_field(&field.type_ref, &enum_decl.name) {
                            boxed_fields_for_enum
                                .entry(variant.name.clone())
                                .or_default()
                                .insert(field.name.clone());
                        }
                    }
                }
                if !boxed_fields_for_enum.is_empty() {
                    self.boxed_enum_fields
                        .insert(enum_decl.name.clone(), boxed_fields_for_enum);
                }
            }
        }

        // B46: Pre-scan for JSON.stringify usage to mark classes needing serde
        self.scan_json_stringify_classes(program);

        // Always include concurrency runtime for now
        if std::env::var("LIVA_DEBUG").is_ok() {
            println!("DEBUG: Including liva_rt module");
        }
        self.writeln("#[allow(dead_code)]");
        self.writeln("mod liva_rt {");
        self.indent();
        self.writeln("use std::future::Future;");
        self.writeln("use tokio::task::JoinHandle;");
        self.writeln("");

        // Add Error type for fallibility system with error trace support
        self.writeln("/// Runtime error type for fallible operations with trace chaining");
        self.writeln("#[derive(Debug, Clone)]");
        self.writeln("pub struct Error {");
        self.indent();
        self.writeln("pub message: String,");
        self.writeln("pub function: &'static str,");
        self.writeln("pub location: &'static str,");
        self.writeln("pub cause: Option<Box<Error>>,");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl PartialEq for Error {");
        self.indent();
        self.writeln("fn eq(&self, other: &Self) -> bool {");
        self.indent();
        self.writeln("self.message == other.message");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl Error {");
        self.indent();
        self.writeln("pub fn from<S: Into<String>>(message: S) -> Self {");
        self.indent();
        self.writeln("Error { message: message.into(), function: \"\", location: \"\", cause: None }");
        self.dedent();
        self.writeln("}");
        self.writeln("pub fn new<S: Into<String>>(message: S, function: &'static str, location: &'static str) -> Self {");
        self.indent();
        self.writeln("Error { message: message.into(), function, location, cause: None }");
        self.dedent();
        self.writeln("}");
        self.writeln("pub fn chain<S: Into<String>>(message: S, function: &'static str, location: &'static str, cause: Error) -> Self {");
        self.indent();
        self.writeln("Error { message: message.into(), function, location, cause: Some(Box::new(cause)) }");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl std::fmt::Display for Error {");
        self.indent();
        self.writeln("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
        self.indent();
        // Collect all errors in chain
        self.writeln("let mut errors: Vec<&Error> = Vec::new();");
        self.writeln("let mut current: Option<&Error> = Some(self);");
        self.writeln("while let Some(err) = current {");
        self.indent();
        self.writeln("errors.push(err);");
        self.writeln("current = err.cause.as_deref();");
        self.dedent();
        self.writeln("}");
        // Check if we have a chain (more than 1 error) — if so, show trace
        self.writeln("if errors.len() == 1 && self.function.is_empty() {");
        self.indent();
        self.writeln("return write!(f, \"{}\", self.message);");
        self.dedent();
        self.writeln("}");
        // Full trace display with colors
        self.writeln("writeln!(f, \"\\x1b[90m╭─ Error Trace ─────────────────────────────────────╮\\x1b[0m\")?;");
        self.writeln("for (i, err) in errors.iter().enumerate() {");
        self.indent();
        self.writeln("let (icon, color) = if i == 0 { (\"✗\", \"\\x1b[1;31m\") } else { (\"⊘\", \"\\x1b[33m\") };");
        self.writeln("writeln!(f, \"\\x1b[90m│\\x1b[0m  {}{} {}\\x1b[0m\", color, icon, err.message)?;");
        self.writeln("if !err.function.is_empty() || !err.location.is_empty() {");
        self.indent();
        self.writeln("writeln!(f, \"\\x1b[90m│    → {}()  {}\\x1b[0m\", err.function, err.location)?;");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("write!(f, \"\\x1b[90m╰───────────────────────────────────────────────────╯\\x1b[0m\")");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl std::error::Error for Error {}");
        self.writeln("");

        // Always include both async and parallel functions
        self.writeln("/// Spawn an async task");
        self.writeln("pub fn spawn_async<F, T>(future: F) -> JoinHandle<T>");
        self.writeln("where");
        self.indent();
        self.writeln("F: Future<Output = T> + Send + 'static,");
        self.writeln("T: Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::spawn(future)");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Fire and forget async task");
        self.writeln("pub fn fire_async<F>(future: F)");
        self.writeln("where");
        self.indent();
        self.writeln("F: Future<Output = ()> + Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::spawn(future);");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Spawn a parallel task on a dedicated blocking thread");
        self.writeln("pub fn spawn_parallel<F, T>(f: F) -> JoinHandle<T>");
        self.writeln("where");
        self.indent();
        self.writeln("F: FnOnce() -> T + Send + 'static,");
        self.writeln("T: Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::task::spawn_blocking(f)");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Fire and forget parallel task");
        self.writeln("pub fn fire_parallel<F>(f: F)");
        self.writeln("where");
        self.indent();
        self.writeln("F: FnOnce() + Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("// For simplicity, just spawn a thread");
        self.writeln("std::thread::spawn(f);");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // String multiplication helper
        self.writeln("/// String multiplication helper");
        self.writeln("/// Supports both String*int and int*String patterns");
        self.writeln(
            "pub fn string_mul<L: StringOrInt, R: StringOrInt>(left: L, right: R) -> String {",
        );
        self.indent();
        self.writeln("match (left.as_string_or_int(), right.as_string_or_int()) {");
        self.indent();
        self.writeln("(StringOrIntValue::String(s), StringOrIntValue::Int(n)) => {");
        self.indent();
        self.writeln("if n <= 0 { String::new() } else { s.repeat(n as usize) }");
        self.dedent();
        self.writeln("}");
        self.writeln("(StringOrIntValue::Int(n), StringOrIntValue::String(s)) => {");
        self.indent();
        self.writeln("if n <= 0 { String::new() } else { s.repeat(n as usize) }");
        self.dedent();
        self.writeln("}");
        self.writeln("(StringOrIntValue::Int(a), StringOrIntValue::Int(b)) => {");
        self.indent();
        self.writeln("(a * b).to_string()");
        self.dedent();
        self.writeln("}");
        self.writeln("(StringOrIntValue::String(_), StringOrIntValue::String(_)) => {");
        self.indent();
        self.writeln("panic!(\"Cannot multiply two strings\")");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub enum StringOrIntValue {");
        self.indent();
        self.writeln("String(String),");
        self.writeln("Int(i64),");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub trait StringOrInt {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue;");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl StringOrInt for String {");
        self.indent();
        self.writeln(
            "fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::String(self) }",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for &str {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::String(self.to_string()) }");
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for i32 {");
        self.indent();
        self.writeln(
            "fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self as i64) }",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for i64 {");
        self.indent();
        self.writeln(
            "fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self) }",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for f64 {");
        self.indent();
        self.writeln(
            "fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self as i64) }",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for usize {");
        self.indent();
        self.writeln(
            "fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self as i64) }",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // HTTP Client Runtime Functions
        self.writeln("// HTTP Client");
        self.writeln("#[derive(Debug, Clone, Default)]");
        self.writeln("pub struct LivaHttpResponse {");
        self.indent();
        self.writeln("pub status: i32,");
        self.writeln("pub status_text: String,");
        self.writeln("pub body: String,");
        self.writeln("pub headers: Vec<String>,");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add impl block with json() method
        self.writeln("impl LivaHttpResponse {");
        self.indent();
        self.writeln("pub fn json(&self) -> (JsonValue, String) {");
        self.indent();
        self.writeln("match serde_json::from_str(&self.body) {");
        self.indent();
        self.writeln("Ok(value) => (JsonValue(value), String::new()),");
        self.writeln(
            "Err(e) => (JsonValue(serde_json::Value::Null), format!(\"JSON parse error: {}\", e)),",
        );
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.writeln(
            "pub async fn liva_http_get(url: String) -> (Option<LivaHttpResponse>, String) {",
        );
        self.indent();
        self.writeln("liva_http_request(\"GET\", url, None).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.writeln("pub async fn liva_http_post(url: String, body: String) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("liva_http_request(\"POST\", url, Some(body)).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.writeln("pub async fn liva_http_put(url: String, body: String) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("liva_http_request(\"PUT\", url, Some(body)).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.writeln(
            "pub async fn liva_http_delete(url: String) -> (Option<LivaHttpResponse>, String) {",
        );
        self.indent();
        self.writeln("liva_http_request(\"DELETE\", url, None).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.writeln("async fn liva_http_request(method: &str, url: String, body: Option<String>) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("if !url.starts_with(\"http://\") && !url.starts_with(\"https://\") {");
        self.indent();
        self.writeln("return (None, format!(\"Invalid URL format: '{}'. URLs must start with http:// or https://\", url));");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build() {");
        self.indent();
        self.writeln("Ok(c) => c,");
        self.writeln("Err(e) => return (None, format!(\"Failed to create HTTP client: {}\", e)),");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("let request_builder = match method {");
        self.indent();
        self.writeln("\"GET\" => client.get(&url),");
        self.writeln("\"POST\" => {");
        self.indent();
        self.writeln("let mut builder = client.post(&url);");
        self.writeln("if let Some(body_content) = body {");
        self.indent();
        self.writeln(
            "builder = builder.header(\"Content-Type\", \"application/json\").body(body_content);",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("builder");
        self.dedent();
        self.writeln("}");
        self.writeln("\"PUT\" => {");
        self.indent();
        self.writeln("let mut builder = client.put(&url);");
        self.writeln("if let Some(body_content) = body {");
        self.indent();
        self.writeln(
            "builder = builder.header(\"Content-Type\", \"application/json\").body(body_content);",
        );
        self.dedent();
        self.writeln("}");
        self.writeln("builder");
        self.dedent();
        self.writeln("}");
        self.writeln("\"DELETE\" => client.delete(&url),");
        self.writeln("_ => return (None, format!(\"Unknown HTTP method: {}\", method)),");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("// Add User-Agent header required by GitHub and other APIs");
        self.writeln("let request_builder = request_builder.header(\"User-Agent\", \"Liva-HTTP-Client/1.0\");");
        self.writeln("");
        self.writeln("let response = match request_builder.send().await {");
        self.indent();
        self.writeln("Ok(resp) => resp,");
        self.writeln("Err(e) => {");
        self.indent();
        self.writeln("let error_msg = if e.is_timeout() { \"Request timeout (30s)\".to_string() }");
        self.writeln("else if e.is_connect() { format!(\"Connection error: {}\", e) }");
        self.writeln("else { format!(\"Network error: {}\", e) };");
        self.writeln("return (None, error_msg);");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("let status = response.status();");
        self.writeln("let status_code = status.as_u16() as i32;");
        self.writeln(
            "let status_text = status.canonical_reason().unwrap_or(\"Unknown\").to_string();",
        );
        self.writeln("");
        self.writeln("let mut headers = Vec::new();");
        self.writeln("for (key, value) in response.headers() {");
        self.indent();
        self.writeln("if let Ok(value_str) = value.to_str() {");
        self.indent();
        self.writeln("headers.push(format!(\"{}: {}\", key.as_str(), value_str));");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("let body = match response.text().await {");
        self.indent();
        self.writeln("Ok(text) => text,");
        self.writeln("Err(e) => return (None, format!(\"Failed to read response body: {}\", e)),");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("(Some(LivaHttpResponse { status: status_code, status_text, body, headers }), String::new())");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // JSON Support - JsonValue wrapper
        self.writeln("// JSON Support");
        self.writeln("#[derive(Debug, Clone)]");
        self.writeln("pub struct JsonValue(pub serde_json::Value);");
        self.writeln("");
        self.writeln("impl Default for JsonValue {");
        self.indent();
        self.writeln("fn default() -> Self { JsonValue(serde_json::Value::Null) }");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl JsonValue {");
        self.indent();
        self.writeln("pub fn new(value: serde_json::Value) -> Self { JsonValue(value) }");
        self.writeln("");
        self.writeln("pub fn length(&self) -> usize {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Array(arr) => arr.len(),");
        self.writeln("serde_json::Value::Object(obj) => obj.len(),");
        self.writeln("serde_json::Value::String(s) => s.len(),");
        self.writeln("_ => 0,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn get(&self, index: usize) -> Option<JsonValue> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln(
            "serde_json::Value::Array(arr) => arr.get(index).map(|v| JsonValue(v.clone())),",
        );
        self.writeln("_ => None,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn get_field(&self, key: &str) -> Option<JsonValue> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln(
            "serde_json::Value::Object(obj) => obj.get(key).map(|v| JsonValue(v.clone())),",
        );
        self.writeln("_ => None,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Type conversion methods
        self.writeln("pub fn as_i32(&self) -> Option<i32> {");
        self.indent();
        self.writeln("self.0.as_i64().map(|n| n as i32)");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn as_f64(&self) -> Option<f64> {");
        self.indent();
        self.writeln("self.0.as_f64()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        // as_float is an alias for as_f64
        self.writeln("pub fn as_float(&self) -> f64 {");
        self.indent();
        self.writeln("self.0.as_f64().unwrap_or(0.0)");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn as_string(&self) -> Option<String> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::String(s) => Some(s.clone()),");
        self.writeln("_ => None,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn as_bool(&self) -> Option<bool> {");
        self.indent();
        self.writeln("self.0.as_bool()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn is_null(&self) -> bool {");
        self.indent();
        self.writeln("self.0.is_null()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn is_array(&self) -> bool {");
        self.indent();
        self.writeln("self.0.is_array()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn is_object(&self) -> bool {");
        self.indent();
        self.writeln("self.0.is_object()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn to_json_string(&self) -> String {");
        self.indent();
        self.writeln("self.0.to_string()");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // as_array() returns Vec<JsonValue> directly (unwraps automatically)
        self.writeln("pub fn as_array(&self) -> Vec<JsonValue> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln(
            "serde_json::Value::Array(arr) => arr.iter().map(|v| JsonValue(v.clone())).collect(),",
        );
        self.writeln("_ => Vec::new(),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add to_vec and iter methods for array operations
        self.writeln("pub fn to_vec(&self) -> Vec<JsonValue> {");
        self.indent();
        self.writeln("self.as_array()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn iter(&self) -> std::vec::IntoIter<JsonValue> {");
        self.indent();
        self.writeln("self.to_vec().into_iter()");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        self.writeln("impl std::fmt::Display for JsonValue {");
        self.indent();
        self.writeln("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
        self.indent();
        // For strings, display without quotes; for other types, use JSON representation
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::String(s) => write!(f, \"{}\", s),");
        self.writeln("serde_json::Value::Null => write!(f, \"null\"),");
        self.writeln("other => write!(f, \"{}\", other),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add Index<&str> for nested JSON access: json["field"]["nested"]
        self.writeln("impl std::ops::Index<&str> for JsonValue {");
        self.indent();
        self.writeln("type Output = JsonValue;");
        self.writeln("");
        self.writeln("fn index(&self, key: &str) -> &Self::Output {");
        self.indent();
        self.writeln("// This is a bit of a hack - we leak the value to get a static reference");
        self.writeln("// In practice, this is safe for our use case since we don't mutate");
        self.writeln(
            "static NULL_VALUE: std::sync::OnceLock<JsonValue> = std::sync::OnceLock::new();",
        );
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Object(obj) => {");
        self.indent();
        self.writeln("if let Some(v) = obj.get(key) {");
        self.indent();
        self.writeln("// Leak to get 'static lifetime - acceptable for read-only JSON access");
        self.writeln("Box::leak(Box::new(JsonValue(v.clone())))");
        self.dedent();
        self.writeln("} else {");
        self.indent();
        self.writeln("NULL_VALUE.get_or_init(|| JsonValue(serde_json::Value::Null))");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("_ => NULL_VALUE.get_or_init(|| JsonValue(serde_json::Value::Null)),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add IntoIterator for for...in loop support
        self.writeln("impl IntoIterator for JsonValue {");
        self.indent();
        self.writeln("type Item = JsonValue;");
        self.writeln("type IntoIter = std::vec::IntoIter<JsonValue>;");
        self.writeln("");
        self.writeln("fn into_iter(self) -> Self::IntoIter {");
        self.indent();
        self.writeln("match self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Array(arr) => {");
        self.indent();
        self.writeln("arr.into_iter().map(|v| JsonValue(v)).collect::<Vec<_>>().into_iter()");
        self.dedent();
        self.writeln("}");
        self.writeln("_ => Vec::new().into_iter(),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add PartialEq<bool> for comparing JSON booleans
        self.writeln("impl PartialEq<bool> for JsonValue {");
        self.indent();
        self.writeln("fn eq(&self, other: &bool) -> bool {");
        self.indent();
        self.writeln("self.as_bool() == Some(*other)");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add PartialEq<&str> for comparing JSON strings
        self.writeln("impl PartialEq<&str> for JsonValue {");
        self.indent();
        self.writeln("fn eq(&self, other: &&str) -> bool {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::String(s) => s == *other,");
        self.writeln("_ => false,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add is_null check method and comparison - null generates .is_null() check
        // No PartialEq needed - we'll translate `x != null` to `!x.is_null()`

        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add rayon imports if parallel execution is used (at top level, after liva_rt module)
        if self.ctx.has_parallel {
            self.writeln("// Rayon parallel iterator support");
            self.writeln("use rayon::prelude::*;");
            self.writeln("");
        }

        // Add logging runtime helpers if Log.* is used
        if self.ctx.has_logging {
            self.writeln("// Logging runtime helpers");
            self.writeln("use std::sync::atomic::{AtomicU8, Ordering};");
            self.writeln("");
            self.writeln("static LIVA_LOG_LEVEL: AtomicU8 = AtomicU8::new(1); // 0=debug, 1=info, 2=warn, 3=error");
            self.writeln("");
            // liva_log: simplified — no context param, variadic message built at call site
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_log(level: u8, label: &str, msg: &str) {");
            self.indent();
            self.writeln("if level < LIVA_LOG_LEVEL.load(Ordering::Relaxed) { return; }");
            self.writeln("if level == 0 && std::env::var(\"LIVA_VERBOSE\").is_err() { return; }");
            self.writeln("let now = chrono::Local::now().format(\"%Y-%m-%dT%H:%M:%S\");");
            self.writeln("eprintln!(\"{} [{:<5}] {}\", now, label, msg);");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            // liva_log_table_kv: Key/Value table for maps with 4+ entries
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_log_table_kv(level: u8, keys: &[&str], values: &[String]) {");
            self.indent();
            self.writeln("if level < LIVA_LOG_LEVEL.load(Ordering::Relaxed) { return; }");
            self.writeln("if level == 0 && std::env::var(\"LIVA_VERBOSE\").is_err() { return; }");
            self.writeln("let kw = keys.iter().map(|k| k.len()).max().unwrap_or(3).max(3);");
            self.writeln("let vw = values.iter().map(|v| v.len()).max().unwrap_or(5).max(5);");
            self.writeln("let ks = \"\\u{2500}\".repeat(kw + 2);");
            self.writeln("let vs = \"\\u{2500}\".repeat(vw + 2);");
            self.writeln("eprintln!(\"   \\u{250c}{}\\u{252c}{}\\u{2510}\", ks, vs);");
            self.writeln("eprintln!(\"   \\u{2502} {:<kw$} \\u{2502} {:<vw$} \\u{2502}\", \"Key\", \"Value\", kw = kw, vw = vw);");
            self.writeln("eprintln!(\"   \\u{251c}{}\\u{253c}{}\\u{2524}\", ks, vs);");
            self.writeln("for (k, v) in keys.iter().zip(values.iter()) {");
            self.indent();
            self.writeln("eprintln!(\"   \\u{2502} {:<kw$} \\u{2502} {:<vw$} \\u{2502}\", k, v, kw = kw, vw = vw);");
            self.dedent();
            self.writeln("}");
            self.writeln("eprintln!(\"   \\u{2514}{}\\u{2534}{}\\u{2518}\", ks, vs);");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            // liva_log_table_rows: columnar table for arrays of maps
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_log_table_rows(level: u8, headers: &[&str], rows: &[Vec<String>]) {");
            self.indent();
            self.writeln("if level < LIVA_LOG_LEVEL.load(Ordering::Relaxed) { return; }");
            self.writeln("if level == 0 && std::env::var(\"LIVA_VERBOSE\").is_err() { return; }");
            self.writeln("let widths: Vec<usize> = headers.iter().enumerate().map(|(i, h)| {");
            self.indent();
            self.writeln("rows.iter().map(|r| r.get(i).map_or(0, |v| v.len())).max().unwrap_or(0).max(h.len())");
            self.dedent();
            self.writeln("}).collect();");
            self.writeln("let border = |left: &str, mid: &str, right: &str| {");
            self.indent();
            self.writeln("let parts: Vec<String> = widths.iter().map(|w| \"\\u{2500}\".repeat(w + 2)).collect();");
            self.writeln("eprintln!(\"   {}{}{}\", left, parts.join(mid), right);");
            self.dedent();
            self.writeln("};");
            self.writeln("border(\"\\u{250c}\", \"\\u{252c}\", \"\\u{2510}\");");
            self.writeln("let hdr: Vec<String> = headers.iter().zip(widths.iter()).map(|(h, w)| format!(\" {:<width$} \", h, width = *w)).collect();");
            self.writeln("eprintln!(\"   \\u{2502}{}\\u{2502}\", hdr.join(\"\\u{2502}\"));");
            self.writeln("border(\"\\u{251c}\", \"\\u{253c}\", \"\\u{2524}\");");
            self.writeln("for row in rows {");
            self.indent();
            self.writeln("let cells: Vec<String> = row.iter().zip(widths.iter()).map(|(v, w)| format!(\" {:<width$} \", v, width = *w)).collect();");
            self.writeln("eprintln!(\"   \\u{2502}{}\\u{2502}\", cells.join(\"\\u{2502}\"));");
            self.dedent();
            self.writeln("}");
            self.writeln("border(\"\\u{2514}\", \"\\u{2534}\", \"\\u{2518}\");");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            // liva_log_json: runtime JSON table rendering (Object → KV table, Array of Objects → columnar table)
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_log_json(level: u8, json: &liva_rt::JsonValue) {");
            self.indent();
            self.writeln("if level < LIVA_LOG_LEVEL.load(Ordering::Relaxed) { return; }");
            self.writeln("if level == 0 && std::env::var(\"LIVA_VERBOSE\").is_err() { return; }");
            self.writeln("match &json.0 {");
            self.indent();
            // Object with 4+ keys → Key/Value table
            self.writeln("serde_json::Value::Object(obj) if obj.len() >= 4 => {");
            self.indent();
            self.writeln("let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();");
            self.writeln("let values: Vec<String> = obj.values().map(|v| match v { serde_json::Value::String(s) => s.clone(), other => other.to_string() }).collect();");
            self.writeln("liva_log_table_kv(level, &keys, &values);");
            self.dedent();
            self.writeln("}");
            // Object with <4 keys → inline
            self.writeln("serde_json::Value::Object(obj) => {");
            self.indent();
            self.writeln("let parts: Vec<String> = obj.iter().map(|(k, v)| match v { serde_json::Value::String(s) => format!(\"{}: {}\", k, s), other => format!(\"{}: {}\", k, other) }).collect();");
            self.writeln("eprintln!(\"   {{{}}}\", parts.join(\", \"));");
            self.dedent();
            self.writeln("}");
            // Array of Objects → columnar table
            self.writeln("serde_json::Value::Array(arr) if !arr.is_empty() && arr.iter().all(|v| v.is_object()) => {");
            self.indent();
            self.writeln("if let Some(serde_json::Value::Object(first)) = arr.first() {");
            self.indent();
            self.writeln("let headers: Vec<&str> = first.keys().map(|k| k.as_str()).collect();");
            self.writeln("let rows: Vec<Vec<String>> = arr.iter().filter_map(|v| v.as_object()).map(|obj| {");
            self.indent();
            self.writeln("headers.iter().map(|h| match obj.get(*h) { Some(serde_json::Value::String(s)) => s.clone(), Some(other) => other.to_string(), None => String::new() }).collect()");
            self.dedent();
            self.writeln("}).collect();");
            self.writeln("liva_log_table_rows(level, &headers, &rows);");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            // Anything else → plain eprintln
            self.writeln("other => eprintln!(\"   {}\", other),");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            // liva_log_set_level: unchanged
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_log_set_level(level: &str) {");
            self.indent();
            self.writeln("let n = match level {");
            self.indent();
            self.writeln("\"debug\" => 0,");
            self.writeln("\"info\" => 1,");
            self.writeln("\"warn\" => 2,");
            self.writeln("\"error\" => 3,");
            self.writeln("_ => 1,");
            self.dedent();
            self.writeln("};");
            self.writeln("LIVA_LOG_LEVEL.store(n, Ordering::Relaxed);");
            self.dedent();
            self.writeln("}");
            self.writeln("");
        }

        // Add Config runtime helpers if Config.* is used
        if self.ctx.has_config {
            self.writeln("// Config runtime helpers (.env file parser)");
            self.writeln("use std::collections::HashMap;");
            self.writeln("");
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_config_load(path: &str) -> (Option<HashMap<String, String>>, String) {");
            self.indent();
            self.writeln("match std::fs::read_to_string(path) {");
            self.indent();
            self.writeln("Ok(content) => {");
            self.indent();
            self.writeln("let mut map = HashMap::new();");
            self.writeln("for line in content.lines() {");
            self.indent();
            self.writeln("let line = line.trim();");
            self.writeln("if line.is_empty() || line.starts_with('#') { continue; }");
            self.writeln("if let Some(eq_pos) = line.find('=') {");
            self.indent();
            self.writeln("let key = line[..eq_pos].trim().to_string();");
            self.writeln("let mut value = line[eq_pos + 1..].trim().to_string();");
            self.writeln("// Strip surrounding quotes");
            self.writeln("if (value.starts_with('\"') && value.ends_with('\"')) || (value.starts_with('\\'') && value.ends_with('\\'')) {");
            self.indent();
            self.writeln("value = value[1..value.len()-1].to_string();");
            self.dedent();
            self.writeln("}");
            self.writeln("if !key.is_empty() { map.insert(key, value); }");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("(Some(map), String::new())");
            self.dedent();
            self.writeln("}");
            self.writeln("Err(e) => (None, format!(\"Config load error: {}\", e)),");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_config_get(map: &HashMap<String, String>, key: &str) -> (Option<String>, String) {");
            self.indent();
            self.writeln("match map.get(key) {");
            self.indent();
            self.writeln("Some(v) => (Some(v.clone()), String::new()),");
            self.writeln("None => (None, format!(\"Config key not found: {}\", key)),");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_config_get_int(map: &HashMap<String, String>, key: &str) -> (Option<i32>, String) {");
            self.indent();
            self.writeln("match map.get(key) {");
            self.indent();
            self.writeln("Some(v) => match v.parse::<i32>() {");
            self.indent();
            self.writeln("Ok(n) => (Some(n), String::new()),");
            self.writeln("Err(e) => (None, format!(\"Config key '{}' is not a valid int: {}\", key, e)),");
            self.dedent();
            self.writeln("},");
            self.writeln("None => (None, format!(\"Config key not found: {}\", key)),");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_config_get_bool(map: &HashMap<String, String>, key: &str) -> (Option<bool>, String) {");
            self.indent();
            self.writeln("match map.get(key) {");
            self.indent();
            self.writeln("Some(v) => match v.to_lowercase().as_str() {");
            self.indent();
            self.writeln("\"true\" | \"1\" | \"yes\" | \"on\" => (Some(true), String::new()),");
            self.writeln("\"false\" | \"0\" | \"no\" | \"off\" => (Some(false), String::new()),");
            self.writeln("_ => (None, format!(\"Config key '{}' is not a valid bool: {}\", key, v)),");
            self.dedent();
            self.writeln("},");
            self.writeln("None => (None, format!(\"Config key not found: {}\", key)),");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("");
            self.writeln("#[allow(dead_code)]");
            self.writeln("fn liva_config_get_all(map: &HashMap<String, String>) -> std::collections::BTreeMap<String, String> {");
            self.indent();
            self.writeln("map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()");
            self.dedent();
            self.writeln("}");
            self.writeln("");
        }

        // Pre-pass: Collect all type aliases
        for item in &program.items {
            if let TopLevel::TypeAlias(alias) = item {
                self.type_aliases.insert(
                    alias.name.clone(),
                    (alias.type_params.clone(), alias.target_type.clone()),
                );
            }
        }

        // Pre-pass: Collect all union types by scanning type annotations
        // This happens during generation, unions are registered in expand_type_alias

        // Generate top-level items (first pass to collect unions)
        for item in &program.items {
            if std::env::var("LIVA_DEBUG").is_ok() {
                println!("DEBUG: Processing top-level item: {:?}", item);
            }
            match item {
                TopLevel::Class(cls) => {
                    if std::env::var("LIVA_DEBUG").is_ok() {
                        println!("DEBUG: Found class: {}", cls.name)
                    }
                }
                TopLevel::Function(func) => {
                    if std::env::var("LIVA_DEBUG").is_ok() {
                        println!("DEBUG: Found function: {}", func.name)
                    }
                }
                _ => {
                    if std::env::var("LIVA_DEBUG").is_ok() {
                        println!("DEBUG: Found other item: {:?}", item)
                    }
                }
            }
            self.generate_top_level(item)?;
            self.output.push('\n');
        }

        // After first pass, generate union type enum definitions
        let unions_to_generate: Vec<Vec<String>> = self.union_types.iter().cloned().collect();
        if !unions_to_generate.is_empty() {
            // Insert union enums before the generated code
            let mut union_defs = String::new();
            union_defs.push_str("\n// Union type definitions\n");

            for union_types in unions_to_generate {
                let enum_name = format!("Union_{}", union_types.join("_"));
                union_defs.push_str(&format!("#[derive(Debug, Clone)]\n"));
                union_defs.push_str(&format!("enum {} {{\n", enum_name));

                // Generate variant for each type in the union
                for (_i, rust_type) in union_types.iter().enumerate() {
                    let variant_name = self.type_to_variant_name(rust_type);
                    union_defs.push_str(&format!("    {}({}),\n", variant_name, rust_type));
                }

                union_defs.push_str("}\n\n");

                // Implement Display for the union enum
                union_defs.push_str(&format!("impl std::fmt::Display for {} {{\n", enum_name));
                union_defs.push_str(
                    "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n",
                );
                union_defs.push_str("        match self {\n");

                for rust_type in &union_types {
                    let variant_name = self.type_to_variant_name(rust_type);
                    union_defs.push_str(&format!(
                        "            {}::{}(val) => write!(f, \"{{}}\", val),\n",
                        enum_name, variant_name
                    ));
                }

                union_defs.push_str("        }\n");
                union_defs.push_str("    }\n");
                union_defs.push_str("}\n\n");
            }

            // Find where to insert (after liva_rt module, before first function)
            // For now, prepend to output (will fix positioning later)
            let temp = self.output.clone();
            self.output = union_defs;
            self.output.push_str(&temp);
        }

        Ok(())
    }

    /// Convert a Rust type string to a valid enum variant name
    fn type_to_variant_name(&self, rust_type: &str) -> String {
        match rust_type {
            "i32" => "Int".to_string(),
            "String" => "Str".to_string(),
            "f64" => "Float".to_string(),
            "bool" => "Bool".to_string(),
            other => {
                // Remove special characters and capitalize
                other
                    .replace("<", "")
                    .replace(">", "")
                    .replace(",", "")
                    .replace(" ", "")
                    .replace("(", "")
                    .replace(")", "")
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_default()
                    + &other[1..]
                        .replace("<", "")
                        .replace(">", "")
                        .replace(",", "")
                        .replace(" ", "")
                        .replace("(", "")
                        .replace(")", "")
            }
        }
    }

    /// Generate union wrapper if needed (e.g., Union_i32_String::Int(42))
    /// Returns (needs_close, needs_to_string) tuple
    fn maybe_wrap_in_union(&mut self, dest_type_ref: &TypeRef, expr: &Expr) -> (bool, bool) {
        // Check if destination is a union type
        if let TypeRef::Union(members) = dest_type_ref {
            // Infer the type of the expression (using existing infer_expr_type)
            if let Some(type_with_arrow) = self.infer_expr_type(expr, None) {
                // Extract type from " -> i32" format
                let expr_type = type_with_arrow.trim_start_matches(" -> ").to_string();

                // Generate the union type name
                let expanded_members: Vec<String> =
                    members.iter().map(|m| self.expand_type_alias(m)).collect();
                let union_name = format!("Union_{}", expanded_members.join("_"));

                // Find which variant to use
                let variant = self.type_to_variant_name(&expr_type);

                // Check if this is a string literal that needs .to_string()
                let needs_to_string =
                    expr_type == "String" && matches!(expr, Expr::Literal(Literal::String(_)));

                // Generate wrapper
                write!(self.output, "{}::{}(", union_name, variant).unwrap();
                return (true, needs_to_string);
            }
        }
        (false, false)
    }

    fn generate_top_level(&mut self, item: &TopLevel) -> Result<()> {
        match item {
            TopLevel::Import(_) => {
                // Imports are handled differently in Rust
                // We'd need to map to actual module paths
                Ok(())
            }
            TopLevel::UseRust(_) => {
                // Already handled in use statements
                Ok(())
            }
            TopLevel::Type(type_decl) => self.generate_type_decl(type_decl),
            TopLevel::TypeAlias(alias) => self.generate_type_alias(alias),
            TopLevel::Class(class) => {
                if std::env::var("LIVA_DEBUG").is_ok() {
                    println!("DEBUG: Generating class {}", class.name);
                }
                self.generate_class(class)
            }
            TopLevel::Enum(enum_decl) => self.generate_enum(enum_decl),
            TopLevel::Function(func) => self.generate_function(func),
            TopLevel::Test(test) => self.generate_test(test),
            TopLevel::ExprStmt(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }
            TopLevel::ConstDecl(const_decl) => {
                write!(self.output, "const {}: ", const_decl.name.to_uppercase()).unwrap();
                let type_str = if let Some(type_ref) = &const_decl.type_ref {
                    let rust_type = type_ref.to_rust_type();
                    // B31: const string can't use String (heap-allocated), must use &str
                    if rust_type == "String" {
                        "&str".to_string()
                    } else {
                        rust_type
                    }
                } else {
                    self.infer_const_type(&const_decl.init)
                };
                self.output.push_str(&type_str);
                self.output.push_str(" = ");
                // B31: For const string, don't add .to_string()
                let is_const_str = type_str == "&str";
                if is_const_str {
                    if let Expr::Literal(Literal::String(s)) = &const_decl.init {
                        write!(self.output, "\"{}\"", s).unwrap();
                    } else {
                        self.generate_expr(&const_decl.init)?;
                    }
                } else {
                    self.generate_expr(&const_decl.init)?;
                }
                self.output.push_str(";\n");
                Ok(())
            }
        }
    }

    fn generate_type_decl(&mut self, type_decl: &TypeDecl) -> Result<()> {
        // Interfaces in Liva are compile-time only contracts.
        // They are validated by the semantic analyzer but do NOT generate any Rust code.
        // Classes implementing interfaces just need to have the required methods.
        // Interface method signatures are collected in generate_program() for type inference.

        // Only generate a comment for documentation purposes
        self.writeln(&format!(
            "// Interface: {} (compile-time validation only)",
            type_decl.name
        ));

        Ok(())
    }

    fn generate_type_alias(&mut self, alias: &TypeAliasDecl) -> Result<()> {
        // Store type alias for expansion during type annotation generation
        self.type_aliases.insert(
            alias.name.clone(),
            (alias.type_params.clone(), alias.target_type.clone()),
        );
        // Type aliases in Liva are expanded inline during type checking
        // We don't generate Rust type aliases to keep codegen simple
        Ok(())
    }

    /// Expand type aliases in a TypeRef to get the final Rust type string
    fn expand_type_alias(&mut self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if it's a type alias
                if let Some((alias_params, target_type)) = self.type_aliases.get(name).cloned() {
                    // If the alias has no type parameters, just expand the target
                    if alias_params.is_empty() {
                        return self.expand_type_alias(&target_type);
                    }
                    // If it has type parameters but no args, just expand (error should be caught in semantic)
                    return self.expand_type_alias(&target_type);
                }
                // Not a type alias, use the normal to_rust_type conversion
                type_ref.to_rust_type()
            }
            TypeRef::Generic { base, args } => {
                // Check if the base is a type alias
                if let Some((alias_params, target_type)) = self.type_aliases.get(base).cloned() {
                    // Substitute type parameters
                    let substituted =
                        self.substitute_type_params_codegen(&target_type, &alias_params, args);
                    return self.expand_type_alias(&substituted);
                }
                // Not a type alias, recursively expand arguments
                let expanded_args: Vec<String> =
                    args.iter().map(|arg| self.expand_type_alias(arg)).collect();
                format!("{}<{}>", base, expanded_args.join(", "))
            }
            TypeRef::Array(inner) => {
                format!("Vec<{}>", self.expand_type_alias(inner))
            }
            TypeRef::Optional(inner) => {
                format!("Option<{}>", self.expand_type_alias(inner))
            }
            TypeRef::Fallible(inner) => {
                format!("Result<{}, liva_rt::Error>", self.expand_type_alias(inner))
            }
            TypeRef::Tuple(types) => {
                let types_str: Vec<String> =
                    types.iter().map(|t| self.expand_type_alias(t)).collect();
                // Rust requires trailing comma for single-element tuples
                if types.len() == 1 {
                    format!("({},)", types_str.join(", "))
                } else {
                    format!("({})", types_str.join(", "))
                }
            }
            TypeRef::Union(types) => {
                // For union types, register and generate a Rust enum
                let type_names: Vec<String> =
                    types.iter().map(|t| self.expand_type_alias(t)).collect();

                // Register this union for enum generation
                self.union_types.insert(type_names.clone());

                // Generate union enum name
                format!("Union_{}", type_names.join("_"))
            }
            TypeRef::Map(key, value) => {
                format!("std::collections::HashMap<{}, {}>",
                    self.expand_type_alias(key),
                    self.expand_type_alias(value))
            }
            TypeRef::Set(inner) => {
                format!("std::collections::HashSet<{}>",
                    self.expand_type_alias(inner))
            }
        }
    }

    /// Substitute type parameters in a TypeRef (for codegen)
    fn substitute_type_params_codegen(
        &self,
        type_ref: &TypeRef,
        params: &[TypeParameter],
        args: &[TypeRef],
    ) -> TypeRef {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if this name is one of the type parameters
                for (i, param) in params.iter().enumerate() {
                    if &param.name == name {
                        return args[i].clone();
                    }
                }
                // Not a type parameter, return as-is
                type_ref.clone()
            }
            TypeRef::Array(inner) => TypeRef::Array(Box::new(
                self.substitute_type_params_codegen(inner, params, args),
            )),
            TypeRef::Optional(inner) => TypeRef::Optional(Box::new(
                self.substitute_type_params_codegen(inner, params, args),
            )),
            TypeRef::Fallible(inner) => TypeRef::Fallible(Box::new(
                self.substitute_type_params_codegen(inner, params, args),
            )),
            TypeRef::Tuple(elements) => TypeRef::Tuple(
                elements
                    .iter()
                    .map(|elem| self.substitute_type_params_codegen(elem, params, args))
                    .collect(),
            ),
            TypeRef::Union(types) => TypeRef::Union(
                types
                    .iter()
                    .map(|ty| self.substitute_type_params_codegen(ty, params, args))
                    .collect(),
            ),
            TypeRef::Generic {
                base,
                args: inner_args,
            } => {
                // Recursively substitute in base and all arguments
                let substituted_base = match self.substitute_type_params_codegen(
                    &TypeRef::Simple(base.clone()),
                    params,
                    args,
                ) {
                    TypeRef::Simple(name) => name,
                    _ => base.clone(), // Shouldn't happen
                };

                TypeRef::Generic {
                    base: substituted_base,
                    args: inner_args
                        .iter()
                        .map(|arg| self.substitute_type_params_codegen(arg, params, args))
                        .collect(),
                }
            }
            TypeRef::Map(key, value) => TypeRef::Map(
                Box::new(self.substitute_type_params_codegen(key, params, args)),
                Box::new(self.substitute_type_params_codegen(value, params, args)),
            ),
            TypeRef::Set(inner) => TypeRef::Set(
                Box::new(self.substitute_type_params_codegen(inner, params, args)),
            ),
        }
    }

    fn generate_class(&mut self, class: &ClassDecl) -> Result<()> {
        // Check if this is actually an interface (no constructor, methods without bodies)
        // Interfaces are compile-time only and don't generate Rust code
        let has_constructor = class
            .members
            .iter()
            .any(|m| matches!(m, Member::Method(method) if method.name == "constructor"));
        let all_methods_abstract = class.members.iter().all(|m| {
            match m {
                Member::Method(method) => method.body.is_none() && method.expr_body.is_none(),
                Member::Field(_) => false, // Fields without init are fine for interfaces
            }
        });
        let has_only_methods = class.members.iter().all(|m| matches!(m, Member::Method(_)));

        // If no constructor and all methods are abstract (no body), it's an interface
        if !has_constructor && all_methods_abstract && has_only_methods {
            self.writeln(&format!(
                "// Interface: {} (compile-time validation only)",
                class.name
            ));
            return Ok(());
        }

        // Auto-detect data classes: if a class has fields but no explicit constructor,
        // it's automatically a data class (auto-derive constructor, PartialEq, Display).
        // This replaces the old `data` keyword — the compiler infers it from structure.
        let has_fields = class
            .members
            .iter()
            .any(|m| matches!(m, Member::Field(_)));
        let is_data = !has_constructor && has_fields;

        // Generate default functions for optional fields with init values
        let needs_serde_early = class.needs_serde || self.serde_classes.contains(&class.name);
        if needs_serde_early {
            for member in &class.members {
                if let Member::Field(field) = member {
                    if field.is_optional && field.init.is_some() {
                        self.generate_field_default_function(&class.name, field)?;
                    }
                }
            }
        }

        // Bug #45-46, #54: Infer bounds for type parameters based on usage
        let inferred_bounds = self.infer_type_param_bounds(class);

        // Format type parameters with both explicit constraints and inferred bounds
        let type_params_str = if !class.type_params.is_empty() {
            let params: Vec<String> = class
                .type_params
                .iter()
                .map(|tp| {
                    let mut all_bounds = Vec::new();

                    // Add explicit constraints first
                    if !tp.constraints.is_empty() {
                        let rust_bounds = self.trait_registry.generate_rust_bounds(&tp.constraints);
                        // Remove leading ": " from rust_bounds
                        let bounds_part = rust_bounds.trim_start_matches(": ");
                        all_bounds.push(bounds_part.to_string());
                    }

                    // Add inferred bounds
                    if let Some(inferred) = inferred_bounds.get(&tp.name) {
                        for bound in inferred {
                            // Check if bound is already included (from explicit constraints)
                            let bound_str = bound.as_str();
                            if !all_bounds.iter().any(|b| b.contains(bound_str)) {
                                all_bounds.push(bound.clone());
                            }
                        }
                    }

                    if all_bounds.is_empty() {
                        tp.name.clone()
                    } else {
                        format!("{}: {}", tp.name, all_bounds.join(" + "))
                    }
                })
                .collect();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };

        // Phase 2: Generate serde derives if class is used with JSON.parse or JSON.stringify (B46)
        // Data classes (auto-detected) also get PartialEq
        // B99 fix: Don't derive Default if any field is an enum type (enums with data don't impl Default)
        let needs_serde = class.needs_serde || self.serde_classes.contains(&class.name);
        let has_enum_field = class.members.iter().any(|m| {
            if let crate::ast::Member::Field(f) = m {
                if let Some(ref type_ref) = f.type_ref {
                    if let crate::ast::TypeRef::Simple(ref name) = type_ref {
                        return self.enum_names.contains(name);
                    }
                }
            }
            false
        });
        let can_default = !has_enum_field;
        let derives = if is_data && needs_serde && can_default {
            "#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]"
        } else if is_data && needs_serde {
            "#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]"
        } else if is_data && can_default {
            "#[derive(Debug, Clone, Default, PartialEq)]"
        } else if is_data {
            "#[derive(Debug, Clone, PartialEq)]"
        } else if needs_serde && can_default {
            "#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]"
        } else if needs_serde {
            "#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]"
        } else if can_default {
            "#[derive(Debug, Clone, Default, PartialEq)]"
        } else {
            "#[derive(Debug, Clone, PartialEq)]"
        };

        // Generate struct (interfaces are validated at compile-time, no runtime representation)
        if !class.implements.is_empty() {
            self.writeln(&format!(
                "// {} implements {}",
                class.name,
                class.implements.join(", ")
            ));
        }
        self.writeln(derives);
        self.writeln(&format!("pub struct {}{} {{", class.name, type_params_str));
        self.indent();

        for member in &class.members {
            if let Member::Field(field) = member {
                self.generate_field(field, needs_serde, Some(&class.name))?;
            }
        }

        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        // Generate impl block
        let _has_methods = class.members.iter().any(|m| matches!(m, Member::Method(_)));
        let has_fields = class.members.iter().any(|m| matches!(m, Member::Field(_)));

        // Find constructor method
        let constructor = class.members.iter().find_map(|m| {
            if let Member::Method(method) = m {
                if method.name == "constructor" {
                    Some(method)
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Format type parameters for impl block (with same bounds as struct)
        let impl_type_params = if !class.type_params.is_empty() {
            let params: Vec<String> = class
                .type_params
                .iter()
                .map(|tp| {
                    let mut all_bounds = Vec::new();

                    // Add explicit constraints first
                    if !tp.constraints.is_empty() {
                        let rust_bounds = self.trait_registry.generate_rust_bounds(&tp.constraints);
                        let bounds_part = rust_bounds.trim_start_matches(": ");
                        all_bounds.push(bounds_part.to_string());
                    }

                    // Add inferred bounds (same as struct)
                    if let Some(inferred) = inferred_bounds.get(&tp.name) {
                        for bound in inferred {
                            let bound_str = bound.as_str();
                            if !all_bounds.iter().any(|b| b.contains(bound_str)) {
                                all_bounds.push(bound.clone());
                            }
                        }
                    }

                    if all_bounds.is_empty() {
                        tp.name.clone()
                    } else {
                        format!("{}: {}", tp.name, all_bounds.join(" + "))
                    }
                })
                .collect();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };

        let impl_type_args = if !class.type_params.is_empty() {
            let args: Vec<String> = class.type_params.iter().map(|tp| tp.name.clone()).collect();
            format!("<{}>", args.join(", "))
        } else {
            String::new()
        };

        self.writeln(&format!(
            "impl{} {}{} {{",
            impl_type_params, class.name, impl_type_args
        ));
        self.indent();

        // Generate constructor
        if let Some(constructor_method) = constructor {
            // SH-002: Full constructor codegen — constructors work like any other method.
            // Two-phase approach:
            //   Phase 1: Walk body statements, emit non-field-assignment statements normally,
            //            collect this.field = expr as (field_name, expr) pairs.
            //   Phase 2: Emit Self { field: expr, ... } with collected values + defaults.
            self.write_indent();
            write!(self.output, "pub fn new(").unwrap();
            let params_str = self.generate_params(
                &constructor_method.params,
                false,
                Some(class),
                Some("constructor"),
                None,
            )?;
            // For optional parameters (Option<T>), use impl Into<Option<T>> for ergonomic callers
            let params_str = {
                let mut result = String::new();
                for (i, part) in params_str.split(", ").enumerate() {
                    if i > 0 { result.push_str(", "); }
                    if let Some(colon_pos) = part.find(": Option<") {
                        let name = &part[..colon_pos];
                        let type_str = &part[colon_pos + 2..];
                        result.push_str(name);
                        result.push_str(": impl Into<");
                        result.push_str(type_str);
                        result.push('>');
                    } else {
                        result.push_str(part);
                    }
                }
                result
            };
            write!(self.output, "{}) -> Self {{\n", params_str).unwrap();
            self.indent();

            // Phase 1: Walk body, emit non-field stmts, collect field assignments
            // Use IndexMap-like Vec to preserve last-write-wins while keeping insertion order
            let mut field_assignments: Vec<(String, &Expr)> = Vec::new();
            let mut assigned_fields: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            if let Some(body) = &constructor_method.body {
                self.in_constructor = true;

                for stmt in &body.stmts {
                    // Check if this is a top-level this.field = expr assignment
                    let is_field_assign = if let Stmt::Assign(assign) = stmt {
                        if let Expr::Member { object, property } = &assign.target {
                            if let Expr::Identifier(obj_name) = object.as_ref() {
                                if obj_name == "this" {
                                    let field_name = self.sanitize_name(property);
                                    // Remove previous assignment to same field (last write wins)
                                    field_assignments.retain(|(f, _)| *f != field_name);
                                    assigned_fields.insert(field_name.clone());
                                    field_assignments.push((field_name, &assign.value));
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !is_field_assign {
                        // Not a field assignment → generate normally (let, if, while, etc.)
                        self.generate_stmt(stmt)?;
                    }
                }

                self.in_constructor = false;
            }

            // Phase 2: Evaluate field expressions in SOURCE ORDER into temporaries,
            // then emit Self { field: temp, ... } in STRUCT order.
            // This prevents Rust move-before-borrow errors when the constructor
            // borrows a value before moving it, but struct order differs.
            for (field_name, value_expr) in &field_assignments {
                self.write_indent();
                write!(self.output, "let __field_{} = ", field_name).unwrap();
                self.generate_expr(value_expr)?;

                // Check if value is a string literal and needs .to_string()
                if matches!(value_expr, Expr::Literal(Literal::String(_))) {
                    self.output.push_str(".to_string()");
                }

                // Check if field is optional (needs .into())
                let is_opt_field = class.members.iter().any(|m| {
                    if let Member::Field(f) = m {
                        let f_name = self.sanitize_name(&f.name);
                        f_name == *field_name && (f.is_optional || matches!(&f.type_ref, Some(TypeRef::Optional(_))))
                    } else {
                        false
                    }
                });
                if is_opt_field {
                    self.output.push_str(".into()");
                }
                self.output.push_str(";\n");
            }

            self.write_indent();
            self.output.push_str("Self {\n");
            self.indent();

            for member in &class.members {
                if let Member::Field(field) = member {
                    let field_name = self.sanitize_name(&field.name);

                    if assigned_fields.contains(&field_name) {
                        // Use the temp variable
                        self.writeln(&format!("{}: __field_{},", field_name, field_name));
                    } else {
                        // Field not assigned in constructor - use default value
                        let default_value = if field.is_optional {
                            "None".to_string()
                        } else if let Some(init_expr) = &field.init {
                            let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                                && field.type_ref.as_ref().map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String")).unwrap_or(false);

                            let old_output = std::mem::take(&mut self.output);
                            self.generate_expr(init_expr)?;
                            let value = std::mem::replace(&mut self.output, old_output);

                            if needs_string_conversion {
                                format!("{}.to_string()", value)
                            } else {
                                value
                            }
                        } else {
                            match field.type_ref.as_ref() {
                                Some(type_ref) => match type_ref {
                                    TypeRef::Simple(name) => match name.as_str() {
                                        "number" | "int" => "0".to_string(),
                                        "float" => "0.0".to_string(),
                                        "string" => "String::new()".to_string(),
                                        "bool" => "false".to_string(),
                                        "char" => "'\\0'".to_string(),
                                        _ => "Default::default()".to_string(),
                                    },
                                    TypeRef::Array(_) => "Vec::new()".to_string(),
                                    _ => "Default::default()".to_string(),
                                },
                                None => "Default::default()".to_string(),
                            }
                        };
                        self.write_indent();
                        self.writeln(&format!("{}: {},", field_name, default_value));
                    }
                }
            }

            self.dedent();
            self.write_indent();
            self.output.push_str("}\n");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');
        } else if has_fields {
            // Default constructor
            // Data classes get a constructor with all fields as parameters
            if is_data {
                // Bug #96 fix: Check if ALL fields have default values
                let all_fields_have_defaults = class.members.iter().all(|m| {
                    match m {
                        Member::Field(field) => field.init.is_some(),
                        _ => true, // non-field members don't count
                    }
                });

                // Collect field info for the constructor signature
                let fields: Vec<(&str, String)> = class
                    .members
                    .iter()
                    .filter_map(|m| {
                        if let Member::Field(field) = m {
                            let field_name_raw = &field.name;
                            let rust_type = if let Some(type_ref) = &field.type_ref {
                                type_ref.to_rust_type()
                            } else {
                                "String".to_string()
                            };
                            Some((field_name_raw.as_str(), rust_type))
                        } else {
                            None
                        }
                    })
                    .collect();

                if all_fields_have_defaults {
                    // Bug #96: All fields have defaults → generate no-arg new() using defaults
                    self.write_indent();
                    self.output.push_str("pub fn new() -> Self {\n");
                    self.indent();
                    self.writeln("Self {");
                    self.indent();
                    for member in &class.members {
                        if let Member::Field(field) = member {
                            let field_name = self.sanitize_name(&field.name);
                            write!(self.output, "{}", " ".repeat(self.indent_level * 4)).unwrap();
                            write!(self.output, "{}: ", field_name).unwrap();

                            let init_expr = field.init.as_ref().unwrap();
                            let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                                && field.type_ref.as_ref().map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String")).unwrap_or(false);

                            if field.is_optional {
                                self.output.push_str("Some(");
                            }

                            if needs_string_conversion {
                                self.generate_expr(init_expr)?;
                                self.output.push_str(".to_string()");
                            } else {
                                self.generate_expr(init_expr)?;
                            }

                            if field.is_optional {
                                self.output.push_str(")");
                            }

                            self.output.push_str(",\n");
                        }
                    }
                    self.dedent();
                    self.writeln("}");
                    self.dedent();
                    self.writeln("}");
                    self.output.push('\n');
                } else {
                    // Regular data class: constructor requires all fields as params
                    // Generate: pub fn new(field1: Type1, field2: Type2, ...) -> Self {
                self.write_indent();
                self.output.push_str("pub fn new(");
                for (i, (name, rust_type)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let sanitized = self.sanitize_name(name);
                    write!(self.output, "{}: {}", sanitized, rust_type).unwrap();
                }
                self.output.push_str(") -> Self {\n");
                self.indent();
                self.writeln("Self {");
                self.indent();
                for (name, _) in &fields {
                    let sanitized = self.sanitize_name(name);
                    self.write_indent();
                    self.writeln(&format!("{},", sanitized));
                }
                self.dedent();
                self.writeln("}");
                self.dedent();
                self.writeln("}");
                self.output.push('\n');
                } // end else (regular data class with all fields as params)
            } else {
                // Regular class without constructor — default no-arg constructor
                self.writeln(&format!("pub fn new() -> Self {{"));
                self.indent();
                self.writeln("Self {");
                self.indent();

                for member in &class.members {
                    if let Member::Field(field) = member {
                        self.write_indent();
                        let field_name = self.sanitize_name(&field.name);

                        // Use explicit init value if provided, otherwise use defaults
                        if let Some(init_expr) = &field.init {
                            write!(self.output, "{}: ", field_name).unwrap();

                            // Check if we need to convert string literal to String
                            let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                            && field.type_ref.as_ref().map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String")).unwrap_or(false);

                            // If field is optional, wrap the init value in Some()
                            if field.is_optional {
                                self.output.push_str("Some(");
                            }

                            if needs_string_conversion {
                                self.generate_expr(init_expr)?;
                                self.output.push_str(".to_string()");
                            } else {
                                self.generate_expr(init_expr)?;
                            }

                            if field.is_optional {
                                self.output.push_str(")");
                            }

                            self.output.push_str(",\n");
                        } else {
                            // Optional fields should default to None
                            let default_value = if field.is_optional {
                                "None".to_string()
                            } else {
                                match field.type_ref.as_ref() {
                                    Some(type_ref) => match type_ref {
                                        TypeRef::Simple(name) => match name.as_str() {
                                            "number" | "int" => "0".to_string(),
                                            "float" => "0.0".to_string(),
                                            "string" => "String::new()".to_string(),
                                            "bool" => "false".to_string(),
                                            "char" => "'\\0'".to_string(),
                                            _ => "Default::default()".to_string(),
                                        },
                                        _ => "Default::default()".to_string(),
                                    },
                                    None => "Default::default()".to_string(),
                                }
                            };
                            self.writeln(&format!("{}: {},", field_name, default_value));
                        }
                    }
                }

                self.dedent();
                self.writeln("}");
                self.dedent();
                self.writeln("}");
                self.output.push('\n');
            } // close else (regular class, not data)
        }

        // Generate other methods (excluding constructor)
        // First, register class field types so method codegen can resolve this.field types
        for member in &class.members {
            if let Member::Field(field) = member {
                let field_name = self.sanitize_name(&field.name);
                if let Some(type_ref) = &field.type_ref {
                    // Track array-typed fields: this.items where items: [string]
                    if let TypeRef::Array(elem_type) = type_ref {
                        self.array_vars.insert(field_name.clone());
                        let elem_type_name = match elem_type.as_ref() {
                            TypeRef::Simple(name) => name.clone(),
                            _ => "i32".to_string(),
                        };
                        self.typed_array_vars
                            .insert(field_name.clone(), elem_type_name);
                    }
                    // Track string-typed fields
                    if matches!(type_ref, TypeRef::Simple(s) if s == "string" || s == "String") {
                        self.string_vars.insert(field_name.clone());
                    }
                    // Bug #76 fix: Track Map-typed fields so this._field.mapMethod() routes through Map codegen
                    if matches!(type_ref, TypeRef::Map(_, _)) {
                        self.map_vars.insert(field_name.clone());
                    }
                    // Bug #76 fix: Track Set-typed fields so this._field.setMethod() routes through Set codegen
                    if matches!(type_ref, TypeRef::Set(_)) {
                        self.set_vars.insert(field_name.clone());
                    }
                }
            }
        }
        // B09: Pre-compute transitive &mut self methods before generating them
        self.compute_mut_self_methods(class);

        for member in &class.members {
            if let Member::Method(method) = member {
                if method.name != "constructor" {
                    self.write_indent();
                    write!(self.output, "// Generating method: {}\n", method.name).unwrap();
                    self.generate_method(method, Some(class))?;
                    self.output.push('\n');
                }
            }
        }

        self.dedent();
        self.writeln("}");

        // BUG-004 fix: Auto-generate Display impl for ALL classes with fields,
        // not just data classes. Classes with explicit constructors also need Display.
        if has_fields {
            self.output.push('\n');
            // B103 fix: For Display impl, add Display bound to type parameters
            let impl_display_type_params = if !class.type_params.is_empty() {
                let params: Vec<String> = class.type_params.iter().map(|tp| {
                    // Start with whatever bounds are already on impl_type_params
                    let mut all_bounds = Vec::new();
                    if !tp.constraints.is_empty() {
                        let rust_bounds = self.trait_registry.generate_rust_bounds(&tp.constraints);
                        let bounds_part = rust_bounds.trim_start_matches(": ");
                        all_bounds.push(bounds_part.to_string());
                    }
                    if let Some(inferred) = inferred_bounds.get(&tp.name) {
                        for bound in inferred {
                            let bound_str = bound.as_str();
                            if !all_bounds.iter().any(|b| b.contains(bound_str)) {
                                all_bounds.push(bound.clone());
                            }
                        }
                    }
                    // Add Display bound if not already present
                    if !all_bounds.iter().any(|b| b.contains("Display")) {
                        all_bounds.push("std::fmt::Display".to_string());
                    }
                    format!("{}: {}", tp.name, all_bounds.join(" + "))
                }).collect();
                format!("<{}>", params.join(", "))
            } else {
                String::new()
            };
            self.writeln(&format!(
                "impl{} std::fmt::Display for {}{} {{",
                impl_display_type_params, class.name, impl_type_args
            ));
            self.indent();
            self.writeln("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
            self.indent();

            let fields: Vec<&FieldDecl> = class
                .members
                .iter()
                .filter_map(|m| {
                    if let Member::Field(field) = m {
                        Some(field)
                    } else {
                        None
                    }
                })
                .collect();

            if fields.is_empty() {
                self.writeln(&format!("write!(f, \"{}\")", class.name));
            } else {
                self.write_indent();
                // Use push_str to avoid write! interpreting {{ as {
                // We need literal {{ and }} in the generated format string
                self.output
                    .push_str(&format!("write!(f, \"{} {{{{ ", class.name));
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let field_name = self.sanitize_name(&field.name);
                    // Use {:?} for types that don't implement Display (arrays, enums, maps, sets, optionals)
                    let needs_debug = match field.type_ref.as_ref() {
                        Some(TypeRef::Array(_)) => true,
                        Some(TypeRef::Map(_, _)) => true,
                        Some(TypeRef::Set(_)) => true,
                        Some(TypeRef::Optional(_)) => true,
                        Some(TypeRef::Simple(name)) => self.enum_names.contains(name),
                        _ => false,
                    };
                    if needs_debug {
                        self.output.push_str(&format!("{}: {{:?}}", field_name));
                    } else {
                        self.output.push_str(&format!("{}: {{}}", field_name));
                    }
                }
                self.output.push_str(" }}}}\"");
                for field in &fields {
                    let field_name = self.sanitize_name(&field.name);
                    write!(self.output, ", self.{}", field_name).unwrap();
                }
                self.output.push_str(")\n");
            }

            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
        }

        Ok(())
    }

    fn generate_field_default_function(
        &mut self,
        class_name: &str,
        field: &FieldDecl,
    ) -> Result<()> {
        let field_name = self.sanitize_name(&field.name);
        let func_name = format!("default_{}_{}", class_name.to_lowercase(), field_name);

        let base_type = if let Some(type_ref) = &field.type_ref {
            type_ref.to_rust_type()
        } else {
            "String".to_string()
        };

        self.writeln(&format!("fn {}() -> Option<{}>{{", func_name, base_type));
        self.indent();

        if let Some(init_expr) = &field.init {
            self.write_indent();
            self.output.push_str("Some(");

            // Check if we need to convert string literal to String
            let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                && field
                    .type_ref
                    .as_ref()
                    .map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String"))
                    .unwrap_or(false);

            if needs_string_conversion {
                self.generate_expr(init_expr)?;
                self.output.push_str(".to_string()");
            } else {
                self.generate_expr(init_expr)?;
            }

            self.output.push_str(")\n");
        }

        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        Ok(())
    }

    fn generate_field(
        &mut self,
        field: &FieldDecl,
        needs_serde: bool,
        class_name: Option<&str>,
    ) -> Result<()> {
        let vis = match field.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        let base_type = if let Some(type_ref) = &field.type_ref {
            type_ref.to_rust_type()
        } else {
            "()".to_string()
        };

        // Wrap in Option<T> if field is optional
        let type_str = if field.is_optional {
            format!("Option<{}>", base_type)
        } else {
            base_type
        };

        let field_name_rust = self.sanitize_name(&field.name);

        // Add serde attributes for optional fields
        if needs_serde && field.is_optional {
            // If the field has a default value, use serde default function
            if field.init.is_some() && class_name.is_some() {
                let func_name = format!(
                    "default_{}_{}",
                    class_name.unwrap().to_lowercase(),
                    field_name_rust
                );
                self.writeln(&format!("#[serde(default = \"{}\")]", func_name));
            }
            self.writeln("#[serde(skip_serializing_if = \"Option::is_none\")]");
        }

        // If serde is needed and the original name differs from snake_case, add rename attribute
        if needs_serde && field.name != field_name_rust {
            self.writeln(&format!("#[serde(rename = \"{}\")]", field.name));
        }

        self.writeln(&format!("{}{}: {},", vis, field_name_rust, type_str));
        Ok(())
    }

    fn infer_expr_type(&self, expr: &Expr, class: Option<&ClassDecl>) -> Option<String> {
        match expr {
            Expr::Tuple(elements) => {
                // Infer tuple type from element expressions
                let mut element_types = Vec::new();
                for elem in elements {
                    if let Some(type_str) = self.infer_expr_type(elem, class) {
                        // Extract just the type part (remove " -> " prefix)
                        let type_part = type_str.trim_start_matches(" -> ");
                        element_types.push(type_part.to_string());
                    } else {
                        // If we can't infer an element type, fall back to generic
                        element_types.push("i32".to_string());
                    }
                }
                if !element_types.is_empty() {
                    Some(format!(" -> ({})", element_types.join(", ")))
                } else {
                    Some(" -> ()".to_string()) // Empty tuple
                }
            }
            Expr::Member { object, property } => {
                // Check if this is accessing a field of 'this'
                if let Expr::Identifier(obj) = object.as_ref() {
                    if obj == "this" && class.is_some() {
                        // Find the field type
                        for member in &class.unwrap().members {
                            if let Member::Field(field) = member {
                                if field.name == *property {
                                    let rust_type = field
                                        .type_ref
                                        .as_ref()
                                        .map(|t| t.to_rust_type())
                                        .unwrap_or_else(|| "String".to_string());
                                    // Return owned type - cloning will be handled in code generation
                                    return Some(format!(" -> {}", rust_type));
                                }
                            }
                        }
                    }
                }
                None
            }
            // B18 fix: Handle index access — this.field[i] → element type of array field
            Expr::Index { object, .. } => {
                // Try to infer the base array type, then return its element type
                if let Some(array_type) = self.infer_expr_type(object, class) {
                    let type_part = array_type.trim_start_matches(" -> ");
                    // Vec<T> → T
                    if type_part.starts_with("Vec<") && type_part.ends_with('>') {
                        let inner = &type_part[4..type_part.len() - 1];
                        return Some(format!(" -> {}", inner));
                    }
                }
                None
            }
            // B18 fix: Handle identifier references — look up from method params
            Expr::Identifier(name) => {
                if let Some(cls) = class {
                    // Check method params — NOT directly available here, but check class fields
                    for member in &cls.members {
                        if let Member::Field(field) = member {
                            if field.name == *name {
                                let rust_type = field
                                    .type_ref
                                    .as_ref()
                                    .map(|t| t.to_rust_type())
                                    .unwrap_or_else(|| "String".to_string());
                                return Some(format!(" -> {}", rust_type));
                            }
                        }
                    }
                }
                None
            }
            Expr::Binary { op, left, right, .. } => {
                // Simple heuristics for binary operations
                match op {
                    BinOp::Lt
                    | BinOp::Le
                    | BinOp::Gt
                    | BinOp::Ge
                    | BinOp::Eq
                    | BinOp::Ne
                    | BinOp::And
                    | BinOp::Or => Some(" -> bool".to_string()),
                    // B18 fix: Arithmetic ops → infer from operands
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // Try to infer from left operand first, then right
                        self.infer_expr_type(left, class)
                            .or_else(|| self.infer_expr_type(right, class))
                    }
                    _ => None,
                }
            }
            Expr::Literal(lit) => match lit {
                Literal::String(_) => Some(" -> String".to_string()), // String literals converted to String in tuples
                Literal::Int(_) => Some(" -> i32".to_string()),
                Literal::Float(_) => Some(" -> f64".to_string()),
                Literal::Bool(_) => Some(" -> bool".to_string()),
                _ => None,
            },
            // String templates (format! calls) return String
            Expr::Call(call) => {
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if name.starts_with("$") || name == "format" {
                        return Some(" -> String".to_string());
                    }
                }
                None
            }
            // String templates ($"...{expr}...") always return String
            Expr::StringTemplate { .. } => Some(" -> String".to_string()),
            // B18 fix: Ternary expressions — infer from then/else branches
            Expr::Ternary { then_expr, else_expr, .. } => {
                self.infer_expr_type(then_expr, class)
                    .or_else(|| self.infer_expr_type(else_expr, class))
            }
            // B18 fix: Unary not → bool
            Expr::Unary { op, .. } if matches!(op, UnOp::Not) => {
                Some(" -> bool".to_string())
            }
            _ => None,
        }
    }

    fn infer_param_type_from_class(
        &self,
        param_name: &str,
        class: &ClassDecl,
        method_name: Option<&str>,
    ) -> Option<String> {
        // Determine candidate field name based on method/prefix
        let field_name = if let Some(method) = method_name {
            if method.starts_with("set") || method.starts_with("get") {
                let without_prefix = if method.starts_with("set") {
                    method.strip_prefix("set").unwrap_or(method)
                } else if method.starts_with("get") {
                    method.strip_prefix("get").unwrap_or(method)
                } else {
                    method
                };
                let mut chars = without_prefix.chars();
                if let Some(first) = chars.next() {
                    format!("{}{}", first.to_lowercase(), chars.as_str())
                } else {
                    param_name.to_string()
                }
            } else {
                param_name.to_string()
            }
        } else {
            param_name.to_string()
        };

        // Look for matching field in class
        if let Some(fields) = self.class_fields.get(&class.name) {
            if fields.contains(&field_name) || fields.contains(&format!("_{}", field_name)) {
                // Search for type in class members
                for m in &class.members {
                    if let Member::Field(f) = m {
                        if f.name == field_name || f.name == format!("_{}", field_name) {
                            return f.type_ref.as_ref().map(|t| t.to_rust_type());
                        }
                    }
                }
                // Fallback for common field names
                return Some(match field_name.as_str() {
                    "name" => "String".to_string(),
                    "age" => "i32".to_string(),
                    _ => "i32".to_string(),
                });
            }
        }
        None
    }

    fn generate_method(&mut self, method: &MethodDecl, class: Option<&ClassDecl>) -> Result<()> {
        // Track current function name for error traces (ClassName.method)
        let method_trace_name = if let Some(cls) = class {
            format!("{}.{}", cls.name, method.name)
        } else {
            method.name.clone()
        };
        let prev_function_name = std::mem::replace(&mut self.current_function_name, method_trace_name);

        // Pre-analyze: collect variables that are mutated after declaration
        self.mutated_vars.clear();
        if let Some(body) = &method.body {
            let mut temp_mutated = std::collections::HashSet::new();
            self.collect_mutated_vars_in_block(body, &mut temp_mutated);
            self.mutated_vars = temp_mutated;
        }

        let vis = match method.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        let async_kw = if method.is_async_inferred {
            "async "
        } else {
            ""
        };

        let type_params = if !method.type_params.is_empty() {
            let bounded: Vec<String> = method
                .type_params
                .iter()
                .map(|param| {
                    if !param.constraints.is_empty() {
                        // Use trait registry to get complete Rust trait bounds
                        let rust_bounds =
                            self.trait_registry.generate_rust_bounds(&param.constraints);
                        format!("{}{}", param.name, rust_bounds)
                    } else {
                        param.name.clone()
                    }
                })
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };

        let params_str = self.generate_params(
            &method.params,
            true,
            class,
            Some(&method.name),
            Some(method),
        )?;

        let return_type = if let Some(ret) = &method.return_type {
            // Bug #70 fix: Wrap return type in Result if method contains fail
            if method.contains_fail {
                format!(" -> Result<{}, liva_rt::Error>", ret.to_rust_type())
            } else {
                format!(" -> {}", ret.to_rust_type())
            }
        } else {
            // First, try to find return type from implemented interfaces
            let interface_return_type = class.and_then(|c| {
                for iface_name in &c.implements {
                    if let Some(methods) = self.interface_methods.get(iface_name) {
                        if let Some(ret_type) = methods.get(&method.name) {
                            return Some(format!(" -> {}", ret_type.to_rust_type()));
                        }
                    }
                }
                None
            });

            if let Some(ret) = interface_return_type {
                ret
            } else if let Some(expr) = &method.expr_body {
                // Try to infer return type from expression
                self.infer_expr_type(expr, class)
                    .unwrap_or_else(|| " -> ()".to_string())
            } else {
                String::new()
            }
        };

        self.write_indent();
        write!(
            self.output,
            "{}{}fn {}{}({}){}",
            vis,
            async_kw,
            self.sanitize_name(&method.name),
            type_params,
            params_str,
            return_type
        )
        .unwrap();

        self.in_method = true;
        let prev_class_name = self.current_class_name.take();
        self.current_class_name = class.map(|c| c.name.clone());
        let prev_method_is_mut = self.current_method_is_mut;
        let is_setter = method.name.starts_with("set");
        self.current_method_is_mut = is_setter || self.method_modifies_self(method);
        let prev_fallible = self.in_fallible_function;
        self.in_fallible_function = method.contains_fail;

        if let Some(expr) = &method.expr_body {
            self.output.push_str(" {\n");
            self.indent();

            // Generate destructuring code for parameters
            self.generate_param_destructuring(&method.params)?;

            self.write_indent();
            // B104 fix: Detect if return type is a type parameter (T) and method is &self (not mut).
            // If so, the expression accesses a field through &self and needs .clone() to avoid moving.
            let class_type_param_names: std::collections::HashSet<String> = class
                .map(|c| c.type_params.iter().map(|tp| tp.name.clone()).collect())
                .unwrap_or_default();
            let returns_type_param = method.return_type.as_ref().map_or(false, |rt| {
                if let TypeRef::Simple(name) = rt {
                    class_type_param_names.contains(name)
                } else { false }
            });
            let needs_clone = returns_type_param && !self.current_method_is_mut;
            if method.contains_fail {
                self.output.push_str("Ok(");
                self.generate_expr(expr)?;
                if needs_clone { self.output.push_str(".clone()"); }
                self.output.push(')');
            } else {
                self.generate_expr(expr)?;
                if needs_clone { self.output.push_str(".clone()"); }
            }
            self.output.push('\n');
            self.dedent();
            self.writeln("}");
        } else if let Some(body) = &method.body {
            self.output.push_str(" {\n");
            self.indent();

            // Generate destructuring code for parameters
            self.generate_param_destructuring(&method.params)?;

            self.generate_block_inner(body)?;
            self.dedent();
            self.writeln("}");
        }
        self.in_fallible_function = prev_fallible;
        self.in_method = false;
        self.current_class_name = prev_class_name;
        self.current_method_is_mut = prev_method_is_mut;

        // Restore previous function name
        self.current_function_name = prev_function_name;

        Ok(())
    }

    /// Check if a TypeRef refers to the given enum name (direct recursion).
    /// Returns true for: Simple("Expr") when enum_name is "Expr".
    fn is_recursive_field(type_ref: &TypeRef, enum_name: &str) -> bool {
        matches!(type_ref, TypeRef::Simple(name) if name == enum_name)
    }

    fn generate_enum(&mut self, enum_decl: &EnumDecl) -> Result<()> {
        // B14: Check if all variants are unit variants — only then derive Default
        // Enums with data variants can't easily derive Default
        let all_unit = enum_decl.variants.iter().all(|v| v.fields.is_empty());

        // Pre-scan: detect recursive fields that need auto-boxing
        let mut boxed_fields_for_enum: std::collections::HashMap<String, std::collections::HashSet<String>> =
            std::collections::HashMap::new();
        for variant in &enum_decl.variants {
            for field in &variant.fields {
                if Self::is_recursive_field(&field.type_ref, &enum_decl.name) {
                    boxed_fields_for_enum
                        .entry(variant.name.clone())
                        .or_default()
                        .insert(field.name.clone());
                }
            }
        }
        if !boxed_fields_for_enum.is_empty() {
            self.boxed_enum_fields
                .insert(enum_decl.name.clone(), boxed_fields_for_enum);
        }

        // Generate Rust enum with derive macros
        // FIX-5: Unit enums (no data fields) get Copy to avoid move errors
        if all_unit && !enum_decl.variants.is_empty() {
            self.writeln("#[derive(Debug, Clone, Copy, PartialEq, Default)]");
        } else {
            self.writeln("#[derive(Debug, Clone, PartialEq)]");
        }

        // Type parameters
        if enum_decl.type_params.is_empty() {
            writeln!(self.output, "enum {} {{", enum_decl.name).unwrap();
        } else {
            let params: Vec<String> = enum_decl
                .type_params
                .iter()
                .map(|tp| tp.name.clone())
                .collect();
            writeln!(
                self.output,
                "enum {}<{}> {{",
                enum_decl.name,
                params.join(", ")
            )
            .unwrap();
        }

        self.indent();

        for (idx, variant) in enum_decl.variants.iter().enumerate() {
            self.write_indent();
            if variant.fields.is_empty() {
                // B14: Mark first unit variant as default
                if idx == 0 && all_unit {
                    writeln!(self.output, "#[default]").unwrap();
                    self.write_indent();
                }
                // Unit variant: Red,
                writeln!(self.output, "{},", variant.name).unwrap();
            } else {
                // Named fields variant: Circle { radius: f64 },
                write!(self.output, "{} {{ ", variant.name).unwrap();
                for (i, field) in variant.fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Auto-boxing: recursive fields get wrapped in Box<T>
                    let rust_type = if Self::is_recursive_field(&field.type_ref, &enum_decl.name) {
                        format!("Box<{}>", field.type_ref.to_rust_type())
                    } else {
                        field.type_ref.to_rust_type()
                    };
                    write!(
                        self.output,
                        "{}: {}",
                        self.sanitize_name(&field.name),
                        rust_type
                    )
                    .unwrap();
                }
                self.output.push_str(" },\n");
            }
        }

        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        // Generate Display impl for simple enums (all unit variants)
        let all_unit = enum_decl.variants.iter().all(|v| v.fields.is_empty());
        if all_unit && enum_decl.type_params.is_empty() {
            writeln!(
                self.output,
                "impl std::fmt::Display for {} {{",
                enum_decl.name
            )
            .unwrap();
            self.indent();
            self.writeln("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
            self.indent();
            self.writeln("match self {");
            self.indent();
            for variant in &enum_decl.variants {
                self.write_indent();
                writeln!(
                    self.output,
                    "{}::{} => write!(f, \"{}\"),",
                    enum_decl.name, variant.name, variant.name
                )
                .unwrap();
            }
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');
        }

        Ok(())
    }

    fn generate_function(&mut self, func: &FunctionDecl) -> Result<()> {
        // Track current function name for error traces
        let prev_function_name = std::mem::replace(&mut self.current_function_name, func.name.clone());

        // Register default parameter values for call-site injection
        let defaults: Vec<(usize, Expr)> = func.params.iter().enumerate()
            .filter_map(|(i, p)| p.default.as_ref().map(|d| (i, d.clone())))
            .collect();
        if !defaults.is_empty() {
            self.function_defaults.insert(func.name.clone(), defaults);
        }

        // Pre-analyze: collect variables that are mutated after declaration
        self.mutated_vars.clear();
        if let Some(body) = &func.body {
            // Collect mutated variables
            let mut temp_mutated = std::collections::HashSet::new();
            self.collect_mutated_vars_in_block(body, &mut temp_mutated);
            self.mutated_vars = temp_mutated;
        }

        let (async_kw, tokio_attr) = if func.name == "main" && func.is_async_inferred {
            // For main function with async, use tokio::main attribute with async keyword
            ("async ", "#[tokio::main]\n")
        } else if func.is_async_inferred {
            ("async ", "")
        } else {
            ("", "")
        };

        let type_params = if !func.type_params.is_empty() {
            let bounded: Vec<String> = func
                .type_params
                .iter()
                .map(|param| {
                    if !param.constraints.is_empty() {
                        // Use trait registry to get complete Rust trait bounds
                        let rust_bounds =
                            self.trait_registry.generate_rust_bounds(&param.constraints);
                        format!("{}{}", param.name, rust_bounds)
                    } else {
                        param.name.clone()
                    }
                })
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };
        let params_str = self.generate_params(&func.params, false, None, None, None)?;

        // Handle fallibility - wrap return type in Result if function contains fail
        let return_type = if func.contains_fail {
            if let Some(ret) = &func.return_type {
                format!(" -> Result<{}, liva_rt::Error>", ret.to_rust_type())
            } else if let Some(expr) = &func.expr_body {
                let inner_type = self
                    .infer_expr_type(expr, None)
                    .unwrap_or_else(|| " -> i32".to_string())
                    .trim_start_matches(" -> ")
                    .to_string();
                format!(" -> Result<{}, liva_rt::Error>", inner_type)
            } else {
                " -> Result<(), liva_rt::Error>".to_string()
            }
        } else {
            if let Some(ret) = &func.return_type {
                format!(" -> {}", ret.to_rust_type())
            } else if let Some(expr) = &func.expr_body {
                // For expression-bodied functions without explicit return type, infer from the expression
                self.infer_expr_type(expr, None)
                    .unwrap_or_else(|| " -> i32".to_string())
            } else if func.body.is_some() {
                // For block-bodied functions, try to infer from return statements
                if let Some(body) = &func.body {
                    if self.block_has_return(body) {
                        // Try to infer type from return statement
                        self.infer_return_type_from_block(body)
                            .unwrap_or_else(|| " -> f64".to_string()) // Default to f64 as fallback
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        };

        write!(
            self.output,
            "{}{}fn {}{}({})",
            tokio_attr,
            async_kw,
            self.sanitize_name(&func.name),
            type_params,
            params_str
        )
        .unwrap();

        if !return_type.is_empty() {
            write!(self.output, "{}", return_type).unwrap();
        }

        if let Some(expr) = &func.expr_body {
            self.output.push_str(" {\n");
            self.indent();

            // Generate destructuring code for parameters
            self.generate_param_destructuring(&func.params)?;

            self.write_indent();
            let was_fallible = self.in_fallible_function;
            self.in_fallible_function = func.contains_fail;
            let was_optional = self.in_optional_function;
            self.in_optional_function = matches!(&func.return_type, Some(TypeRef::Optional(_)));

            // Track return type for division casting (Bug #52)
            let prev_return_type = self.current_return_type.take();
            if let Some(ret) = &func.return_type {
                self.current_return_type = Some(ret.to_rust_type());
            }

            if func.contains_fail {
                // Check if the expression already returns a Result (like a fallible ternary)
                let expr_returns_result = matches!(expr, Expr::Ternary { then_expr, else_expr, .. }
                    if self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr));

                if !expr_returns_result {
                    self.output.push_str("Ok(");
                }
                self.generate_expr(expr)?;
                if !expr_returns_result {
                    self.output.push(')');
                }
            } else if self.in_optional_function {
                // BUG-006: Wrap expression body in Some() for optional return
                self.output.push_str("Some(");
                self.generate_expr(expr)?;
                self.output.push(')');
            } else {
                self.generate_expr(expr)?;
            }
            self.in_fallible_function = was_fallible;
            self.in_optional_function = was_optional;
            self.current_return_type = prev_return_type;
            self.output.push('\n');

            // Phase 4.2: Check for dead tasks
            self.check_dead_tasks();
            self.pending_tasks.clear();

            self.dedent();
            self.writeln("}");
        } else if let Some(body) = &func.body {
            self.output.push_str(" {\n");
            self.indent();

            // Generate destructuring code for parameters
            self.generate_param_destructuring(&func.params)?;

            let was_fallible = self.in_fallible_function;
            self.in_fallible_function = func.contains_fail;
            let was_optional = self.in_optional_function;
            self.in_optional_function = matches!(&func.return_type, Some(TypeRef::Optional(_)));

            // Track return type for division casting (Bug #52)
            let prev_return_type = self.current_return_type.take();
            if let Some(ret) = &func.return_type {
                self.current_return_type = Some(ret.to_rust_type());
            }

            self.generate_block_inner(body)?;
            // If function is fallible and doesn't end with explicit return, add Ok(())
            if func.contains_fail && !self.block_ends_with_return(body) {
                self.write_indent();
                self.writeln("Ok(())");
            }
            self.in_fallible_function = was_fallible;
            self.in_optional_function = was_optional;
            self.current_return_type = prev_return_type;

            // Phase 4.2: Check for dead tasks (tasks that were never awaited)
            self.check_dead_tasks();

            // Clear pending tasks for next function
            self.pending_tasks.clear();

            self.dedent();
            self.writeln("}");
        }

        // Restore previous function name
        self.current_function_name = prev_function_name;

        Ok(())
    }

    fn generate_test(&mut self, test: &TestDecl) -> Result<()> {
        self.writeln("#[test]");
        self.writeln(&format!(
            "fn test_{}() {{",
            self.sanitize_test_name(&test.name)
        ));
        self.indent();
        let was_in_test = self.in_test_block;
        self.in_test_block = true;

        // BUG-001 fix: Pre-analyze mutated variables in test body
        // (same as generate_function does for regular function bodies)
        let saved_mutated = std::mem::take(&mut self.mutated_vars);
        let mut temp_mutated = std::collections::HashSet::new();
        self.collect_mutated_vars_in_block(&test.body, &mut temp_mutated);
        self.mutated_vars = temp_mutated;

        self.generate_block_inner(&test.body)?;
        self.in_test_block = was_in_test;
        self.mutated_vars = saved_mutated;
        self.dedent();
        self.writeln("}");
        Ok(())
    }

    // ─── liva/test virtual library codegen ───────────────────────────

    /// Scan a describe block's statements to detect which lifecycle hooks are present.
    /// This is called before generating the block so we know what hooks to auto-invoke.
    fn scan_describe_for_hooks(&self, block: &BlockStmt) -> TestHookScope {
        let depth = self.test_hooks_stack.len();
        let mut scope = TestHookScope {
            depth,
            ..Default::default()
        };
        for stmt in &block.stmts {
            if let Stmt::Expr(expr_stmt) = stmt {
                if let Expr::Call(call) = &expr_stmt.expr {
                    if let Expr::Identifier(name) = call.callee.as_ref() {
                        match name.as_str() {
                            "beforeEach" => {
                                scope.has_before_each = true;
                                if let Some(Expr::Lambda(lambda)) = call.args.first() {
                                    scope.before_each_is_async =
                                        ast_lambda_body_has_async(&lambda.body);
                                }
                            }
                            "afterEach" => {
                                scope.has_after_each = true;
                                if let Some(Expr::Lambda(lambda)) = call.args.first() {
                                    scope.after_each_is_async =
                                        ast_lambda_body_has_async(&lambda.body);
                                }
                            }
                            "beforeAll" => scope.has_before_all = true,
                            "afterAll" => scope.has_after_all = true,
                            _ => {}
                        }
                    }
                }
            }
        }
        scope
    }

    /// Collect all hook function names that should be called for each test,
    /// traversing the entire hooks stack (parent describes + current).
    /// Returns (fn_name, is_async) pairs.
    fn collect_before_each_hooks(&self) -> Vec<(String, bool)> {
        let mut hooks = Vec::new();
        for (i, scope) in self.test_hooks_stack.iter().enumerate() {
            if scope.has_before_each {
                let name = if i == 0 {
                    "before_each".to_string()
                } else {
                    format!("before_each_{}", i)
                };
                hooks.push((name, scope.before_each_is_async));
            }
        }
        hooks
    }

    fn collect_after_each_hooks(&self) -> Vec<(String, bool)> {
        // After hooks run in reverse order (innermost first)
        let mut hooks = Vec::new();
        for (i, scope) in self.test_hooks_stack.iter().enumerate().rev() {
            if scope.has_after_each {
                let name = if i == 0 {
                    "after_each".to_string()
                } else {
                    format!("after_each_{}", i)
                };
                hooks.push((name, scope.after_each_is_async));
            }
        }
        hooks
    }

    /// Generate a describe() block → #[cfg(test)] mod test_name { use super::*; ... }
    fn generate_test_describe(&mut self, call: &CallExpr) -> Result<()> {
        // describe("name", () => { ... })
        if call.args.len() < 2 {
            return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                "E3000",
                "describe() requires 2 arguments",
                "describe(name: string, callback: () => void)",
            )));
        }

        // Extract the name string
        let mod_name = match &call.args[0] {
            Expr::Literal(Literal::String(s)) => self.sanitize_test_name(s),
            _ => "unnamed".to_string(),
        };

        // Extract the lambda body
        let lambda_body = match &call.args[1] {
            Expr::Lambda(lambda) => &lambda.body,
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    "describe() second argument must be a function",
                    "describe(\"name\", () => { ... })",
                )));
            }
        };

        self.writeln("#[cfg(test)]");
        self.writeln(&format!("mod test_{} {{", mod_name));
        self.indent();
        self.writeln("use super::*;");
        self.output.push('\n');

        let was_in_test = self.in_test_block;
        self.in_test_block = true;

        match lambda_body {
            LambdaBody::Block(block) => {
                // Pre-scan for lifecycle hooks and push scope
                let hook_scope = self.scan_describe_for_hooks(block);
                self.test_hooks_stack.push(hook_scope);
                self.generate_block_inner(block)?;

                // Generate beforeAll() call at module level if present
                // (beforeAll runs once when the module is loaded — we use a static + sync_once pattern,
                //  but for simplicity in Rust's test framework we rely on the function being there
                //  for tests to call; actually beforeAll/afterAll are best effort with #[ctor])
                // For now, beforeAll/afterAll functions are generated and we skip module-level call
                // since Rust tests don't have module-level setup. Users call them explicitly if needed.

                self.test_hooks_stack.pop();
            }
            LambdaBody::Expr(expr) => {
                self.test_hooks_stack.push(TestHookScope::default());
                self.write_indent();
                self.generate_expr(expr)?;
                self.output.push_str(";\n");
                self.test_hooks_stack.pop();
            }
        }

        self.in_test_block = was_in_test;
        self.dedent();
        self.writeln("}");
        Ok(())
    }

    /// Generate a test() call → #[test] fn test_name() { ... }
    /// If the lambda body contains async calls, generates #[tokio::test] async fn instead
    fn generate_test_case(&mut self, call: &CallExpr) -> Result<()> {
        // test("name", () => { ... })
        if call.args.len() < 2 {
            return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                "E3000",
                "test() requires 2 arguments",
                "test(name: string, callback: () => void)",
            )));
        }

        // Extract the name string — B109 fix: deduplicate collision
        let test_name = {
            let base = match &call.args[0] {
                Expr::Literal(Literal::String(s)) => self.sanitize_test_name(s),
                _ => "unnamed".to_string(),
            };
            let count = self.used_test_names.entry(base.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                format!("{}_{}", base, count)
            } else {
                base
            }
        };

        // Extract the lambda body
        let lambda_body = match &call.args[1] {
            Expr::Lambda(lambda) => &lambda.body,
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    "test() second argument must be a function",
                    "test(\"name\", () => { ... })",
                )));
            }
        };

        // Detect if the test body contains async calls or await expressions
        let is_async = ast_lambda_body_has_async(lambda_body);

        if is_async {
            self.writeln("#[tokio::test]");
            self.writeln(&format!("async fn test_{}() {{", test_name));
        } else {
            self.writeln("#[test]");
            self.writeln(&format!("fn test_{}() {{", test_name));
        }
        self.indent();

        let was_in_test = self.in_test_block;
        self.in_test_block = true;

        // BUG-001 fix: Pre-analyze mutated variables in test body
        let saved_mutated = std::mem::take(&mut self.mutated_vars);
        if let LambdaBody::Block(block) = lambda_body {
            let mut temp_mutated = std::collections::HashSet::new();
            self.collect_mutated_vars_in_block(block, &mut temp_mutated);
            self.mutated_vars = temp_mutated;
        }

        // Auto-invoke beforeEach hooks (from all parent describe scopes + current)
        let before_hooks = self.collect_before_each_hooks();
        for (hook_fn, hook_is_async) in &before_hooks {
            if is_async && *hook_is_async {
                self.writeln(&format!("{}().await;", hook_fn));
            } else {
                self.writeln(&format!("{}();", hook_fn));
            }
        }

        match lambda_body {
            LambdaBody::Block(block) => {
                self.generate_block_inner(block)?;
            }
            LambdaBody::Expr(expr) => {
                self.write_indent();
                self.generate_expr(expr)?;
                self.output.push_str(";\n");
            }
        }

        // Auto-invoke afterEach hooks (innermost first, then parent scopes)
        let after_hooks = self.collect_after_each_hooks();
        for (hook_fn, hook_is_async) in &after_hooks {
            if is_async && *hook_is_async {
                self.writeln(&format!("{}().await;", hook_fn));
            } else {
                self.writeln(&format!("{}();", hook_fn));
            }
        }

        self.in_test_block = was_in_test;
        self.mutated_vars = saved_mutated;
        self.dedent();
        self.writeln("}");
        Ok(())
    }

    /// Generate lifecycle hooks (beforeEach, afterEach, beforeAll, afterAll)
    /// Generates a helper function that test cases auto-invoke
    fn generate_test_lifecycle(&mut self, hook_name: &str, call: &CallExpr) -> Result<()> {
        if call.args.is_empty() {
            return Ok(());
        }

        // Generate unique fn name based on nesting depth
        let base_name = self.to_snake_case(hook_name);
        let depth = if self.test_hooks_stack.is_empty() {
            0
        } else {
            self.test_hooks_stack.last().map_or(0, |s| s.depth)
        };
        let fn_name = if depth == 0 {
            base_name
        } else {
            format!("{}_{}", base_name, depth)
        };

        let lambda_body = match &call.args[0] {
            Expr::Lambda(lambda) => &lambda.body,
            _ => return Ok(()),
        };

        // Detect if the hook body contains async calls
        let is_async = ast_lambda_body_has_async(lambda_body);

        if is_async {
            self.writeln(&format!("async fn {}() {{", fn_name));
        } else {
            self.writeln(&format!("fn {}() {{", fn_name));
        }
        self.indent();

        // BUG-001 fix: Pre-analyze mutated variables in lifecycle hook body
        let saved_mutated = std::mem::take(&mut self.mutated_vars);
        if let LambdaBody::Block(block) = lambda_body {
            let mut temp_mutated = std::collections::HashSet::new();
            self.collect_mutated_vars_in_block(block, &mut temp_mutated);
            self.mutated_vars = temp_mutated;
        }

        match lambda_body {
            LambdaBody::Block(block) => {
                self.generate_block_inner(block)?;
            }
            LambdaBody::Expr(expr) => {
                self.write_indent();
                self.generate_expr(expr)?;
                self.output.push_str(";\n");
            }
        }

        self.mutated_vars = saved_mutated;
        self.dedent();
        self.writeln("}");
        Ok(())
    }

    /// Try to generate an expect() chain: expect(x).toBe(y) -> assert_eq!(x, y)
    /// Returns Some(code) if this is an expect chain, None otherwise
    fn try_generate_expect_chain(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<Option<String>> {
        // Pattern: expect(actual).matcher(expected)
        // Object is Call(expect, [actual]) and method is the matcher name

        // Check for negated: expect(x).not.toBe(y)
        // Parsed as MethodCall { object: Member { Call(expect, [x]), "not" }, method: "toBe" }
        let (actual_expr, matcher, is_negated) = match method_call.object.as_ref() {
            // Direct: expect(x).toBe(y)
            Expr::Call(call) => {
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if name == "expect" && !call.args.is_empty() {
                        (&call.args[0], &method_call.method, false)
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            // Negated: expect(x).not.toBe(y)
            Expr::Member { object, property } if property == "not" => {
                if let Expr::Call(call) = object.as_ref() {
                    if let Expr::Identifier(name) = call.callee.as_ref() {
                        if name == "expect" && !call.args.is_empty() {
                            (&call.args[0], &method_call.method, true)
                        } else {
                            return Ok(None);
                        }
                    } else {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        // Generate actual expression into a buffer
        let actual_code = {
            let saved = std::mem::take(&mut self.output);
            self.generate_expr(actual_expr)?;
            let code = std::mem::replace(&mut self.output, saved);
            code
        };

        // Helper to generate expected arg
        let gen_expected = |this: &mut Self| -> Result<String> {
            if method_call.args.is_empty() {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    "Matcher requires 1 argument",
                    "expect(actual).toBe(expected)",
                )));
            }
            let saved = std::mem::take(&mut this.output);
            this.generate_expr(&method_call.args[0])?;
            let code = std::mem::replace(&mut this.output, saved);
            Ok(code)
        };

        let result = match matcher.as_str() {
            "toBe" | "toEqual" => {
                // B105 fix: If expected is an empty array literal [], use .is_empty() instead of assert_eq!(_, vec![])
                let expected_is_empty_array = if !method_call.args.is_empty() {
                    matches!(&method_call.args[0], Expr::ArrayLiteral(elements) if elements.is_empty())
                } else { false };
                // B111 fix: If actual is Option<T> (option_value_vars or error_binding_vars),
                // wrap expected in Some() or use is_none() for null comparisons
                let actual_is_option = if let Expr::Identifier(name) = actual_expr {
                    let sname = self.sanitize_name(name);
                    self.option_value_vars.contains(&sname) || self.error_binding_vars.contains(&sname)
                } else { false };
                let expected_is_null = if !method_call.args.is_empty() {
                    matches!(&method_call.args[0], Expr::Literal(Literal::Null))
                } else { false };
                if expected_is_empty_array {
                    if is_negated {
                        format!("assert!(!{}.is_empty())", actual_code)
                    } else {
                        format!("assert!({}.is_empty())", actual_code)
                    }
                } else if actual_is_option && expected_is_null {
                    // expect(maybe).toBe(null) → assert!(maybe.is_none())
                    if is_negated {
                        format!("assert!({}.is_some())", actual_code)
                    } else {
                        format!("assert!({}.is_none())", actual_code)
                    }
                } else if actual_is_option {
                    // expect(maybe).toBe(42) → assert_eq!(maybe, Some(42))
                    let expected = gen_expected(self)?;
                    if is_negated {
                        format!("assert_ne!({}, Some({}))", actual_code, expected)
                    } else {
                        format!("assert_eq!({}, Some({}))", actual_code, expected)
                    }
                } else {
                    let expected = gen_expected(self)?;
                    if is_negated {
                        format!("assert_ne!({}, {})", actual_code, expected)
                    } else {
                        format!("assert_eq!({}, {})", actual_code, expected)
                    }
                }
            }
            "toBeTruthy" => {
                // B101 fix: If actual_expr is an error binding var (Option<Error>) or option_value_var,
                // generate .is_some() / .is_none() instead of assert!()
                let is_option = if let Expr::Identifier(name) = actual_expr {
                    let sname = self.sanitize_name(name);
                    self.error_binding_vars.contains(&sname) || self.option_value_vars.contains(&sname)
                } else { false };
                if is_option {
                    if is_negated {
                        format!("assert!({}.is_none())", actual_code)
                    } else {
                        format!("assert!({}.is_some())", actual_code)
                    }
                } else if is_negated {
                    format!("assert!(!({}))", actual_code)
                } else {
                    format!("assert!({})", actual_code)
                }
            }
            "toBeFalsy" => {
                let is_option = if let Expr::Identifier(name) = actual_expr {
                    let sname = self.sanitize_name(name);
                    self.error_binding_vars.contains(&sname) || self.option_value_vars.contains(&sname)
                } else { false };
                if is_option {
                    if is_negated {
                        format!("assert!({}.is_some())", actual_code)
                    } else {
                        format!("assert!({}.is_none())", actual_code)
                    }
                } else if is_negated {
                    format!("assert!({})", actual_code)
                } else {
                    format!("assert!(!({}))", actual_code)
                }
            }
            "toBeGreaterThan" => {
                let expected = gen_expected(self)?;
                if is_negated {
                    format!("assert!({} <= {})", actual_code, expected)
                } else {
                    format!("assert!({} > {})", actual_code, expected)
                }
            }
            "toBeLessThan" => {
                let expected = gen_expected(self)?;
                if is_negated {
                    format!("assert!({} >= {})", actual_code, expected)
                } else {
                    format!("assert!({} < {})", actual_code, expected)
                }
            }
            "toBeGreaterThanOrEqual" => {
                let expected = gen_expected(self)?;
                if is_negated {
                    format!("assert!({} < {})", actual_code, expected)
                } else {
                    format!("assert!({} >= {})", actual_code, expected)
                }
            }
            "toBeLessThanOrEqual" => {
                let expected = gen_expected(self)?;
                if is_negated {
                    format!("assert!({} > {})", actual_code, expected)
                } else {
                    format!("assert!({} <= {})", actual_code, expected)
                }
            }
            "toContain" => {
                let expected = gen_expected(self)?;
                if is_negated {
                    format!("assert!(!{}.contains(&{}))", actual_code, expected)
                } else {
                    format!("assert!({}.contains(&{}))", actual_code, expected)
                }
            }
            "toBeNull" => {
                if is_negated {
                    format!("assert!({}.is_some())", actual_code)
                } else {
                    format!("assert!({}.is_none())", actual_code)
                }
            }
            "toThrow" => {
                if is_negated {
                    format!(
                        "assert!(std::panic::catch_unwind(|| {{ {} }}).is_ok())",
                        actual_code
                    )
                } else {
                    format!(
                        "assert!(std::panic::catch_unwind(|| {{ {} }}).is_err())",
                        actual_code
                    )
                }
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown matcher: {}", matcher),
                    "Available: toBe, toEqual, toBeTruthy, toBeFalsy, toBeGreaterThan, toBeLessThan, toContain, toBeNull, toThrow",
                )));
            }
        };

        Ok(Some(result))
    }

    // ─── End liva/test codegen ───────────────────────────────────────

    fn generate_params(
        &mut self,
        params: &[Param],
        is_method: bool,
        class: Option<&ClassDecl>,
        method_name: Option<&str>,
        method: Option<&MethodDecl>,
    ) -> Result<String> {
        let mut result = String::new();

        if is_method {
            // Use &mut self for methods that modify fields
            let is_setter = method_name.map_or(false, |name| name.starts_with("set"));
            let modifies_self = method.map_or(false, |m| self.method_modifies_self(m));

            if is_setter || modifies_self {
                result.push_str("&mut self");
            } else {
                result.push_str("&self");
            }
            if !params.is_empty() {
                result.push_str(", ");
            }
        }

        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }

            // For destructured parameters, use temporary names (_param_0, _param_1, etc.)
            // Otherwise use the actual parameter name
            let param_name = if param.is_destructuring() {
                format!("_param_{}", i)
            } else {
                self.sanitize_name(param.name().unwrap())
            };

            let type_str = if let Some(type_ref) = &param.type_ref {
                let rust_type = type_ref.to_rust_type();

                // Register parameter as class instance if its type is a known class
                if !param.is_destructuring() {
                    let type_name = match type_ref {
                        TypeRef::Simple(name) => Some(name.clone()),
                        _ => None,
                    };
                    if let Some(tname) = &type_name {
                        // Track string parameters for proper .length -> .len() translation
                        if matches!(tname.as_str(), "string" | "String") {
                            self.string_vars.insert(param_name.clone());
                        }
                        // Check if this is a class type (starts with uppercase, not primitive)
                        else if !matches!(
                            tname.as_str(),
                            "number" | "i32" | "i64" | "f64" | "bool" | "char" | "Vec" | "Option"
                        ) && tname
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                        {
                            self.class_instance_vars.insert(param_name.clone());
                            self.var_types.insert(param_name.clone(), tname.clone());
                        }
                    }
                    // Track array-typed parameters in typed_array_vars and array_vars
                    // so that forEach/map/filter generate correct lambda patterns
                    // e.g., segments: [string] → typed_array_vars["segments"] = "string"
                    if let TypeRef::Array(elem_type) = type_ref {
                        self.array_vars.insert(param_name.clone());
                        let elem_type_name = match elem_type.as_ref() {
                            TypeRef::Simple(name) => name.clone(),
                            _ => "i32".to_string(),
                        };
                        self.typed_array_vars
                            .insert(param_name.clone(), elem_type_name.clone());
                        // If element type is "string", also track for proper string handling
                        if matches!(elem_type_name.as_str(), "string" | "String") {
                            // Array of strings - tracked for forEach |s| pattern
                        }
                    }
                    // Track Optional-typed parameters so that init_is_already_optional
                    // can detect them and avoid double-wrapping in Some() at call sites
                    if matches!(type_ref, TypeRef::Optional(_)) {
                        self.option_value_vars.insert(param_name.clone());
                    }
                }

                rust_type
            } else if let Some(cls) = class {
                // Try to infer from field types in the class
                // For destructured params, use the type annotation if present
                if param.is_destructuring() {
                    "serde_json::Value".to_string() // Default for destructured params without type
                } else {
                    self.infer_param_type_from_class(param.name().unwrap(), cls, method_name)
                        .unwrap_or_else(|| "i32".to_string())
                }
            } else {
                // Infer type based on parameter name (hack for constructor)
                if param.is_destructuring() {
                    "serde_json::Value".to_string() // Default for destructured params without type
                } else {
                    match param.name().unwrap() {
                        "name" => "String".to_string(),
                        "age" => "i32".to_string(),
                        "items" => "Vec<serde_json::Value>".to_string(),
                        _ => "i32".to_string(),
                    }
                }
            };

            write!(result, "{}: {}", param_name, type_str).unwrap();
        }

        Ok(result)
    }

    fn generate_block_inner(&mut self, block: &BlockStmt) -> Result<()> {
        for stmt in &block.stmts {
            self.generate_stmt(stmt)?;
        }
        Ok(())
    }

    fn generate_if_body(&mut self, body: &IfBody) -> Result<()> {
        match body {
            IfBody::Block(block) => {
                for stmt in &block.stmts {
                    self.generate_stmt(stmt)?;
                }
            }
            IfBody::Stmt(stmt) => {
                self.generate_stmt(stmt)?;
            }
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        // Phase 4: Check if this statement uses multiple pending tasks (join combining optimization)
        let used_tasks = self.stmt_uses_pending_tasks(stmt);

        if used_tasks.len() > 1 {
            // Multiple tasks used - generate tokio::join! for parallel await
            self.generate_tasks_join(&used_tasks)?;
        } else if used_tasks.len() == 1 {
            // Single task - use regular await (Phase 2 behavior)
            self.generate_task_await(&used_tasks[0])?;
        }
        // Phase 2 fallback: Check if this statement uses a pending task for the first time
        // (This is kept for backwards compatibility, but should not trigger if Phase 4 works)
        else if let Some(var_name) = self.stmt_uses_pending_task(stmt) {
            self.generate_task_await(&var_name)?;
        }

        match stmt {
            Stmt::VarDecl(var) => {
                self.write_indent();

                // Handle `or fail "message"` — error propagation shorthand (v1.1.0)
                if let Some(fail_msg) = &var.or_fail_msg {
                    // Check if this is bare `or fail` (no message) — Bug #95 fix
                    // Bare `or fail` propagates the original error unchanged.
                    let is_bare_or_fail = matches!(
                        fail_msg.as_ref(),
                        Expr::Literal(crate::ast::Literal::String(s)) if s.is_empty()
                    );

                    // let x = fallible_expr or fail "message"
                    // Generates: let x = match fallible_expr { Ok(v) => v, Err(e) => return Err(liva_rt::Error::chain("message", fn, loc, e)) };
                    let binding = &var.bindings[0];
                    let var_name = self.sanitize_name(binding.name().unwrap());

                    let fn_name = self.current_function_name.clone();
                    let filename = self.source_filename.clone();
                    let location = if var.or_fail_line > 0 {
                        format!("{}:{}", filename, var.or_fail_line)
                    } else {
                        filename.clone()
                    };

                    // Check if the init is an HTTP call, File call, or user-fallible call
                    let is_http = self.is_http_call(&var.init);
                    let is_file = self.is_file_call(&var.init);
                    let is_json_parse = self.is_json_parse_call(&var.init);
                    let is_user_fallible = self.is_fallible_expr(&var.init);

                    if is_http {
                        // HTTP calls return (Option<Response>, String)
                        write!(self.output, "let {} = {{ let (opt, err_str) = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        write!(self.output,
                            ".await; if !err_str.is_empty() {{ return Err(liva_rt::Error::new(",
                        ).unwrap();
                        if is_bare_or_fail {
                            self.output.push_str("err_str");
                        } else {
                            self.generate_expr(fail_msg)?;
                        }
                        write!(self.output, ", \"{}\", \"{}\")); }} opt.unwrap_or_default() }};\n", fn_name, location).unwrap();

                        // Track as rust_struct for member access
                        self.rust_struct_vars.insert(var_name);
                    } else if is_file {
                        // File calls return (Option<T>, String)
                        write!(self.output, "let {} = {{ let (opt, err_str) = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        write!(self.output,
                            "; if !err_str.is_empty() {{ return Err(liva_rt::Error::new(",
                        ).unwrap();
                        if is_bare_or_fail {
                            self.output.push_str("err_str");
                        } else {
                            self.generate_expr(fail_msg)?;
                        }
                        write!(self.output, ", \"{}\", \"{}\")); }} opt.unwrap_or_default() }};\n", fn_name, location).unwrap();
                    } else if is_json_parse {
                        // JSON.parse returns (Option<JsonValue>, String)
                        write!(self.output, "let {} = {{ let (opt, err_str) = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        write!(self.output,
                            "; if !err_str.is_empty() {{ return Err(liva_rt::Error::new(",
                        ).unwrap();
                        if is_bare_or_fail {
                            self.output.push_str("err_str");
                        } else {
                            self.generate_expr(fail_msg)?;
                        }
                        write!(self.output, ", \"{}\", \"{}\")); }} opt.unwrap_or_default() }};\n", fn_name, location).unwrap();

                        // Track as json_value_var for indexed access
                        self.json_value_vars.insert(var_name);
                    } else if is_user_fallible {
                        // User-defined fallible functions return Result<T, Error>
                        write!(self.output, "let {} = match ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        if is_bare_or_fail {
                            // Bare `or fail` — propagate original error unchanged
                            self.output.push_str(" { Ok(v) => v, Err(e) => return Err(e) };\n");
                        } else {
                            // Chain: Err(e) => return Err(Error::chain("msg", fn, loc, e))
                            self.output
                                .push_str(" { Ok(v) => v, Err(e) => return Err(liva_rt::Error::chain(");
                            self.generate_expr(fail_msg)?;
                            write!(self.output, ", \"{}\", \"{}\", e)) }};\n", fn_name, location).unwrap();
                        }
                    } else if self.is_option_returning_method(&var.init) {
                        // Option-returning methods (find, first, last, min, max) with or fail
                        self.suppress_option_unwrap = true;
                        write!(self.output, "let {} = match ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.suppress_option_unwrap = false;
                        self.output.push_str(" { Some(v) => v, None => panic!(\"or fail: {}\", ");
                        self.generate_expr(fail_msg)?;
                        self.output.push_str(") };\n");
                    } else {
                        // Non-fallible expression with or fail — just assign directly (or fail never triggers)
                        write!(self.output, "let {} = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");
                    }

                    return Ok(());
                }

                // Handle `or <value>` — default value on error (like JS `||`)
                if let Some(default_val) = &var.or_value {
                    let binding = &var.bindings[0];
                    let var_name = self.sanitize_name(binding.name().unwrap());

                    let is_http = self.is_http_call(&var.init);
                    let is_file = self.is_file_call(&var.init);
                    let is_json_parse = self.is_json_parse_call(&var.init);
                    let is_user_fallible = self.is_fallible_expr(&var.init);

                    if is_http {
                        // HTTP calls: let var = { let (opt, err_str) = HTTP.get(...).await; if !err_str.is_empty() { default } else { opt.unwrap_or_default() } };
                        write!(self.output, "let {} = {{ let (opt, err_str) = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(".await; if !err_str.is_empty() { ");
                        self.generate_expr(default_val)?;
                        self.output.push_str(" } else { opt.unwrap_or_default() } };\n");
                        self.rust_struct_vars.insert(var_name);
                    } else if is_file {
                        write!(self.output, "let {} = {{ let (opt, err_str) = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str("; if !err_str.is_empty() { ");
                        self.generate_expr(default_val)?;
                        // B113 fix: String literal defaults need .to_string() to match String type of success arm
                        if matches!(default_val.as_ref(), Expr::Literal(Literal::String(_))) {
                            self.output.push_str(".to_string()");
                        }
                        self.output.push_str(" } else { opt.unwrap_or_default() } };\n");
                    } else if is_json_parse {
                        write!(self.output, "let {} = {{ let (opt, err_str) = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str("; if !err_str.is_empty() { ");
                        self.generate_expr(default_val)?;
                        self.output.push_str(" } else { opt.unwrap_or_default() } };\n");
                        self.json_value_vars.insert(var_name);
                    } else if self.is_map_get_call(&var.init) {
                        // Map.get with or default: let var = map.get(&key).cloned().unwrap_or(default);
                        self.suppress_map_get_unwrap = true;
                        write!(self.output, "let {} = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.suppress_map_get_unwrap = false;
                        self.output.push_str(".unwrap_or(");
                        // String literals need .to_string() for HashMap<String, String>
                        if matches!(default_val.as_ref(), Expr::Literal(Literal::String(_))) {
                            self.generate_expr(default_val)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(default_val)?;
                        }
                        self.output.push_str(");\n");
                    } else if is_user_fallible {
                        // User-defined fallible: let var = match expr { Ok(v) => v, Err(_) => defaultValue };
                        write!(self.output, "let {} = match ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(" { Ok(v) => v, Err(_) => ");
                        self.generate_expr(default_val)?;
                        // Bug #77 fix: String literals need .to_string() to match Ok(String) arm
                        if matches!(default_val.as_ref(), Expr::Literal(Literal::String(_))) {
                            self.output.push_str(".to_string()");
                        }
                        self.output.push_str(" };\n");
                    } else if self.is_option_returning_method(&var.init) {
                        // Option-returning methods (find, first, last, min, max) with or default
                        self.suppress_option_unwrap = true;
                        write!(self.output, "let {} = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.suppress_option_unwrap = false;
                        self.output.push_str(".unwrap_or(");
                        self.generate_expr(default_val)?;
                        self.output.push_str(");\n");
                    } else if let Expr::Call(call) = &var.init {
                        // B16 fix: parseInt/parseFloat with or default
                        // Generate: let var = match arg.parse::<T>() { Ok(v) => v, Err(_) => default };
                        if let Expr::Identifier(name) = call.callee.as_ref() {
                            if name == "parseInt" && !call.args.is_empty() {
                                write!(self.output, "let {} = match ", var_name).unwrap();
                                self.generate_expr(&call.args[0])?;
                                self.output.push_str(".parse::<i32>() { Ok(v) => v, Err(_) => ");
                                self.generate_expr(default_val)?;
                                self.output.push_str(" };\n");
                            } else if name == "parseFloat" && !call.args.is_empty() {
                                write!(self.output, "let {} = match ", var_name).unwrap();
                                self.generate_expr(&call.args[0])?;
                                self.output.push_str(".parse::<f64>() { Ok(v) => v, Err(_) => ");
                                self.generate_expr(default_val)?;
                                self.output.push_str(" };\n");
                            } else {
                                // BUG-006: Generic function call with `or` — unwrap Option
                                write!(self.output, "let {} = ", var_name).unwrap();
                                self.generate_expr(&var.init)?;
                                self.output.push_str(".unwrap_or(");
                                if matches!(default_val.as_ref(), Expr::Literal(Literal::String(_))) {
                                    self.generate_expr(default_val)?;
                                    self.output.push_str(".to_string()");
                                } else {
                                    self.generate_expr(default_val)?;
                                }
                                self.output.push_str(");\n");
                            }
                        } else {
                            // BUG-006: Method call with `or` — unwrap Option
                            write!(self.output, "let {} = ", var_name).unwrap();
                            self.generate_expr(&var.init)?;
                            self.output.push_str(".unwrap_or(");
                            if matches!(default_val.as_ref(), Expr::Literal(Literal::String(_))) {
                                self.generate_expr(default_val)?;
                                self.output.push_str(".to_string()");
                            } else {
                                self.generate_expr(default_val)?;
                            }
                            self.output.push_str(");\n");
                        }
                    } else {
                        // BUG-006: Generic fallback — use .unwrap_or / .unwrap_or_else for
                        // functions that return Option<T> (user-defined or otherwise)
                        write!(self.output, "let {} = ", var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(".unwrap_or(");
                        if matches!(default_val.as_ref(), Expr::Literal(Literal::String(_))) {
                            self.generate_expr(default_val)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(default_val)?;
                        }
                        self.output.push_str(");\n");
                    }

                    return Ok(());
                }

                // Phase 2: Check if init expression is a Task (async/par call)
                let task_exec_policy = self.is_task_expr(&var.init);

                if var.bindings.len() > 1 {
                    // Multiple bindings - fallible pattern (error binding)
                    let is_fallible_call = self.is_fallible_expr(&var.init);

                    // Collect binding names for tracking
                    let binding_names: Vec<String> = var
                        .bindings
                        .iter()
                        .filter_map(|b| b.name().map(|n| self.sanitize_name(n)))
                        .collect();

                    // Check if the init expression returns a tuple directly (before tracking)
                    let returns_tuple = self.is_builtin_conversion_call(&var.init);

                    // Check if this is an await of HTTP task (also returns tuple with String error)
                    let is_await_http = self.is_await_http_task(&var.init).is_some();

                    // Phase 1: Check if this is typed JSON.parse (returns direct values, not Option)
                    let is_typed_json_parse = self.is_json_parse_call(&var.init)
                        && var
                            .bindings
                            .first()
                            .and_then(|b| b.type_ref.as_ref())
                            .is_some();

                    // Phase 3: Track the error variable (second binding) as Option<String>
                    // BUT: Only track as Option if NOT a tuple-returning function AND NOT typed JSON.parse
                    // AND NOT an await of HTTP task (which returns String error, not Option)
                    // AND NOT a task/concurrency call (match unwrap yields direct values, not Option)
                    // AND NOT a user-defined fallible call (match Ok(v)/Err(e) yields direct T, not Option)
                    // Tuple functions return (Option<T>, String) - err is String, not Option
                    // Typed JSON.parse returns (T, String) - value is T, not Option<T>
                    let is_task = task_exec_policy.is_some();
                    let is_await_task = self.is_await_of_pending_task(&var.init).is_some();
                    let is_file_or_config = self.is_file_call(&var.init);
                    if binding_names.len() == 2
                        && !returns_tuple
                        && !is_typed_json_parse
                        && !is_await_http
                        && !is_task
                        && !is_await_task
                        && !is_fallible_call
                        && !is_file_or_config
                    {
                        self.error_binding_vars.insert(binding_names[1].clone());
                        if let Some(scope) = self.error_binding_scope_stack.last_mut() {
                            scope.push(binding_names[1].clone());
                        }
                        self.option_value_vars.insert(binding_names[0].clone());
                    // Also track the value (first binding)
                    } else if binding_names.len() == 2 && is_fallible_call {
                        // Bug #80 fix: User fallible calls: match { Ok(v) => (v, None), Err(e) => (Default, Some(e)) }
                        // value is T (direct), err is Option<Error>
                        self.error_binding_vars.insert(binding_names[1].clone());
                        if let Some(scope) = self.error_binding_scope_stack.last_mut() {
                            scope.push(binding_names[1].clone());
                        }
                        // Do NOT add to option_value_vars - value is direct T, not Option<T>
                    } else if binding_names.len() == 2 && (is_task || is_await_task) {
                        // Task error bindings: match { Ok(v) => (v, None), Err(e) => (Default, Some(e)) }
                        // value is T (direct), err is Option<Error>
                        // B03 fix: HTTP tasks return String error, not Option<Error>
                        let inner_is_http = self.is_http_call(&var.init);
                        if inner_is_http {
                            // HTTP tasks: err is String (from tuple), not Option
                            self.string_error_vars.insert(binding_names[1].clone());
                            self.rust_struct_vars.insert(binding_names[0].clone());
                        } else {
                            self.error_binding_vars.insert(binding_names[1].clone());
                            if let Some(scope) = self.error_binding_scope_stack.last_mut() {
                                scope.push(binding_names[1].clone());
                            }
                        }
                        // Do NOT add to option_value_vars - value is direct, not Option
                    } else if binding_names.len() == 2 && (returns_tuple || is_typed_json_parse) {
                        // B102 fix: parseInt/parseFloat now return (T, Option<Error>)
                        // Track their error binding as error_binding_vars for consistent handling
                        let is_parse_int_float = matches!(&var.init, Expr::Call(call) if matches!(call.callee.as_ref(), Expr::Identifier(n) if n == "parseInt" || n == "parseFloat"));
                        if is_parse_int_float {
                            self.error_binding_vars.insert(binding_names[1].clone());
                            if let Some(scope) = self.error_binding_scope_stack.last_mut() {
                                scope.push(binding_names[1].clone());
                            }
                        } else {
                            // For other tuple-returning functions AND typed JSON.parse: err is String
                            self.string_error_vars.insert(binding_names[1].clone());
                        }

                        // Check if this is an HTTP call - mark first binding as rust_struct
                        if self.is_http_call(&var.init) {
                            self.rust_struct_vars.insert(binding_names[0].clone());
                        }

                        // Check if this is typed JSON.parse with array type - track element type
                        if is_typed_json_parse {
                            if let Some(first_binding) = var.bindings.first() {
                                if let Some(type_ref) = &first_binding.type_ref {
                                    // Check if it's an array type like [Post] or [string]
                                    if let TypeRef::Array(element_type) = type_ref {
                                        match element_type.as_ref() {
                                            TypeRef::Simple(type_name) => {
                                                // Track arrays of classes and strings (both need .cloned())
                                                self.typed_array_vars.insert(
                                                    binding_names[0].clone(),
                                                    type_name.clone(),
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                    // Also track the data variable as an array for .length -> .len() conversion
                                    if matches!(type_ref, TypeRef::Array(_)) {
                                        self.array_vars.insert(binding_names[0].clone());
                                    }
                                }
                            }
                        }
                    }

                    if let Some(exec_policy) = task_exec_policy {
                        // Phase 2: Error binding with Task - store task without awaiting
                        // Generate: let task_name = async/par call();
                        let task_var_name = format!("{}_task", binding_names[0]);
                        write!(self.output, "let {} = ", task_var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");

                        // Register as pending task with error binding
                        let is_http = self.is_http_call(&var.init);

                        self.pending_tasks.insert(
                            binding_names[0].clone(),
                            TaskInfo {
                                is_error_binding: true,
                                binding_names: binding_names.clone(),
                                awaited: false,
                                exec_policy,
                                returns_tuple,
                                is_http_call: is_http,
                            },
                        );
                    } else {
                        // Non-Task error binding (original behavior)
                        write!(self.output, "let (").unwrap();
                        for (i, binding) in var.bindings.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            if let Some(name) = binding.name() {
                                // B34 fix: Mark error binding vars as mut when reassigned
                                let sanitized = self.sanitize_name(name);
                                if self.mutated_vars.contains(&sanitized) {
                                    write!(self.output, "mut {}", sanitized).unwrap();
                                } else {
                                    write!(self.output, "{}", sanitized).unwrap();
                                }
                            }
                        }

                        if is_fallible_call {
                            // Generate: let (value, err) = match expr { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };
                            self.output.push_str(") = match ");
                            self.generate_expr(&var.init)?;
                            self.output.push_str(" { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n");
                        } else {
                            // Check if the expression is a built-in conversion function that returns a tuple
                            let returns_tuple = self.is_builtin_conversion_call(&var.init);

                            // Phase 1: Check if this is JSON.parse with type hint
                            let is_json_parse = self.is_json_parse_call(&var.init);
                            let has_type_hint = var
                                .bindings
                                .first()
                                .and_then(|b| b.type_ref.as_ref())
                                .is_some();

                            if is_json_parse && has_type_hint {
                                // Typed JSON parsing with error binding: let nums: [i32], err = JSON.parse("[1,2,3]")
                                // Generate: let (nums, err): (Vec<i32>, String) = match serde_json::from_str::<Vec<i32>>(...) { Ok(v) => (v, String::new()), Err(e) => (Vec::new(), format!("{}", e)) };
                                let type_ref =
                                    var.bindings.first().unwrap().type_ref.as_ref().unwrap();
                                let rust_type = self.expand_type_alias(type_ref);

                                write!(self.output, "): ({}, String) = match ", rust_type).unwrap();

                                if let Expr::MethodCall(method_call) = &var.init {
                                    self.generate_typed_json_parse(method_call, type_ref)?;
                                }

                                // Generate default value for error case
                                let default_value = match type_ref {
                                    TypeRef::Array(_) => "Vec::new()".to_string(),
                                    TypeRef::Optional(_) => "None".to_string(),
                                    TypeRef::Simple(name) => match name.as_str() {
                                        "int" | "i8" | "i16" | "i32" | "i64" | "i128" | "u8"
                                        | "u16" | "u32" | "u64" | "u128" | "usize" | "isize" => {
                                            "0".to_string()
                                        }
                                        "float" | "f32" | "f64" => "0.0".to_string(),
                                        "bool" => "false".to_string(),
                                        "string" | "String" => "String::new()".to_string(),
                                        _ => "Default::default()".to_string(),
                                    },
                                    _ => "Default::default()".to_string(),
                                };

                                write!(self.output, " {{ Ok(v) => (v, String::new()), Err(e) => ({}, format!(\"JSON parse error: {{}}\", e)) }};\n", default_value).unwrap();

                                // Phase 2.2: Track class instances for proper member access codegen
                                if let TypeRef::Simple(class_name) = type_ref {
                                    // Check if this is a class type (not a primitive)
                                    if self.class_fields.contains_key(class_name) {
                                        let binding = &var.bindings[0];
                                        if let Some(name) = binding.name() {
                                            self.class_instance_vars
                                                .insert(self.sanitize_name(name));
                                        }
                                    }
                                } else if let TypeRef::Array(elem_type) = type_ref {
                                    // Track array variable and element type
                                    let binding = &var.bindings[0];
                                    if let Some(name) = binding.name() {
                                        let sanitized_name = self.sanitize_name(name);
                                        self.array_vars.insert(sanitized_name.clone());

                                        // Track element type (class, string, etc.) for proper forEach/map patterns
                                        if let TypeRef::Simple(type_name) = elem_type.as_ref() {
                                            // Track in typed_array_vars for both classes and primitives like "string"
                                            // This ensures forEach uses correct lambda pattern |p| instead of |&p|
                                            self.typed_array_vars
                                                .insert(sanitized_name.clone(), type_name.clone());

                                            // Also track class instances for member access
                                            if self.class_fields.contains_key(type_name) {
                                                // Already handled by typed_array_vars
                                            }
                                        }
                                    }
                                }
                            } else if is_json_parse && !has_type_hint {
                                // Untyped JSON parsing with error binding: let data, err = JSON.parse(body)
                                // JSON.parse returns (Option<JsonValue>, String)
                                // Generate: let (data, err) = expr;
                                self.output.push_str(") = ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(";\n");

                                // Track this variable as Option<JsonValue> so we can unwrap it before field access
                                if let Some(first_binding) = var.bindings.first() {
                                    if let Some(name) = first_binding.name() {
                                        let sanitized = self.sanitize_name(name);
                                        self.option_value_vars.insert(sanitized.clone());
                                        self.json_value_vars.insert(sanitized);
                                    }
                                }
                            } else if self.is_http_call(&var.init) {
                                // HTTP call without async - generate direct .await
                                // let response, err = HTTP.get(url)
                                // Generate: let (response, err) = { let (opt, err) = liva_http_get(url).await; (opt.unwrap_or_default(), err) };
                                self.output.push_str(") = { let (opt, err) = ");
                                self.generate_expr(&var.init)?;
                                self.output
                                    .push_str(".await; (opt.unwrap_or_default(), err) };\n");

                                // Track the response variable as rust_struct
                                if let Some(first_binding) = var.bindings.first() {
                                    if let Some(name) = first_binding.name() {
                                        self.rust_struct_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Track the error variable as string_error_vars (for `if err` sugar)
                                if var.bindings.len() >= 2 {
                                    if let Some(name) = var.bindings[1].name() {
                                        self.string_error_vars.insert(self.sanitize_name(name));
                                    }
                                }
                            } else if self.is_file_call(&var.init) {
                                // File/Dir/Config/Regex/Date call - returns (Option<T>, String)
                                // let content, err = File.read(path)
                                // Generate: let (content, err) = { let (opt, err) = expr; (opt.unwrap_or_default(), err) };
                                // Special case for Date.parse: NaiveDateTime has no Default, use epoch
                                let is_date_parse = if let Expr::MethodCall(mc) = &var.init {
                                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                                        obj == "Date" && mc.method == "parse"
                                    } else { false }
                                } else { false };

                                let is_db_open = if let Expr::MethodCall(mc) = &var.init {
                                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                                        obj == "DB" && mc.method == "open"
                                    } else { false }
                                } else { false };

                                if is_db_open {
                                    // DB.open returns (Option<Connection>, String) — Connection has no Default
                                    // Use in-memory fallback so the variable is always valid (check err to detect failure)
                                    // Bug #81 fix: Wrap in Arc<Mutex<>> so DB connection can be shared across async handlers
                                    self.output.push_str(") = { let (opt, err) = ");
                                    self.generate_expr(&var.init)?;
                                    self.output.push_str("; (std::sync::Arc::new(std::sync::Mutex::new(opt.unwrap_or_else(|| rusqlite::Connection::open_in_memory().unwrap()))), err) };\n");
                                } else if is_date_parse {
                                    self.output.push_str(") = { let (opt, err) = ");
                                    self.generate_expr(&var.init)?;
                                    self.output.push_str("; (opt.unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()), err) };\n");
                                } else {
                                    self.output.push_str(") = { let (opt, err) = ");
                                    self.generate_expr(&var.init)?;
                                    self.output.push_str("; (opt.unwrap_or_default(), err) };\n");
                                }
                                // Track the error variable as string_error_vars (for `if err` sugar)
                                if var.bindings.len() >= 2 {
                                    if let Some(name) = var.bindings[1].name() {
                                        self.string_error_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Track Dir.list()/Dir.listRecursive()/Dir.walk()/File.readLines() result as array of strings for proper par_iter/map lambda patterns
                                if let Expr::MethodCall(mc) = &var.init {
                                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                                        if (obj == "Dir" && matches!(mc.method.as_str(), "list" | "listRecursive" | "walk"))
                                            || (obj == "File" && mc.method == "readLines")
                                        {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.array_vars.insert(sanitized.clone());
                                                    self.native_vec_string_vars.insert(sanitized.clone());
                                                    self.typed_array_vars.insert(sanitized, "string".to_string());
                                                }
                                            }
                                        }
                                        // Track Config.load() result as map variable
                                        if obj == "Config" && mc.method == "load" {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.map_vars.insert(sanitized);
                                                }
                                            }
                                        }
                                        // Track Date.parse() result as date variable
                                        if obj == "Date" && mc.method == "parse" {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.date_vars.insert(sanitized);
                                                }
                                            }
                                        }
                                        // Track CSV.read() result as array variable (Vec<Vec<String>>)
                                        if obj == "CSV" && mc.method == "read" {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.array_vars.insert(sanitized);
                                                }
                                            }
                                        }
                                        // Track CSV.readTable() result as array variable (Vec<HashMap<String,String>>)
                                        if obj == "CSV" && mc.method == "readTable" {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.array_vars.insert(sanitized);
                                                }
                                            }
                                        }
                                        // Track DB.open() result as db connection variable
                                        if obj == "DB" && mc.method == "open" {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.db_vars.insert(sanitized);
                                                }
                                            }
                                        }
                                        // Track DB.query() result as array variable (Vec<HashMap<String,String>>)
                                        if obj == "DB" && mc.method == "query" {
                                            if let Some(first_binding) = var.bindings.first() {
                                                if let Some(name) = first_binding.name() {
                                                    let sanitized = self.sanitize_name(name);
                                                    self.array_vars.insert(sanitized.clone());
                                                    self.map_array_vars.insert(sanitized);
                                                }
                                            }
                                        }
                                    }
                                }
                            } else if let Some(task_name) = self.is_await_http_task(&var.init) {
                                // Await of pending HTTP task - unwrap JoinHandle and extract result
                                // let res, err = await task1
                                // Generate: let (res, err) = { let (opt, err) = task1_task.await.unwrap(); (opt.unwrap_or_default(), err) };
                                write!(self.output, ") = {{ let (opt, err) = {}_task.await.unwrap(); (opt.unwrap_or_default(), err) }};\n", task_name).unwrap();

                                // Track the response variable as rust_struct
                                if let Some(first_binding) = var.bindings.first() {
                                    if let Some(name) = first_binding.name() {
                                        self.rust_struct_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Track the error variable as string_error_vars (for `if err` sugar)
                                if var.bindings.len() >= 2 {
                                    if let Some(name) = var.bindings[1].name() {
                                        self.string_error_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Mark task as awaited
                                if let Some(task_info) = self.pending_tasks.get_mut(&task_name) {
                                    task_info.awaited = true;
                                }
                            } else if returns_tuple {
                                // Built-in conversion functions (parseInt, parseFloat) already return (value, Option<Error>)
                                // Generate: let (value, err) = expr;
                                self.output.push_str(") = ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(";\n");
                            } else if self.is_await_of_pending_task(&var.init).is_some() {
                                // Explicit await of a pending task with error binding
                                // let result, err = await calcTask
                                // The task function is fallible (returns Result<T, Error>)
                                // Generate: let (result, err) = match calcTask_task.await.unwrap() { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };
                                let task_name = self.is_await_of_pending_task(&var.init).unwrap();
                                self.output.push_str(") = match ");
                                write!(self.output, "{}_task.await.unwrap()", task_name).unwrap();
                                self.output.push_str(" { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n");
                                // Mark task as awaited
                                if let Some(task_info) = self.pending_tasks.get_mut(&task_name) {
                                    task_info.awaited = true;
                                }
                            } else {
                                // Non-fallible function called with fallible binding pattern
                                // Generate: let (value, err) = (expr, None);
                                self.output.push_str("): (_, Option<liva_rt::Error>) = (");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(", None);\n");
                            }
                        }
                    }
                } else {
                    // Normal binding: let a = expr (only one binding expected)
                    if var.bindings.len() != 1 {
                        return Err(CompilerError::CodegenError(
                            SemanticErrorInfo::new(
                                "E3000",
                                "Invalid binding pattern",
                                "Let statement should have exactly one binding when not using fallible pattern"
                            )
                            .with_help("Use fallible binding pattern 'let result, err = ...' or single binding 'let result = ...'")
                        ));
                    }
                    let binding = &var.bindings[0];

                    // Check if this is a destructuring pattern
                    if !binding.pattern.is_simple() {
                        // Handle destructuring patterns
                        self.generate_destructuring_pattern(&binding.pattern, &var.init)?;
                        return Ok(());
                    }

                    // Simple identifier binding
                    let var_name = self.sanitize_name(binding.name().unwrap());

                    // Phase 2: Check if this is a Task assignment
                    if let Some(exec_policy) = task_exec_policy {
                        // Simple task binding (no error handling)
                        // Generate: let var_name_task = async/par call();
                        let task_var_name = format!("{}_task", var_name);
                        write!(self.output, "let {} = ", task_var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");

                        // Check if the inner call is HTTP (for await unwrapping later)
                        let is_http = self.is_http_call_in_async(&var.init);

                        // Register as pending task
                        self.pending_tasks.insert(
                            var_name.clone(),
                            TaskInfo {
                                is_error_binding: false,
                                binding_names: vec![var_name.clone()],
                                awaited: false,
                                exec_policy,
                                returns_tuple: false, // Simple binding, no tuple destructuring
                                is_http_call: is_http,
                            },
                        );
                    } else {
                        // Non-Task normal binding (original behavior)

                        // B33 fix: Single-var binding for fallible/builtin-conversion calls
                        // e.g., let writeErr = File.write(path, content)
                        // Should extract only the error string (.1), not the raw tuple
                        // Exclude JSON.parse/stringify and parseInt/parseFloat which have their own handlers
                        let is_parse_int_float = matches!(&var.init, Expr::Call(call) if matches!(call.callee.as_ref(), Expr::Identifier(n) if n == "parseInt" || n == "parseFloat"));
                        let is_single_builtin_tuple = self.is_builtin_conversion_call(&var.init)
                            && !self.is_json_parse_call(&var.init)
                            && !self.is_json_stringify_call(&var.init)
                            && !is_parse_int_float;
                        let is_single_fallible = self.is_fallible_expr(&var.init);
                        if is_single_builtin_tuple || is_single_fallible {
                            // Track as string error var for `if err` sugar
                            self.string_error_vars.insert(var_name.clone());
                            self.string_vars.insert(var_name.clone());

                            let needs_mut = self.mutated_vars.contains(&var_name);
                            if needs_mut {
                                write!(self.output, "let mut {}", var_name).unwrap();
                            } else {
                                write!(self.output, "let {}", var_name).unwrap();
                            }
                            self.output.push_str(" = ");

                            if is_single_builtin_tuple {
                                // Builtin tuple-returning calls: extract error string with .1
                                self.generate_expr(&var.init)?;
                                self.output.push_str(".1");
                            } else {
                                // User-defined fallible calls: match to extract error
                                self.output.push_str("match ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(" { Ok(_) => String::new(), Err(e) => e.to_string() }");
                            }
                            self.output.push_str(";\n");
                            return Ok(());
                        }

                        // Check if initializing with a string literal or string expression - mark variable as string
                        if let Expr::Literal(Literal::String(_)) = &var.init {
                            if let Some(name) = binding.name() {
                                self.string_vars.insert(self.sanitize_name(name));
                            }
                        }
                        // Bug #84 fix: Track variables assigned from req.body as string_vars
                        // so they get .clone() when passed to functions
                        else if let Expr::Member { object: member_obj, property: member_prop } = &var.init {
                            if let Some(ref req_param_name) = self.server_request_param {
                                if let Expr::Identifier(name) = member_obj.as_ref() {
                                    if name == req_param_name && member_prop == "body" {
                                        if let Some(name) = binding.name() {
                                            self.string_vars.insert(self.sanitize_name(name));
                                        }
                                    }
                                }
                            }
                        }
                        // B32: Track float variables for mixed-type arithmetic detection
                        if let Expr::Literal(Literal::Float(_)) = &var.init {
                            if let Some(name) = binding.name() {
                                self.float_vars.insert(self.sanitize_name(name));
                            }
                        }
                        // Also mark as float if type annotation says float/f64/f32
                        if binding.type_ref.as_ref().map_or(false, |t| {
                            matches!(t, TypeRef::Simple(name) if name == "float" || name == "f64" || name == "f32")
                        }) {
                            if let Some(name) = binding.name() {
                                self.float_vars.insert(self.sanitize_name(name));
                            }
                        }
                        // Also mark as string if initialized with a string concatenation
                        else if self.expr_is_stringy(&var.init) {
                            if let Some(name) = binding.name() {
                                self.string_vars.insert(self.sanitize_name(name));
                            }
                        }

                        // Check if initializing with an object literal - mark variable for bracket notation
                        if let Expr::ObjectLiteral(_) = &var.init {
                            if let Some(name) = binding.name() {
                                self.bracket_notation_vars.insert(name.to_string());
                            }
                        }

                        // Check if initializing with an array literal - mark variable as array
                        if let Expr::ArrayLiteral(elements) = &var.init {
                            if let Some(name) = binding.name() {
                                self.array_vars.insert(name.to_string());

                                // If array contains anonymous objects, mark as json_value
                                // e.g., let users = [{ id: 1, name: "Alice" }, { id: 2, name: "Bob" }]
                                if !elements.is_empty() {
                                    if matches!(elements[0], Expr::ObjectLiteral(_)) {
                                        self.json_value_vars.insert(name.to_string());
                                    }
                                    // If array contains class instances, track the element type
                                    // e.g., let items = [Item("one", true), Item("two", false)]
                                    else if let Expr::Call(call) = &elements[0] {
                                        if let Expr::Identifier(class_name) = &*call.callee {
                                            // Check if first letter is uppercase (likely a class)
                                            if class_name
                                                .chars()
                                                .next()
                                                .map(|c| c.is_uppercase())
                                                .unwrap_or(false)
                                            {
                                                self.typed_array_vars
                                                    .insert(name.to_string(), class_name.clone());
                                            }
                                        }
                                    }
                                    // Bug #50 fix: Track primitive type arrays for proper filter/map lambda patterns
                                    // e.g., let nums = [1, 2, 3] -> typed as "i32" (Copy type)
                                    else if let Expr::Literal(lit) = &elements[0] {
                                        let elem_type = match lit {
                                            Literal::Int(_) => Some("i32"),
                                            Literal::Float(_) => Some("f64"),
                                            Literal::Bool(_) => Some("bool"),
                                            Literal::String(_) => Some("string"),
                                            _ => None,
                                        };
                                        if let Some(type_name) = elem_type {
                                            self.typed_array_vars
                                                .insert(name.to_string(), type_name.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        // Check if initializing with a map literal — mark variable as map
                        // Bug #75 fix: Use sanitized name so camelCase vars (e.g., usedTags -> used_tags) match lookups
                        if let Expr::MapLiteral(_) = &var.init {
                            if let Some(name) = binding.name() {
                                self.map_vars.insert(self.sanitize_name(&name));
                            }
                        }
                        // Check if initializing with a set literal — mark variable as set
                        if let Expr::SetLiteral(_) = &var.init {
                            if let Some(name) = binding.name() {
                                self.set_vars.insert(self.sanitize_name(&name));
                            }
                        }
                        // Also track map vars from type annotation Map<K,V>
                        if binding.type_ref.as_ref().map_or(false, |t| matches!(t, TypeRef::Map(_, _))) {
                            if let Some(name) = binding.name() {
                                self.map_vars.insert(self.sanitize_name(&name));
                            }
                        }
                        // Also track set vars from type annotation Set<T>
                        if binding.type_ref.as_ref().map_or(false, |t| matches!(t, TypeRef::Set(_))) {
                            if let Some(name) = binding.name() {
                                self.set_vars.insert(self.sanitize_name(&name));
                            }
                        }
                        // Check if initializing with a method call that returns an array (map, filter, etc.)
                        // or Option (find)
                        else if let Expr::MethodCall(method_call) = &var.init {
                            if matches!(method_call.method.as_str(), "map" | "filter" | "split"
                                | "sort" | "sortBy" | "reversed" | "distinct" | "flat" | "flatten"
                                | "take" | "drop" | "slice" | "chunks" | "flatMap") {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());

                                    // split() always returns [string]
                                    if method_call.method == "split" {
                                        self.typed_array_vars.insert(name.to_string(), "string".to_string());
                                    }

                                    // Propagate element type from source array to filter/map result
                                    if let Some(base_var_name) =
                                        self.get_base_var_name(&method_call.object)
                                    {
                                        if let Some(elem_type) =
                                            self.typed_array_vars.get(&base_var_name).cloned()
                                        {
                                            self.typed_array_vars
                                                .insert(name.to_string(), elem_type);
                                        }
                                    }

                                    // If the method is called on a JsonValue, the result is also Vec<JsonValue>
                                    // BUT we need to mark it as json_value for proper forEach iteration
                                    if self.is_json_value_expr(&method_call.object) {
                                        self.json_value_vars.insert(name.to_string());
                                    }
                                }
                            }
                            // Bug #35: .split() returns array of strings
                            else if method_call.method.as_str() == "split" {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());
                                    self.typed_array_vars
                                        .insert(name.to_string(), "string".to_string());
                                }
                            }
                            // groupBy returns HashMap<K, Vec<V>> — track as map variable
                            else if method_call.method.as_str() == "groupBy" {
                                if let Some(name) = binding.name() {
                                    self.map_vars.insert(name.to_string());
                                }
                            }
                            // B110 fix: union/intersection/difference on Set return HashSet — track as set
                            else if matches!(method_call.method.as_str(), "union" | "intersection" | "difference") {
                                if let Some(name) = binding.name() {
                                    self.set_vars.insert(self.sanitize_name(name));
                                }
                            }
                            // Sys.args() returns Vec<String> - need direct indexing
                            else if method_call.method.as_str() == "args" {
                                if let Expr::Identifier(obj_name) = method_call.object.as_ref() {
                                    if obj_name == "Sys" {
                                        if let Some(name) = binding.name() {
                                            self.native_vec_string_vars.insert(name.to_string());
                                            self.array_vars.insert(name.to_string());
                                        }
                                    }
                                }
                            }
                            // .as_array() returns Vec<JsonValue> - mark as array for .length -> .len()
                            else if method_call.method.as_str() == "as_array" {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());
                                }
                            }
                            // Date.now() / Date.new() returns chrono::NaiveDateTime
                            if let Expr::Identifier(obj_name) = method_call.object.as_ref() {
                                // FIX-4: Track variables assigned from enum variant construction
                                // e.g., let expr = Expr.Identifier("x") → expr is of type Expr
                                if self.enum_variants.contains_key(obj_name.as_str()) {
                                    if let Some(name) = binding.name() {
                                        let san = self.sanitize_name(name);
                                        self.var_types.insert(san.clone(), obj_name.clone());
                                        // Track as class instance for clone behavior
                                        self.class_instance_vars.insert(san);
                                    }
                                }
                                if obj_name == "Date" && matches!(method_call.method.as_str(), "now" | "new") {
                                    if let Some(name) = binding.name() {
                                        self.date_vars.insert(name.to_string());
                                    }
                                }
                                // Server.create() returns axum::Router
                                if obj_name == "Server" && method_call.method == "create" {
                                    if let Some(name) = binding.name() {
                                        self.server_vars.insert(name.to_string());
                                        self.mutated_vars.insert(name.to_string());
                                    }
                                }
                                // DB.open() returns rusqlite::Connection (tracked for DB.exec/query/close)
                                if obj_name == "DB" && method_call.method == "open" {
                                    if let Some(name) = binding.name() {
                                        self.db_vars.insert(name.to_string());
                                    }
                                }
                                // d.add() on a Date variable also returns Date
                                let sanitized_obj = self.sanitize_name(obj_name);
                                if self.date_vars.contains(&sanitized_obj) && method_call.method == "add" {
                                    if let Some(name) = binding.name() {
                                        self.date_vars.insert(name.to_string());
                                    }
                                }
                            }
                            // B100 fix: Track variables assigned from method calls that return string
                            // e.g., let ch = this._peek() where _peek() returns string
                            if self.string_returning_methods.contains(&method_call.method) {
                                if let Some(name) = binding.name() {
                                    self.string_vars.insert(self.sanitize_name(name));
                                }
                            }
                            // B100 fix: Track variables assigned from method calls that return [T]
                            if let Some(elem_type) = self.array_returning_methods.get(&method_call.method).cloned() {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());
                                    if !elem_type.is_empty() {
                                        self.typed_array_vars.insert(name.to_string(), elem_type);
                                    }
                                }
                            }
                        }
                        // Mark instances created via constructor call: let x = ClassName(...)
                        else if let Expr::Call(call) = &var.init {
                            if let Expr::Identifier(class_name) = &*call.callee {
                                if let Some(name) = binding.name() {
                                    // BUG-007: Track variables assigned from optional-returning functions
                                    if self.optional_returning_functions.contains(class_name) {
                                        self.option_value_vars.insert(name.to_string());
                                    }
                                    // Check if this is an array-returning function
                                    else if let Some(elem_type) = self.array_returning_functions.get(class_name).cloned() {
                                        self.array_vars.insert(name.to_string());
                                        if !elem_type.is_empty() {
                                            self.typed_array_vars.insert(name.to_string(), elem_type);
                                        }
                                    }
                                    // Check if this is a string-returning function
                                    else if self.string_returning_functions.contains(class_name) {
                                        self.string_vars.insert(name.to_string());
                                    } else {
                                        self.class_instance_vars.insert(name.to_string());
                                        self.var_types.insert(name.to_string(), class_name.clone());
                                    }
                                }
                            }
                        }
                        // Track variables assigned from optional chaining: let name = user?.field
                        // The result is Option<T>, so track as option_value_var
                        else if matches!(&var.init, Expr::OptionalChain { .. }) {
                            if let Some(name) = binding.name() {
                                self.option_value_vars.insert(self.sanitize_name(name));
                            }
                        }
                        // Mark instances created via struct literal: let x = ClassName { ... }
                        else if let Expr::StructLiteral { type_name, .. } = &var.init {
                            if let Some(name) = binding.name() {
                                self.class_instance_vars.insert(name.to_string());
                                self.var_types.insert(name.to_string(), type_name.clone());
                            }
                        }
                        // Mark variables initialized from JSON indexing as json_value
                        // e.g., let items = result["items"] where result is a JsonValue
                        else if let Expr::Index { object, .. } = &var.init {
                            // Check if the object being indexed is a JsonValue
                            if self.is_json_value_expr(object) {
                                if let Some(name) = binding.name() {
                                    self.json_value_vars.insert(name.to_string());
                                }
                            }
                            // Bug #80 fix: Track variables indexed from map_array_vars as map_vars
                            // e.g., let row = rows[0] where rows is Vec<HashMap<String,String>>
                            if let Expr::Identifier(obj_name) = object.as_ref() {
                                let sanitized_obj = self.sanitize_name(obj_name);
                                if self.map_array_vars.contains(&sanitized_obj) {
                                    if let Some(name) = binding.name() {
                                        self.map_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Bug #94 fix: Track variables indexed from typed arrays
                                // e.g., let tok = toks[i] where toks is [string] → tok is string
                                // e.g., let p = people[i] where people is [Person] → p is class instance
                                if let Some(elem_type) = self.typed_array_vars.get(&sanitized_obj).cloned() {
                                    if let Some(name) = binding.name() {
                                        let var_name_s = self.sanitize_name(name);
                                        if elem_type == "string" {
                                            self.string_vars.insert(var_name_s);
                                        } else if self.class_fields.contains_key(&elem_type) {
                                            self.class_instance_vars.insert(var_name_s.clone());
                                            self.var_types.insert(var_name_s, elem_type);
                                        }
                                    }
                                }
                            }
                        }

                        // Only add 'mut' if the variable is actually mutated after declaration
                        let needs_mut = self.mutated_vars.contains(&var_name);
                        if needs_mut {
                            write!(self.output, "let mut {}", var_name).unwrap();
                        } else {
                            write!(self.output, "let {}", var_name).unwrap();
                        }

                        if let Some(type_ref) = &binding.type_ref {
                            let rust_type = self.expand_type_alias(type_ref);
                            write!(self.output, ": {}", rust_type).unwrap();

                            // Track optional variables for Some() wrapping on assignment
                            if matches!(type_ref, TypeRef::Optional(_)) {
                                self.option_value_vars.insert(var_name.clone());
                            }

                            // Track string type for .length -> .len() conversion
                            if matches!(type_ref, TypeRef::Simple(name) if name == "string") {
                                self.string_vars.insert(var_name.clone());
                            }

                            // Track Date type for property/method access generation
                            if matches!(type_ref, TypeRef::Simple(name) if name == "Date") {
                                self.date_vars.insert(var_name.clone());
                            }

                            // Bug #35 fix: Track array types for proper forEach/map lambda patterns
                            // e.g., let parts: [string] = text.split(",") should use |p| not |&p|
                            if let TypeRef::Array(elem_type) = type_ref {
                                self.array_vars.insert(var_name.clone());
                                if let TypeRef::Simple(type_name) = elem_type.as_ref() {
                                    self.typed_array_vars
                                        .insert(var_name.clone(), type_name.clone());
                                }
                            }
                        }

                        // Set float literal suffix context for f32 types
                        let prev_float_suffix = self.float_literal_suffix.clone();
                        if let Some(type_ref) = &binding.type_ref {
                            if matches!(type_ref, TypeRef::Simple(name) if name == "f32") {
                                self.float_literal_suffix = "f32".to_string();
                            }
                        }

                        self.output.push_str(" = ");

                        // FIX-1 (ISSUE-001): Wrap init in Some() if type annotation is T?
                        // but the value is not already optional (null, optional chain, option-returning fn/method)
                        let needs_some_wrap_decl = binding.type_ref.as_ref()
                            .is_some_and(|t| matches!(t, TypeRef::Optional(_)))
                            && !matches!(&var.init, Expr::Literal(Literal::Null))
                            && !matches!(&var.init, Expr::OptionalChain { .. })
                            && !self.is_option_returning_method(&var.init)
                            && !self.init_is_already_optional(&var.init);

                        // Check if we need to wrap in a union variant
                        let (needs_union_close, mut needs_to_string) =
                            if let Some(type_ref) = &binding.type_ref {
                                self.maybe_wrap_in_union(type_ref, &var.init)
                            } else {
                                (false, false)
                            };

                        // Bug #17 fix: String literals should always be converted to String
                        // to avoid &str vs String type mismatch when variable is reassigned
                        if matches!(&var.init, Expr::Literal(Literal::String(_))) {
                            needs_to_string = true;
                        }

                        // Phase 1: Check if this is JSON.parse with type hint (typed parsing)
                        let is_json_parse = self.is_json_parse_call(&var.init);
                        let has_type_hint = binding.type_ref.is_some();

                        // FIX-1: Open Some() wrapper for T? = non-optional-value
                        if needs_some_wrap_decl {
                            self.output.push_str("Some(");
                        }

                        if is_json_parse && has_type_hint {
                            // Typed JSON parsing — not expected with T? annotation, skip Some wrap
                            if let Expr::MethodCall(method_call) = &var.init {
                                self.generate_typed_json_parse(
                                    method_call,
                                    binding.type_ref.as_ref().unwrap(),
                                )?;
                                self.output.push_str(".expect(\"JSON parse failed\")");
                            }
                        } else if is_json_parse {
                            // Untyped JSON parsing (original behavior): let data = JSON.parse(body)
                            // Mark this variable as JsonValue for lambda pattern detection
                            if let Some(name) = binding.name() {
                                self.json_value_vars.insert(name.to_string());
                            }

                            // Generate: let posts = JSON.parse(body).0.expect("JSON parse failed");
                            self.generate_expr(&var.init)?;
                            self.output.push_str(".0.expect(\"JSON parse failed\")");
                        } else if self.is_json_stringify_call(&var.init) {
                            // Bug #39 fix: JSON.stringify without error binding
                            // JSON.stringify returns (Option<String>, String), extract just the value
                            // Generate: let json = JSON.stringify(obj).0.unwrap_or_default();
                            self.generate_expr(&var.init)?;
                            self.output.push_str(".0.unwrap_or_default()");
                        } else {
                            // Auto-clone when assigning this.field to a local variable
                            let needs_clone = self.expr_is_self_field(&var.init);
                            self.generate_expr(&var.init)?;
                            if needs_clone {
                                self.output.push_str(".clone()");
                            }
                        }

                        // Add .to_string() if needed for string literals
                        if needs_to_string {
                            self.output.push_str(".to_string()");
                        }

                        // FIX-1: Close Some() wrapper
                        if needs_some_wrap_decl {
                            self.output.push(')');
                        }

                        // Close union wrapper if opened
                        if needs_union_close {
                            self.output.push(')');
                        }

                        // Restore float literal suffix context
                        self.float_literal_suffix = prev_float_suffix;

                        self.output.push_str(";\n");
                    }
                }
            }
            Stmt::ConstDecl(const_decl) => {
                self.write_indent();
                write!(self.output, "const {}: ", const_decl.name.to_uppercase()).unwrap();
                let type_str = if let Some(type_ref) = &const_decl.type_ref {
                    let rust_type = type_ref.to_rust_type();
                    // B31: const string can't use String (heap-allocated), must use &str
                    if rust_type == "String" {
                        "&str".to_string()
                    } else {
                        rust_type
                    }
                } else {
                    self.infer_const_type(&const_decl.init)
                };
                self.output.push_str(&type_str);
                self.output.push_str(" = ");
                // B31: For const string, don't add .to_string() — just use the literal directly
                let is_const_str = type_str == "&str";
                if is_const_str {
                    // Generate string literal without .to_string() conversion
                    if let Expr::Literal(Literal::String(s)) = &const_decl.init {
                        write!(self.output, "\"{}\"", s).unwrap();
                    } else {
                        self.generate_expr(&const_decl.init)?;
                    }
                } else {
                    self.generate_expr(&const_decl.init)?;
                }
                self.output.push_str(";\n");
            }
            Stmt::Assign(assign) => {
                // ── push_str optimization: x = x + expr → x.push_str(...) ──
                if let Expr::Identifier(var_name) = &assign.target {
                    let sanitized_target = self.sanitize_name(var_name);
                    if self.string_vars.contains(&sanitized_target) {
                        if let Expr::Binary { op: BinOp::Add, left, right } = &assign.value {
                            if let Expr::Identifier(left_name) = left.as_ref() {
                                if self.sanitize_name(left_name) == sanitized_target {
                                    // Pattern: var = var + rhs → var.push_str(...)
                                    self.write_indent();
                                    write!(self.output, "{}.push_str(", sanitized_target).unwrap();
                                    // Determine how to pass rhs
                                    match right.as_ref() {
                                        Expr::Literal(Literal::String(s)) => {
                                            write!(self.output, "\"{}\"", s).unwrap();
                                        }
                                        Expr::Identifier(rhs_name) => {
                                            let rhs_san = self.sanitize_name(rhs_name);
                                            if self.string_vars.contains(&rhs_san) {
                                                write!(self.output, "&{}", rhs_san).unwrap();
                                            } else {
                                                write!(self.output, "&{}.to_string()", rhs_san).unwrap();
                                            }
                                        }
                                        _ => {
                                            self.output.push_str("&");
                                            self.generate_expr(right)?;
                                            self.output.push_str(".to_string()");
                                        }
                                    }
                                    self.output.push_str(");\n");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }

                // Check if assigning an object literal - mark variable for bracket notation
                if let Expr::Identifier(var_name) = &assign.target {
                    if let Expr::ObjectLiteral(_) = &assign.value {
                        self.bracket_notation_vars.insert(var_name.clone());
                    }
                    // Check if assigning from a class constructor call - mark variable as class instance
                    else if let Expr::Call(call) = &assign.value {
                        if let Expr::Identifier(_class_name) = &*call.callee {
                            // Check if this is a class constructor call by looking at the context
                            // For now, assume any call to an identifier is a potential class constructor
                            // This is a heuristic - we could improve this by checking against known class names
                            self.class_instance_vars.insert(var_name.clone());
                        }
                    }
                }

                self.write_indent();
                self.in_assignment_target = true;
                self.generate_expr(&assign.target)?;
                self.in_assignment_target = false;
                self.output.push_str(" = ");

                // Check if assigning to an optional variable — wrap in Some()
                let needs_some_wrap = if let Expr::Identifier(var_name) = &assign.target {
                    let san = self.sanitize_name(var_name);
                    self.option_value_vars.contains(&san) && !matches!(&assign.value, Expr::Literal(Literal::Null))
                } else if let Expr::Member { object, property } = &assign.target {
                    // Check this.field = value for optional class fields
                    if let Expr::Identifier(name) = object.as_ref() {
                        if (name == "this" || name == "self") {
                            if let Some(class_name) = &self.current_class_name {
                                self.class_optional_fields
                                    .get(class_name)
                                    .map_or(false, |fields| fields.contains(property))
                                    && !matches!(&assign.value, Expr::Literal(Literal::Null))
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if needs_some_wrap {
                    self.output.push_str("Some(");
                }

                // If assigning a string literal to what might be a String field, add .to_string()
                if let Expr::Literal(Literal::String(_)) = &assign.value {
                    self.generate_expr(&assign.value)?;
                    self.output.push_str(".to_string()");
                } else {
                    self.generate_expr(&assign.value)?;
                }

                if needs_some_wrap {
                    self.output.push_str(")");
                }

                self.output.push_str(";\n");
            }
            Stmt::If(if_stmt) => {
                // BUG-007: Detect `if x != null { ... }` where x is Option<T>
                // Transform to `if let Some(x) = x { ... }` for type narrowing
                let option_null_var = self.extract_option_null_check(&if_stmt.condition);
                
                self.write_indent();
                if let Some(ref var_name) = option_null_var {
                    // Generate: if let Some(var) = var { ... }
                    write!(self.output, "if let Some({}) = {} {{\n", var_name, var_name).unwrap();
                } else {
                    self.output.push_str("if ");
                    self.generate_condition_expr(&if_stmt.condition)?;
                    self.output.push_str(" {\n");
                }
                self.indent();
                // BUG-007: Inside the narrowed block, the variable is no longer Option
                if let Some(ref var_name) = option_null_var {
                    self.option_value_vars.remove(var_name);
                }
                self.generate_if_body(&if_stmt.then_branch)?;
                // BUG-007: Restore Option tracking after the block
                if let Some(ref var_name) = option_null_var {
                    self.option_value_vars.insert(var_name.clone());
                }
                self.dedent();
                self.write_indent();
                self.output.push('}');

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent();
                    self.generate_if_body(else_branch)?;
                    self.dedent();
                    self.write_indent();
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            Stmt::While(while_stmt) => {
                self.write_indent();
                self.output.push_str("while ");
                self.generate_expr(&while_stmt.condition)?;
                self.output.push_str(" {\n");
                self.indent();
                self.generate_block_inner(&while_stmt.body)?;
                self.dedent();
                self.writeln("}");
            }
            Stmt::For(for_stmt) => {
                // Two-variable for loop: for key, value in map OR for i, item in array
                if let Some(ref var2_name) = for_stmt.var2 {
                    let var1_name = self.sanitize_name(&for_stmt.var);
                    let var2_name_san = self.sanitize_name(var2_name);

                    // Detect if iterable is a Map (key-value iteration) or Array (enumerate)
                    let is_map_iteration = match &for_stmt.iterable {
                        Expr::Identifier(name) => {
                            let sanitized = self.sanitize_name(name);
                            self.map_vars.contains(&sanitized) || self.map_vars.contains(name.as_str())
                        }
                        _ => false,
                    };

                    if is_map_iteration {
                        // Map iteration: for key, value in map { ... }
                        self.write_indent();
                        write!(self.output, "for ({}, {}) in ", var1_name, var2_name_san).unwrap();
                        self.generate_expr(&for_stmt.iterable)?;
                        self.output.push_str(".iter()");
                        self.output.push_str(" {\n");
                        self.indent();
                        // Bug #79 fix: Clone loop variables to get owned values
                        self.writeln(&format!("let {} = {}.clone();", var1_name, var1_name));
                        self.writeln(&format!("let {} = {}.clone();", var2_name_san, var2_name_san));
                        self.string_vars.insert(var1_name.clone());
                        self.string_vars.insert(var2_name_san.clone());
                    } else {
                        // Array enumerate: for i, item in array { ... }
                        self.write_indent();
                        write!(self.output, "for ({}, {}) in ", var1_name, var2_name_san).unwrap();
                        self.generate_expr(&for_stmt.iterable)?;
                        self.output.push_str(".iter().enumerate()");
                        self.output.push_str(" {\n");
                        self.indent();
                        // enumerate() returns (usize, &T) — cast index to i32 for Liva's int type
                        self.writeln(&format!("let {} = {} as i32;", var1_name, var1_name));
                        // Clone item to get owned value
                        self.writeln(&format!("let {} = {}.clone();", var2_name_san, var2_name_san));
                        self.string_vars.insert(var2_name_san.clone());
                    }

                    self.generate_block_inner(&for_stmt.body)?;
                    self.dedent();
                    self.writeln("}");
                } else {
                // Single-variable for loop (arrays, ranges, etc.)
                // Mark the loop variable for bracket notation (likely iterating over JSON objects)
                let var_name = self.sanitize_name(&for_stmt.var);
                self.bracket_notation_vars.insert(var_name.clone());

                // Bug fix: If iterating over a typed array of class instances,
                // register the loop variable as a class instance so field access
                // generates dot notation instead of get_field()
                if let Expr::Identifier(iterable_name) = &for_stmt.iterable {
                    let sanitized_iterable = self.sanitize_name(iterable_name);
                    // typed_array_vars may store raw (camelCase) or sanitized (snake_case) names
                    let element_type = self.typed_array_vars.get(&sanitized_iterable).cloned()
                        .or_else(|| self.typed_array_vars.get(iterable_name.as_str()).cloned());
                    if let Some(element_type) = element_type {
                        // Check if element type is a class (uppercase first char, not a primitive)
                        let is_class_type = !matches!(
                            element_type.as_str(),
                            "number" | "int" | "i32" | "float" | "f64" | "bool" | "char" | "string"
                        ) && element_type.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                        if is_class_type {
                            self.class_instance_vars.insert(var_name.clone());
                            self.var_types.insert(var_name.clone(), element_type.clone());
                        }
                        // Bug #75: Register string loop variables so they get .clone()
                        // when passed to functions inside the loop body
                        if element_type == "string" {
                            self.string_vars.insert(var_name.clone());
                        }
                    }
                    // Also handle arrays from .split() that are tracked as array_vars
                    // but may not be in typed_array_vars — default to string element
                    if (self.array_vars.contains(&sanitized_iterable) || self.array_vars.contains(iterable_name.as_str()))
                        && !self.typed_array_vars.contains_key(&sanitized_iterable)
                        && !self.typed_array_vars.contains_key(iterable_name.as_str())
                    {
                        self.string_vars.insert(var_name.clone());
                    }
                    // DB.query() results: Vec<HashMap<String,String>> — loop var is a HashMap
                    if self.map_array_vars.contains(&sanitized_iterable) {
                        self.map_vars.insert(var_name.clone());
                    }
                }
                // Also handle `for item in obj.field` where obj is a class instance
                if let Expr::Member { object, property } = &for_stmt.iterable {
                    if let Expr::Identifier(obj_name) = object.as_ref() {
                        let sanitized_obj = self.sanitize_name(obj_name);
                        // Check typed_array_vars for the field name
                        if let Some(element_type) = self.typed_array_vars.get(property).cloned() {
                            let is_class_type = !matches!(
                                element_type.as_str(),
                                "number" | "int" | "i32" | "float" | "f64" | "bool" | "char" | "string"
                            ) && element_type.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                            if is_class_type {
                                self.class_instance_vars.insert(var_name.clone());
                            }
                        }
                        // Look up the object's class type via var_types, then check class_array_field_types
                        if let Some(class_name) = self.var_types.get(&sanitized_obj).cloned() {
                            if let Some(field_types) = self.class_array_field_types.get(&class_name) {
                                if let Some(element_type) = field_types.get(property) {
                                    let is_class_type = !matches!(
                                        element_type.as_str(),
                                        "number" | "int" | "i32" | "float" | "f64" | "bool" | "char" | "string"
                                    ) && element_type.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                                    if is_class_type {
                                        self.class_instance_vars.insert(var_name.clone());
                                        self.var_types.insert(var_name.clone(), element_type.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                self.write_indent();
                write!(self.output, "for {} in ", var_name).unwrap();

                // Bug #74 fix: In Liva, iterating doesn't consume collections. In Rust,
                // `for x in vec` moves the vec. Use .clone() or .iter() to avoid this.
                // B45 fix: For self.field in &mut self methods where loop body mutates
                // the loop variable, use .iter_mut() so mutations are not lost.
                let needs_clone = match &for_stmt.iterable {
                    Expr::Identifier(_) | Expr::Member { .. } => true,
                    _ => false,
                };
                let is_self_field = match &for_stmt.iterable {
                    Expr::Member { object, .. } => {
                        matches!(object.as_ref(), Expr::Identifier(name) if name == "this")
                    }
                    _ => false,
                };
                // Check if this for-loop mutates the loop variable's fields
                let mutates_loop_var = is_self_field
                    && self.current_method_is_mut
                    && self.block_assigns_to_var_field(&for_stmt.body, &for_stmt.var);
                // BUG-005 fix: Check if iterable is a string variable — if so,
                // emit .chars() instead of .clone() since String is not iterable in Rust.
                let is_string_iterable = match &for_stmt.iterable {
                    Expr::Identifier(name) => {
                        let sanitized = self.sanitize_name(name);
                        self.string_vars.contains(&sanitized) || self.string_vars.contains(name.as_str())
                    }
                    Expr::Member { object, property } => {
                        // this.field where field is a string
                        if matches!(object.as_ref(), Expr::Identifier(name) if name == "this") {
                            self.string_vars.contains(property)
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                self.generate_expr(&for_stmt.iterable)?;
                if is_string_iterable {
                    self.output.push_str(".chars()");
                } else if is_self_field && mutates_loop_var {
                    self.output.push_str(".iter_mut()");
                } else if is_self_field || needs_clone {
                    self.output.push_str(".clone()");
                }
                self.output.push_str(" {\n");
                self.indent();

                // BUG-005: When iterating over string chars, the loop variable is a Rust `char`.
                // In Liva, characters are strings, so convert to String for compatibility.
                if is_string_iterable {
                    self.writeln(&format!("let {} = {}.to_string();", var_name, var_name));
                    self.string_vars.insert(var_name.clone());
                }

                // Phase 11.3: Point-free in for loops
                // for item in items => print  →  for item in items { print(item) }
                // for item in items => mifuncion  →  for item in items { mifuncion(item) }
                let is_point_free_body = for_stmt.body.stmts.len() == 1
                    && matches!(&for_stmt.body.stmts[0], Stmt::Expr(expr_stmt)
                        if matches!(&expr_stmt.expr, Expr::Identifier(_) | Expr::MethodRef { .. }));

                if is_point_free_body {
                    if let Stmt::Expr(expr_stmt) = &for_stmt.body.stmts[0] {
                        match &expr_stmt.expr {
                            Expr::Identifier(func_name) => {
                                self.write_indent();
                                match func_name.as_str() {
                                    "print" => {
                                        write!(self.output, "println!(\"{{}}\", {});\n", var_name)
                                            .unwrap();
                                    }
                                    "toString" => {
                                        write!(self.output, "format!(\"{{}}\", {});\n", var_name)
                                            .unwrap();
                                    }
                                    _ => {
                                        let sanitized = self.sanitize_name(func_name);
                                        write!(self.output, "{}({});\n", sanitized, var_name)
                                            .unwrap();
                                    }
                                }
                            }
                            Expr::MethodRef { object, method } => {
                                // Phase 11.4: for item in items => Utils::log
                                self.write_indent();
                                let sanitized_obj = self.sanitize_name(object);
                                let sanitized_method = self.sanitize_name(method);
                                let is_class =
                                    object.chars().next().map_or(false, |c| c.is_uppercase());

                                if method == "new" {
                                    write!(self.output, "{}::new({});\n", sanitized_obj, var_name)
                                        .unwrap();
                                } else if is_class {
                                    write!(
                                        self.output,
                                        "{}::{}({});\n",
                                        sanitized_obj, sanitized_method, var_name
                                    )
                                    .unwrap();
                                } else {
                                    write!(
                                        self.output,
                                        "{}.{}({});\n",
                                        sanitized_obj, sanitized_method, var_name
                                    )
                                    .unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    self.generate_block_inner(&for_stmt.body)?;
                }

                self.dedent();
                self.writeln("}");
                } // end else (single-variable for loop)
            }
            Stmt::Switch(switch_stmt) => {
                // Check if this is a string-based switch (if any case value is a string literal)
                let is_string_switch = switch_stmt
                    .cases
                    .iter()
                    .any(|case| matches!(&case.value, Expr::Literal(Literal::String(_))));

                // BUG-008: Check if discriminant is an Option variable
                let is_option_discriminant = if let Expr::Identifier(name) = &switch_stmt.discriminant {
                    let sanitized = self.sanitize_name(name);
                    self.option_value_vars.contains(&sanitized)
                } else {
                    false
                };

                // FIX-3 (ISSUE-003): Detect enum switches with data bindings.
                // When matching on a local variable with enum data variants,
                // use `match &variable` to avoid consuming the variable.
                let is_enum_data_switch = !is_string_switch && switch_stmt.cases.iter().any(|case| {
                    if let Expr::MethodCall(mc) = &case.value {
                        if let Expr::Identifier(name) = mc.object.as_ref() {
                            self.enum_variants.contains_key(name) && !mc.args.is_empty()
                        } else { false }
                    } else { false }
                });
                let needs_borrow = is_enum_data_switch
                    && matches!(&switch_stmt.discriminant, Expr::Identifier(_))
                    && !is_option_discriminant;

                self.write_indent();
                self.output.push_str("match ");
                // FIX-3: Borrow discriminant to avoid move
                if needs_borrow {
                    self.output.push('&');
                }
                self.generate_expr(&switch_stmt.discriminant)?;
                // Add .as_str() for string-based switches so literals match
                if is_string_switch {
                    if is_option_discriminant {
                        // BUG-008: Use .as_deref() for Option<String> -> Option<&str>
                        self.output.push_str(".as_deref()");
                    } else {
                        self.output.push_str(".as_str()");
                    }
                }
                self.output.push_str(" {\n");
                self.indent();

                for case in &switch_stmt.cases {
                    self.write_indent();

                    // BUG-008: Handle null case → None pattern
                    let is_null_case = matches!(&case.value, Expr::Literal(Literal::Null));

                    // BUG-008: For null cases on Option discriminant, generate None pattern
                    let (boxed_deref_bindings, enum_pattern) = if is_null_case && is_option_discriminant {
                        self.output.push_str("None");
                        (Vec::new(), None)
                    } else {
                        // BUG-008: Wrap non-null case values in Some() for Option discriminants
                        if is_option_discriminant {
                            self.output.push_str("Some(");
                        }
                        // Detect enum variant patterns: Enum.Variant(a, b) parsed as MethodCall
                        let result = if let Expr::MethodCall(mc) = &case.value {
                            if let Expr::Identifier(name) = mc.object.as_ref() {
                                if self.enum_variants.contains_key(name) {
                                    // This is an enum variant pattern — generate as pattern, not expression
                                    let variant = &mc.method;
                                    let bindings: Vec<String> = mc
                                        .args
                                        .iter()
                                        .map(|a| {
                                            if let Expr::Identifier(id) = a {
                                                id.clone()
                                            } else {
                                                "_".to_string()
                                            }
                                        })
                                        .collect();
                                    let pattern = Pattern::EnumVariant {
                                        enum_name: name.clone(),
                                        variant_name: variant.clone(),
                                        bindings: bindings.clone(),
                                    };
                                    let deref_bindings =
                                        self.get_boxed_pattern_bindings(&pattern);
                                    self.generate_pattern(&pattern)?;
                                    (deref_bindings, Some(pattern))
                                } else {
                                    self.generate_expr(&case.value)?;
                                    (Vec::new(), None)
                                }
                            } else {
                                self.generate_expr(&case.value)?;
                                (Vec::new(), None)
                            }
                        } else {
                            self.generate_expr(&case.value)?;
                            (Vec::new(), None)
                        };
                        // BUG-008: Close Some() wrapper
                        if is_option_discriminant {
                            self.output.push(')');
                        }
                        result
                    };
                    self.output.push_str(" => {\n");
                    self.indent();
                    // Auto-dereference boxed bindings
                    // When matching by reference (needs_borrow), binding is &Box<T>,
                    // so *binding gives Box<T> — need .clone() first to get Box<T>,
                    // then deref to get T: `*binding.clone()`
                    // When matching by value, binding is Box<T>, so `*binding` gives T directly.
                    for binding in &boxed_deref_bindings {
                        if needs_borrow {
                            self.writeln(&format!("let {} = *{}.clone();", binding, binding));
                        } else {
                            self.writeln(&format!("let {} = *{};", binding, binding));
                        }
                    }
                    // FIX-3: Clone bindings when matching by reference (&expr)
                    // In `match &expr { Variant { field } => ... }`, field is &T.
                    // Clone to get owned values matching the behavior of match-by-value.
                    if needs_borrow {
                        if let Some(ref pat) = enum_pattern {
                            if let Pattern::EnumVariant { bindings, .. } = pat {
                                for b in bindings {
                                    if b != "_" {
                                        let san = self.sanitize_name(b);
                                        // Don't re-clone boxed bindings (already handled)
                                        if !boxed_deref_bindings.contains(&san) {
                                            self.writeln(&format!("let {} = {}.clone();", san, san));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // GAP-007 fix: Register pattern bindings as class instances
                    let registered = if let Some(ref pat) = enum_pattern {
                        self.register_pattern_bindings(pat)
                    } else {
                        Vec::new()
                    };
                    for stmt in &case.body {
                        self.generate_stmt(stmt)?;
                    }
                    self.unregister_pattern_bindings(&registered);
                    self.dedent();
                    self.writeln("}");
                }

                if let Some(default) = &switch_stmt.default {
                    self.writeln("_ => {");
                    self.indent();
                    for stmt in default {
                        self.generate_stmt(stmt)?;
                    }
                    self.dedent();
                    self.writeln("}");
                }

                self.dedent();
                self.writeln("}");
            }
            Stmt::TryCatch(try_catch) => {
                self.writeln("match (|| -> Result<(), Box<dyn std::error::Error>> {");
                self.indent();
                self.generate_block_inner(&try_catch.try_block)?;
                self.writeln("Ok(())");
                self.dedent();
                self.writeln("})() {");
                self.indent();
                self.writeln("Ok(_) => {},");
                self.write_indent();
                write!(
                    self.output,
                    "Err({}) => {{\n",
                    self.sanitize_name(&try_catch.catch_var)
                )
                .unwrap();
                self.indent();
                self.generate_block_inner(&try_catch.catch_block)?;
                self.dedent();
                self.writeln("}");
                self.dedent();
                self.writeln("}");
            }
            Stmt::Throw(throw_stmt) => {
                self.write_indent();
                self.output.push_str("panic!(\"{}\", ");
                self.generate_expr(&throw_stmt.expr)?;
                self.output.push_str(");\n");
            }
            Stmt::Return(ret) => {
                self.write_indent();
                self.output.push_str("return");
                if let Some(expr) = &ret.expr {
                    self.output.push(' ');
                    if self.in_fallible_function {
                        self.output.push_str("Ok(");
                        self.generate_return_expr(expr)?;
                        self.output.push(')');
                    } else if self.in_optional_function {
                        // BUG-006: return value → return Some(value), return null → return None
                        let is_null = matches!(expr, Expr::Literal(Literal::Null));
                        if is_null {
                            self.output.push_str("None");
                        } else {
                            self.output.push_str("Some(");
                            self.generate_return_expr(expr)?;
                            self.output.push(')');
                        }
                    } else {
                        self.generate_return_expr(expr)?;
                    }
                }
                self.output.push_str(";\n");
            }
            Stmt::Break => {
                self.write_indent();
                self.output.push_str("break;\n");
            }
            Stmt::Continue => {
                self.write_indent();
                self.output.push_str("continue;\n");
            }
            Stmt::Expr(expr_stmt) => {
                // liva/test: describe(), test(), lifecycle hooks generate blocks — no trailing ;
                let is_test_block_call = if let Expr::Call(call) = &expr_stmt.expr {
                    if let Expr::Identifier(name) = call.callee.as_ref() {
                        matches!(
                            name.as_str(),
                            "describe"
                                | "test"
                                | "beforeEach"
                                | "afterEach"
                                | "beforeAll"
                                | "afterAll"
                        )
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Fire-and-forget inference: async/par calls used as statements
                // (not assigned to a variable) are automatically fire-and-forget
                if let Expr::Call(call) = &expr_stmt.expr {
                    if matches!(call.exec_policy, ExecPolicy::Async | ExecPolicy::Par) {
                        self.write_indent();
                        self.generate_fire_call(call, match call.exec_policy {
                            ExecPolicy::Async => ConcurrencyMode::Async,
                            ExecPolicy::Par => ConcurrencyMode::Parallel,
                            _ => unreachable!(),
                        })?;
                        self.output.push_str(";\n");
                        return Ok(());
                    }
                }

                if is_test_block_call {
                    self.generate_expr(&expr_stmt.expr)?;
                } else {
                    self.write_indent();
                    self.generate_expr(&expr_stmt.expr)?;
                    self.output.push_str(";\n");
                }
            }
            Stmt::Block(block) => {
                self.writeln("{");
                self.indent();
                self.generate_block_inner(block)?;
                self.dedent();
                self.writeln("}");
            }
            Stmt::Defer(defer_stmt) => {
                // Generate a Rust scope guard using Drop trait.
                // `defer expr` → creates a guard variable that executes expr when dropped.
                let idx = self.defer_counter;
                self.defer_counter += 1;
                self.write_indent();
                writeln!(self.output, "let _defer_{} = {{", idx).unwrap();
                self.indent();
                self.write_indent();
                self.output.push_str("struct _DeferGuard<F: FnOnce()>(Option<F>);\n");
                self.write_indent();
                self.output.push_str("impl<F: FnOnce()> Drop for _DeferGuard<F> {\n");
                self.indent();
                self.write_indent();
                self.output.push_str("fn drop(&mut self) { if let Some(f) = self.0.take() { f(); } }\n");
                self.dedent();
                self.write_indent();
                self.output.push_str("}\n");
                self.write_indent();
                self.output.push_str("_DeferGuard(Some(|| {\n");
                self.indent();
                self.generate_stmt(&defer_stmt.body)?;
                self.dedent();
                self.write_indent();
                self.output.push_str("}))\n");
                self.dedent();
                self.write_indent();
                self.output.push_str("};\n");
            }
            Stmt::Fail(fail_stmt) => {
                self.write_indent();
                let fn_name = self.current_function_name.clone();
                let filename = self.source_filename.clone();
                let location = if fail_stmt.line > 0 {
                    format!("{}:{}", filename, fail_stmt.line)
                } else {
                    filename.clone()
                };

                // SH-004 fix: In test blocks, use panic!() instead of return Err(...)
                // because test functions have return type () not Result.
                // SH-002: In constructors, also use panic!() — new() returns Self, not Result.
                if self.in_test_block || self.in_constructor {
                    self.output.push_str("panic!(\"{}\", ");
                    match &fail_stmt.expr {
                        Expr::Literal(Literal::String(s)) => {
                            write!(self.output, "format!(\"FAIL [{}] at {}: {}\")", fn_name, location, s).unwrap();
                        }
                        Expr::Identifier(name) if self.error_binding_vars.contains(name) => {
                            // fail err — use the error's message
                            write!(self.output, "format!(\"FAIL [{}] at {}: {{:?}}\", {})", fn_name, location, name).unwrap();
                        }
                        _ => {
                            write!(self.output, "format!(\"FAIL [{}] at {}: {{}}\", ", fn_name, location).unwrap();
                            self.generate_expr(&fail_stmt.expr)?;
                            self.output.push(')');
                        }
                    }
                    self.output.push_str(");\n");
                    return Ok(());
                }

                // B20 fix: Determine whether to chain or create a new error.
                // Case 1: `fail err` (identifier that IS an error binding var) → always chain
                // Case 2: `fail "string"` → only chain if there's an error var in scope (by indent level)
                // Case 3: otherwise → Error::new
                let is_fail_with_error_var = if let Expr::Identifier(name) = &fail_stmt.expr {
                    self.error_binding_vars.contains(name)
                } else {
                    false
                };
                if is_fail_with_error_var {
                    // Case 1: `fail err` — chain using the error variable itself
                    let err_var = if let Expr::Identifier(name) = &fail_stmt.expr {
                        name.clone()
                    } else {
                        unreachable!()
                    };
                    write!(self.output,
                        "return Err(liva_rt::Error::chain(",
                    ).unwrap();
                    // Use the error's message as the chain message
                    write!(self.output,
                        "{}.as_ref().unwrap().message.clone()", err_var
                    ).unwrap();
                    write!(self.output,
                        ", \"{}\", \"{}\", {}.unwrap()))",
                        fn_name, location, err_var
                    ).unwrap();
                    self.output.push_str(";\n");
                } else if let Some(err_var) = self.find_error_var_in_scope() {
                    // Case 2: `fail "string"` with an error var in scope — chain with custom message
                    write!(self.output,
                        "return Err(liva_rt::Error::chain(",
                    ).unwrap();
                    self.generate_expr(&fail_stmt.expr)?;
                    write!(self.output,
                        ", \"{}\", \"{}\", {}.unwrap()))",
                        fn_name, location, err_var
                    ).unwrap();
                    self.output.push_str(";\n");
                } else {
                    // Case 3: No error var in scope — standalone error
                    write!(self.output,
                        "return Err(liva_rt::Error::new(",
                    ).unwrap();
                    self.generate_expr(&fail_stmt.expr)?;
                    write!(self.output,
                        ", \"{}\", \"{}\"))",
                        fn_name, location
                    ).unwrap();
                    self.output.push_str(";\n");
                }
            }
        }
        Ok(())
    }

    /// Find the most recent error binding variable in scope (for error chaining in `fail`)
    /// B20 fix: Uses scope stack to properly track which error vars are visible.
    /// Only vars declared in the current scope or parent scopes are returned.
    /// Vars from sibling/child blocks that have been exited are not visible.
    fn find_error_var_in_scope(&self) -> Option<String> {
        // Walk the scope stack from top (innermost) to bottom (outermost)
        // Return the first (most recent) error binding var found
        for scope in self.error_binding_scope_stack.iter().rev() {
            if let Some(last) = scope.last() {
                return Some(last.clone());
            }
        }
        None
    }

    /// Checks if an expression is a member access on 'this' (self.field) or array indexing (self.items[i])
    /// Returns true if the expression needs .clone() when used in assignment or return
    /// Bug #45-46: Extended to handle array indexing of generic array fields
    fn expr_is_self_field(&self, expr: &Expr) -> bool {
        // Direct field access: this.field
        if let Expr::Member {
            object,
            property: _,
        } = expr
        {
            if let Expr::Identifier(obj) = object.as_ref() {
                return obj == "this" && self.in_method;
            }
        }
        // Bug #45-46: Array indexing: this.items[i]
        if let Expr::Index { object, .. } = expr {
            if let Expr::Member { object: base, .. } = object.as_ref() {
                if let Expr::Identifier(obj) = base.as_ref() {
                    return obj == "this" && self.in_method;
                }
            }
        }
        false
    }

    /// Generate return expression with auto-clone for non-Copy types
    /// Detects when returning a field from self and automatically adds .clone()
    /// Bug #52: Also handles casting integer division to float when return type is f64
    fn generate_return_expr(&mut self, expr: &Expr) -> Result<()> {
        // Check if this is a string literal - needs .to_string() for String return type
        if let Expr::Literal(Literal::String(_)) = expr {
            self.generate_expr(expr)?;
            self.output.push_str(".to_string()");
            return Ok(());
        }

        // Bug #52: Check if return type is float and expression contains integer division
        // We need to generate proper float division, not cast after integer division
        if self
            .current_return_type
            .as_ref()
            .map_or(false, |t| t == "f64")
        {
            if let Expr::Binary {
                op: BinOp::Div,
                left,
                right,
            } = expr
            {
                // For division with float return type, cast left operand to f64
                self.output.push('(');
                self.generate_expr(left)?;
                self.output.push_str(") as f64 / (");
                self.generate_expr(right)?;
                self.output.push_str(") as f64");
                return Ok(());
            }
            // For other integer expressions (not division), cast the whole thing
            if self.expr_is_integer_expr(expr) {
                self.output.push('(');
                self.generate_expr(expr)?;
                self.output.push_str(") as f64");
                return Ok(());
            }
        }

        // Check if this is a member access on 'this' (self.field)
        // Use the helper function for this check
        let needs_clone = self.expr_is_self_field(expr);

        if needs_clone {
            self.generate_expr(expr)?;
            self.output.push_str(".clone()");
        } else {
            self.generate_expr(expr)?;
        }

        Ok(())
    }

    /// Check if an expression evaluates to an integer type (i32)
    /// Used for Bug #52 to detect when we need to cast to f64
    fn expr_is_integer_expr(&self, expr: &Expr) -> bool {
        match expr {
            // Integer literals are i32
            Expr::Literal(Literal::Int(_)) => true,
            // Float literals are f64
            Expr::Literal(Literal::Float(_)) => false,
            // Identifiers - check against known float vars or assume int
            Expr::Identifier(name) => {
                // If it's a known float variable, return false
                // Otherwise assume it could be int (conservative)
                !self.is_known_float_var(name)
            }
            // Binary operations with arithmetic operators on integers produce integers
            Expr::Binary { op, left, right } => {
                matches!(
                    op,
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
                ) && self.expr_is_integer_expr(left)
                    && self.expr_is_integer_expr(right)
            }
            // Method calls - could return anything, assume not integer for safety
            _ => false,
        }
    }

    /// Check if a variable is known to be a float type
    fn is_known_float_var(&self, _name: &str) -> bool {
        // For now, we don't have float var tracking, so return false
        // This means we'll be conservative and cast when return type is f64
        false
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit)?,
            Expr::Identifier(name) => {
                // Convert 'this' to 'self' when inside a method
                let actual_name = if self.in_method && name == "this" {
                    "self"
                } else {
                    name
                };

                // Check if this is a constant (uppercase identifier)
                if actual_name.chars().all(|c| c.is_uppercase() || c == '_') {
                    write!(self.output, "{}", actual_name).unwrap();
                } else {
                    write!(self.output, "{}", self.sanitize_name(actual_name)).unwrap();
                }
            }
            Expr::Binary { op, left, right } => {
                if matches!(op, BinOp::Add) && self.expr_is_array(left, right)
                    // B28 fix: string concat takes priority over array concat
                    && !self.expr_is_stringy(left) && !self.expr_is_stringy(right)
                {
                    // Array concatenation: arr + [element] or arr + otherArr
                    // Generate: { let mut __v = left.clone(); __v.extend(right); __v }
                    self.output.push_str("{ let mut __v = ");
                    self.generate_expr(left)?;
                    self.output.push_str(".clone(); __v.extend(");
                    // B36 fix: Clone elements in array literals to prevent move
                    // In loops, the variable may be needed in subsequent iterations
                    if let Expr::ArrayLiteral(elements) = right.as_ref() {
                        self.output.push_str("vec![");
                        for (i, elem) in elements.iter().enumerate() {
                            if i > 0 { self.output.push_str(", "); }
                            self.generate_expr(elem)?;
                            // Clone non-Copy identifiers (structs, strings, arrays, etc.)
                            if let Expr::Identifier(name) = elem {
                                let sanitized = self.sanitize_name(name);
                                if self.class_instance_vars.contains(&sanitized)
                                    || self.string_vars.contains(&sanitized)
                                    || self.array_vars.contains(&sanitized)
                                    || self.map_vars.contains(&sanitized)
                                {
                                    self.output.push_str(".clone()");
                                }
                            }
                        }
                        self.output.push(']');
                    } else {
                        self.generate_expr(right)?;
                    }
                    self.output.push_str("); __v }");
                } else if matches!(op, BinOp::Add)
                    && (self.expr_is_stringy(left) || self.expr_is_stringy(right))
                {
                    self.output.push_str("format!(\"{}{}\", ");
                    self.generate_expr_for_string_concat(left)?;
                    self.output.push_str(", ");
                    self.generate_expr_for_string_concat(right)?;
                    self.output.push(')');
                } else {
                    self.generate_binary_operation(op, left, right)?;
                }
            }
            Expr::Unary { op, operand } => match op {
                crate::ast::UnOp::Await => {
                    // Check if we're awaiting a pending task variable
                    if let Expr::Identifier(name) = operand.as_ref() {
                        let sanitized = self.sanitize_name(name);
                        if self.pending_tasks.contains_key(&sanitized) {
                            // Generate task_name_task.await.unwrap() instead of task_name.await
                            // JoinHandle.await returns Result<T, JoinError>, needs unwrap
                            write!(self.output, "{}_task.await.unwrap()", sanitized).unwrap();
                            // Mark task as explicitly awaited
                            if let Some(task_info) = self.pending_tasks.get_mut(&sanitized) {
                                task_info.awaited = true;
                            }
                            return Ok(());
                        }
                    }
                    self.generate_expr(operand)?;
                    self.output.push_str(".await");
                }
                crate::ast::UnOp::Not => {
                    // Special handling for !error_var -> error_var.is_none()
                    if let Expr::Identifier(name) = operand.as_ref() {
                        let sanitized = self.sanitize_name(name);
                        if self.error_binding_vars.contains(&sanitized) {
                            write!(self.output, "{}.is_none()", sanitized).unwrap();
                            return Ok(());
                        }
                        // String error vars (from HTTP/File): !err -> err.is_empty()
                        if self.string_error_vars.contains(&sanitized) {
                            write!(self.output, "{}.is_empty()", sanitized).unwrap();
                            return Ok(());
                        }
                    }
                    write!(self.output, "{}", op).unwrap();
                    self.generate_expr(operand)?;
                }
                _ => {
                    write!(self.output, "{}", op).unwrap();
                    self.generate_expr(operand)?;
                }
            },
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                // Check if this ternary contains a fail - if so, generate as Result
                let has_fail =
                    self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr);

                if has_fail {
                    // Generate as Result: if cond { Ok(then) or Err(...) } else { Err(...) or Ok(else) }
                    self.output.push_str("if ");
                    self.generate_expr(condition)?;
                    self.output.push_str(" { ");
                    // For the then branch, check if it's a fail
                    if let Expr::Fail(expr) = then_expr.as_ref() {
                        self.output.push_str("return Err(liva_rt::Error::from(");
                        self.generate_expr(expr)?;
                        self.output.push_str("))");
                    } else {
                        self.output.push_str("Ok(");
                        self.generate_expr(then_expr)?;
                        self.output.push_str(")");
                    }
                    self.output.push_str(" } else { ");
                    // For the else branch, check if it's a fail
                    if let Expr::Fail(expr) = else_expr.as_ref() {
                        self.output.push_str("Err(liva_rt::Error::from(");
                        self.generate_expr(expr)?;
                        self.output.push_str("))");
                    } else {
                        self.output.push_str("Ok(");
                        self.generate_expr(else_expr)?;
                        self.output.push_str(")");
                    }
                    self.output.push_str(" }");
                } else {
                    self.output.push_str("if ");
                    self.generate_expr(condition)?;
                    self.output.push_str(" { ");
                    self.generate_expr(then_expr)?;
                    self.output.push_str(" } else { ");
                    self.generate_expr(else_expr)?;
                    self.output.push_str(" }");
                }
            }
            Expr::Call(call) => {
                // Check if this is a .count() call on a sequence
                if let Expr::Member { object, property } = call.callee.as_ref() {
                    if property == "count" {
                        self.generate_expr(object)?;
                        self.output.push_str(".count()");
                        return Ok(());
                    }
                }
                self.generate_call_expr(call)?;
            }
            Expr::Member { object, property } => {
                // Server request parameter interception: req.params → __params, req.body → body
                if let Some(ref req_param_name) = self.server_request_param {
                    if let Expr::Identifier(name) = object.as_ref() {
                        if name == req_param_name {
                            match property.as_str() {
                                "params" => {
                                    self.output.push_str("__params");
                                    return Ok(());
                                }
                                "body" => {
                                    self.output.push_str("body.clone()");
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Math constants: Math.PI, Math.E
                if let Expr::Identifier(name) = object.as_ref() {
                    if name == "Math" {
                        match property.as_str() {
                            "PI" => {
                                self.output.push_str("std::f64::consts::PI");
                                return Ok(());
                            }
                            "E" => {
                                self.output.push_str("std::f64::consts::E");
                                return Ok(());
                            }
                            _ => {} // Fall through to method call handling
                        }
                    }

                    // Enum variant access: Color.Red → Color::Red
                    if self.enum_names.contains(name) {
                        write!(self.output, "{}::{}", name, property).unwrap();
                        return Ok(());
                    }

                    // Date property access: d.year, d.month, d.day, d.hour, d.minute, d.second
                    let sanitized_name = self.sanitize_name(name);
                    if self.date_vars.contains(&sanitized_name) {
                        match property.as_str() {
                            "year" => {
                                self.output.push_str("(chrono::Datelike::year(&");
                                self.generate_expr(object)?;
                                self.output.push_str(") as i32)");
                                return Ok(());
                            }
                            "month" => {
                                self.output.push_str("(chrono::Datelike::month(&");
                                self.generate_expr(object)?;
                                self.output.push_str(") as i32)");
                                return Ok(());
                            }
                            "day" => {
                                self.output.push_str("(chrono::Datelike::day(&");
                                self.generate_expr(object)?;
                                self.output.push_str(") as i32)");
                                return Ok(());
                            }
                            "hour" => {
                                self.output.push_str("(chrono::Timelike::hour(&");
                                self.generate_expr(object)?;
                                self.output.push_str(") as i32)");
                                return Ok(());
                            }
                            "minute" => {
                                self.output.push_str("(chrono::Timelike::minute(&");
                                self.generate_expr(object)?;
                                self.output.push_str(") as i32)");
                                return Ok(());
                            }
                            "second" => {
                                self.output.push_str("(chrono::Timelike::second(&");
                                self.generate_expr(object)?;
                                self.output.push_str(") as i32)");
                                return Ok(());
                            }
                            _ => {} // Fall through
                        }
                    }
                }

                // Phase 3.5: Special handling for error.message
                if let Expr::Identifier(name) = object.as_ref() {
                    let sanitized = self.sanitize_name(name);
                    if self.error_binding_vars.contains(&sanitized) && property == "message" {
                        write!(
                            self.output,
                            "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"None\")",
                            sanitized
                        )
                        .unwrap();
                        return Ok(());
                    }

                    // Special handling for Option<Struct> from tuple-returning functions
                    // For HTTP responses, File contents, JSON values, etc. - unwrap before accessing field
                    if self.option_value_vars.contains(&sanitized) {
                        if property == "length" {
                            // For JSON values: use .length() method
                            write!(self.output, "{}.as_ref().unwrap().length()", sanitized)
                                .unwrap();
                            return Ok(());
                        }

                        // Check if this is a JSON value (from JSON.parse, HTTP, etc.)
                        let is_json_value = self.json_value_vars.contains(&sanitized);

                        // Check if this is a struct field access (not JSON)
                        // Common struct fields: status, statusText, body, headers, content, etc.
                        let is_http_struct_field = matches!(
                            property.as_str(),
                            "status" | "statusText" | "body" | "headers" | "content" | "data"
                        );

                        if is_http_struct_field {
                            // Convert camelCase to snake_case for Rust structs
                            let rust_field = self.sanitize_name(property);
                            write!(
                                self.output,
                                "{}.as_ref().unwrap().{}",
                                sanitized, rust_field
                            )
                            .unwrap();
                            return Ok(());
                        }

                        // For Option<T> from .find() on class arrays, unwrap before field access
                        // If it's not a JSON value, it's a class instance wrapped in Option
                        if !is_json_value {
                            let rust_field = self.sanitize_name(property);
                            write!(
                                self.output,
                                "{}.as_ref().unwrap().{}",
                                sanitized, rust_field
                            )
                            .unwrap();
                            return Ok(());
                        }
                    }
                }

                // v0.11.0: Tuple member access - check if property is numeric (tuple.0, tuple.1, etc.)
                if property.parse::<usize>().is_ok() {
                    self.generate_expr(object)?;
                    write!(self.output, ".{}", property).unwrap();
                    return Ok(());
                }

                if property == "length" {
                    // Bug #90 fix: Check if this is a class instance with a 'length' field
                    // If so, emit .length as a struct field, NOT .len()
                    if let Expr::Identifier(var_name) = object.as_ref() {
                        let sanitized = self.sanitize_name(var_name);
                        if let Some(class_name) = self.var_types.get(&sanitized) {
                            if let Some(fields) = self.class_fields.get(class_name) {
                                if fields.contains("length") {
                                    self.generate_expr(object)?;
                                    self.output.push_str(".length");
                                    return Ok(());
                                }
                            }
                        }
                    }

                    // Check if this is a JsonValue (not an array, string, or class instance)
                    // JsonValue uses .length(), Rust arrays/strings use .len()
                    // Note: .len() returns usize, but Liva uses i32 (number), so we cast
                    // Bug #31 fix: Wrap in parens so .toString() works: (x.len() as i32).to_string()
                    match object.as_ref() {
                        Expr::Identifier(var_name) => {
                            let sanitized = self.sanitize_name(var_name);
                            // Check if this is a known JsonValue variable
                            let is_json_value = self.json_value_vars.contains(&sanitized)
                                || self.json_value_vars.contains(var_name);

                            if is_json_value {
                                // JsonValue uses .length() (already returns i32)
                                self.generate_expr(object)?;
                                self.output.push_str(".length()");
                            } else {
                                // Default to (obj.len() as i32) for strings, arrays, and other types
                                self.output.push('(');
                                self.generate_expr(object)?;
                                self.output.push_str(".len() as i32)");
                            }
                        }
                        _ => {
                            self.output.push('(');
                            self.generate_expr(object)?;
                            self.output.push_str(".len() as i32)");
                        }
                    }
                    return Ok(());
                }

                self.generate_expr(object)?;

                // Use bracket notation for JSON objects, dot notation for structs
                match object.as_ref() {
                    Expr::Identifier(var_name) => {
                        // Check if this is a Rust struct (HTTP response, etc.)
                        // Sanitize the name to match how it was stored in rust_struct_vars
                        let sanitized_name = self.sanitize_name(var_name);
                        let is_rust_struct = self.rust_struct_vars.contains(&sanitized_name);

                        // Check if this is likely a JsonValue (not array, not class instance, not rust struct)
                        if !is_rust_struct
                            && !self.is_class_instance(var_name)
                            && !self.array_vars.contains(var_name)
                            && !var_name.contains("person")
                            && !var_name.contains("user")
                        {
                            // Likely a JsonValue - use get_field()
                            write!(
                                self.output,
                                ".get_field(\"{}\").unwrap_or_default()",
                                property
                            )
                            .unwrap();
                            return Ok(());
                        }

                        // For class instances and Rust structs, use dot notation
                        if is_rust_struct
                            || self.is_class_instance(var_name)
                            || var_name.contains("person")
                            || var_name.contains("user")
                        {
                            // Convert camelCase to snake_case for Rust structs (+ keyword escape)
                            let rust_field = self.sanitize_name(property);
                            write!(self.output, ".{}", rust_field).unwrap();

                            // Clone common owned types when returning by value and not assigning
                            if self.in_method
                                && !self.in_assignment_target
                                && (property == "title"
                                    || property == "author"
                                    || property == "name"
                                    || property.contains("dni")
                                    || property.ends_with("text")
                                    || property.ends_with("data"))
                            {
                                self.output.push_str(".clone()");
                            }
                            return Ok(());
                        }

                        // For JSON access, generate bracket notation
                        write!(self.output, "[\"{}\"]", property).unwrap();

                        // Convert numeric properties automatically (but not in string templates - format! handles it)
                        if !self.in_string_template {
                            if property == "price"
                                || property == "age"
                                || property.contains("count")
                                || property.contains("total")
                                || property.contains("sum")
                            {
                                self.output.push_str(".as_f64().unwrap_or(0.0)");
                            } else if property == "name"
                                || property.contains("text")
                                || property.contains("data")
                            {
                                self.output.push_str(".as_string().unwrap_or_default()");
                            }
                        }
                    }
                    Expr::Index {
                        object: arr_obj, ..
                    } => {
                        // Bug #51 fix: Check if this is a typed array with class elements
                        // If so, use dot notation instead of bracket notation
                        let use_dot_notation = if let Expr::Identifier(arr_name) = arr_obj.as_ref()
                        {
                            let sanitized = self.sanitize_name(arr_name);
                            self.typed_array_vars
                                .get(&sanitized)
                                .map(|t| {
                                    // Check if element type is a class (starts with uppercase, not primitive)
                                    !matches!(
                                        t.as_str(),
                                        "number"
                                            | "int"
                                            | "i32"
                                            | "float"
                                            | "f64"
                                            | "bool"
                                            | "char"
                                            | "string"
                                    ) && t.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                                })
                                .unwrap_or(false)
                        } else if let Expr::Member {
                            object: member_obj,
                            property: member_prop,
                        } = arr_obj.as_ref()
                        {
                            // Bug #69 fix: Handle this.field[i].prop — check typed_array_vars for the field name
                            if matches!(member_obj.as_ref(), Expr::Identifier(name) if name == "this")
                            {
                                let sanitized = self.sanitize_name(member_prop);
                                self.typed_array_vars
                                    .get(&sanitized)
                                    .map(|t| {
                                        !matches!(
                                            t.as_str(),
                                            "number"
                                                | "int"
                                                | "i32"
                                                | "float"
                                                | "f64"
                                                | "bool"
                                                | "char"
                                                | "string"
                                        ) && t
                                            .chars()
                                            .next()
                                            .map(|c| c.is_uppercase())
                                            .unwrap_or(false)
                                    })
                                    .unwrap_or(false)
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if use_dot_notation {
                            // Class element - use dot notation with snake_case field name (+ keyword escape)
                            let rust_field = self.sanitize_name(property);
                            write!(self.output, ".{}", rust_field).unwrap();

                            // Bug #91 fix: Always clone fields accessed through array-indexed
                            // class elements. Primitive types (i32, f64, bool) implement Copy
                            // so .clone() is harmless; String/Vec/struct fields need it.
                            // This replaces the old Bug #51 hardcoded field name list.
                            if !self.in_assignment_target {
                                self.output.push_str(".clone()");
                            }
                        } else {
                            // Indexed access like array[index] - the result is typically a JSON object
                            // So property access should use bracket notation
                            write!(self.output, "[\"{}\"]", property).unwrap();

                            // For numeric fields in JSON objects, convert to appropriate type (but not in string templates)
                            if !self.in_string_template {
                                // Always convert price to f64 since it's commonly used in arithmetic
                                if property == "price" {
                                    self.output.push_str(".as_f64().unwrap_or(0.0)");
                                } else if property == "age"
                                    || property.contains("count")
                                    || property.contains("total")
                                    || property.contains("sum")
                                {
                                    self.output.push_str(".as_f64().unwrap_or(0.0)");
                                } else if property == "name"
                                    || property.contains("text")
                                    || property.contains("data")
                                {
                                    self.output.push_str(".as_string().unwrap_or_default()");
                                }
                            }
                        }
                    }
                    _ => {
                        // For other expressions, use dot notation
                        write!(self.output, ".{}", self.sanitize_name(property)).unwrap();
                    }
                }
            }
            Expr::Index { object, index } => {
                // Special handling for JsonValue (both Option<JsonValue> and JsonValue)
                // BUT: Skip this if we're in a string template (handled separately there)
                if !self.in_string_template {
                    if let Expr::Identifier(var_name) = object.as_ref() {
                        let sanitized = self.sanitize_name(var_name);
                        let is_option_json = self.option_value_vars.contains(&sanitized);

                        // Check if this might be a JsonValue (either Option or direct)
                        // We detect Option<JsonValue> via option_value_vars
                        // For direct JsonValue, we'll try to generate the method call
                        // and let Rust's type system validate it
                        if is_option_json {
                            // Option<JsonValue> case
                            match index.as_ref() {
                                Expr::Literal(Literal::String(key)) => {
                                    write!(self.output, "{}.as_ref().unwrap().get_field(\"{}\").unwrap_or_default()", sanitized, key).unwrap();
                                }
                                Expr::Identifier(index_var) => {
                                    // If the index is a variable, check if it's a string variable
                                    let index_sanitized = self.sanitize_name(index_var);
                                    if self.string_vars.contains(&index_sanitized) {
                                        // String variable - use get_field for object access
                                        write!(self.output, "{}.as_ref().unwrap().get_field(&{}).unwrap_or_default()", sanitized, index_sanitized).unwrap();
                                    } else {
                                        // Assume numeric index for array access
                                        write!(self.output, "{}.as_ref().unwrap().get(", sanitized)
                                            .unwrap();
                                        self.generate_expr(index)?;
                                        self.output.push_str(").unwrap_or_default()");
                                    }
                                }
                                _ => {
                                    write!(self.output, "{}.as_ref().unwrap().get(", sanitized)
                                        .unwrap();
                                    self.generate_expr(index)?;
                                    self.output.push_str(").unwrap_or_default()");
                                }
                            }
                            return Ok(());
                        }
                    }
                }

                // Special handling for string indexing: s[i] -> s.chars().nth(i).unwrap_or_default()
                // Rust strings are UTF-8 and don't support direct indexing
                // This must come BEFORE generate_expr(object) to prevent emitting the object first
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if self.string_vars.contains(&sanitized) {
                        // String indexing - use .chars().nth(i)
                        self.generate_expr(object)?;
                        self.output.push_str(".chars().nth((");
                        self.generate_expr(index)?;
                        self.output
                            .push_str(") as usize).map(|c| c.to_string()).unwrap_or_default()");
                        return Ok(());
                    }
                }

                self.generate_expr(object)?;

                // For native Vec<String> (from Sys.args()), use direct indexing with .clone()
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if self.native_vec_string_vars.contains(&sanitized) {
                        self.output.push_str("[(");
                        self.generate_expr(index)?;
                        self.output.push_str(") as usize].clone()");
                        return Ok(());
                    }
                }

                // For JsonValue direct access (not Option), check if object is a known JsonValue var.
                // SH-012 fix: Only use JsonValue access when the variable is explicitly tracked as
                // json_value_vars. Previously, any unknown var was assumed JsonValue, which broke
                // array indexing for types that don't implement Default.
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if !self.array_vars.contains(&sanitized)
                        && !self.class_instance_vars.contains(&sanitized)
                        && (self.json_value_vars.contains(&sanitized)
                            || !self.var_types.contains_key(&sanitized))
                    {
                        match index.as_ref() {
                            Expr::Literal(Literal::String(key)) => {
                                // Try JsonValue object access
                                write!(self.output, ".get_field(\"{}\").unwrap_or_default()", key)
                                    .unwrap();
                                return Ok(());
                            }
                            Expr::Literal(Literal::Int(num)) => {
                                // Try JsonValue array access with numeric literal
                                write!(
                                    self.output,
                                    ".get({} as usize).cloned().unwrap_or_default()",
                                    num
                                )
                                .unwrap();
                                return Ok(());
                            }
                            Expr::Identifier(index_var) => {
                                // If the index is a variable, check if it's a string variable
                                // String variables should use get_field(), numeric should use get()
                                let index_sanitized = self.sanitize_name(index_var);
                                if self.string_vars.contains(&index_sanitized) {
                                    // String variable - use get_field for object access
                                    self.output.push_str(".get_field(&");
                                    self.output.push_str(&index_sanitized);
                                    self.output.push_str(").unwrap_or_default()");
                                } else {
                                    // Assume numeric index for array access
                                    self.output.push_str(".get((");
                                    self.generate_expr(index)?;
                                    self.output
                                        .push_str(") as usize).cloned().unwrap_or_default()");
                                }
                                return Ok(());
                            }
                            _ => {
                                // Try JsonValue array access with expression
                                self.output.push_str(".get((");
                                self.generate_expr(index)?;
                                self.output
                                    .push_str(") as usize).cloned().unwrap_or_default()");
                                return Ok(());
                            }
                        }
                    }
                }

                // Handle nested JSON access: when object is another Index expression (e.g., issue["user"]["login"])
                // The object was already generated above, now we need to chain .get_field() for the nested access
                if let Expr::Index { .. } = object.as_ref() {
                    // Object is another Index, which means this is nested JSON access
                    // Generate .get_field("key") for the next level
                    match index.as_ref() {
                        Expr::Literal(Literal::String(key)) => {
                            write!(self.output, ".get_field(\"{}\").unwrap_or_default()", key)
                                .unwrap();
                            return Ok(());
                        }
                        Expr::Literal(Literal::Int(num)) => {
                            write!(
                                self.output,
                                ".get({} as usize).cloned().unwrap_or_default()",
                                num
                            )
                            .unwrap();
                            return Ok(());
                        }
                        _ => {
                            self.output.push_str(".get((");
                            self.generate_expr(index)?;
                            self.output
                                .push_str(") as usize).cloned().unwrap_or_default()");
                            return Ok(());
                        }
                    }
                }

                // Fall back to standard array indexing
                // Bug #34: For arrays with non-literal index (e.g., lines[i] where i is int),
                // we need to add `as usize` because Rust Vec indexing requires usize
                // Bug #42: Also handle this.field[idx] for generic class array fields
                self.output.push('[');

                // Determine if we need `as usize` conversion BEFORE generating the expression
                // so we can wrap it in parentheses if needed
                let needs_usize_conversion = match object.as_ref() {
                    Expr::Identifier(var_name) => {
                        let sanitized = self.sanitize_name(var_name);
                        // SH-012: Also convert for typed vars that bypassed JsonValue heuristic
                        if self.array_vars.contains(&sanitized)
                            || self.var_types.contains_key(&sanitized)
                        {
                            match index.as_ref() {
                                Expr::Literal(Literal::Int(_)) => false,
                                _ => true,
                            }
                        } else {
                            false
                        }
                    }
                    Expr::Member { .. } => match index.as_ref() {
                        Expr::Literal(Literal::Int(_)) => false,
                        _ => true,
                    },
                    _ => false,
                };

                // Wrap in parentheses if we need usize conversion
                if needs_usize_conversion {
                    self.output.push('(');
                }
                self.generate_expr(index)?;
                if needs_usize_conversion {
                    self.output.push_str(") as usize");
                }

                // Check if this is a non-Copy array element - need .clone()
                let needs_clone = if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    // Bug #82 fix: map_array_vars (from DB.query) contain HashMap — always need .clone()
                    if self.map_array_vars.contains(&sanitized) {
                        true
                    } else if self.array_vars.contains(&sanitized) {
                        if let Some(elem_type) = self.typed_array_vars.get(&sanitized) {
                            // Clone for string and class instance types (not number/bool)
                            // B100 fix: Also check enum_names for imported class types not in class_fields
                            elem_type == "string" || self.class_fields.contains_key(elem_type)
                                || elem_type.contains("[]")
                                || self.enum_names.contains(elem_type)
                                || (!matches!(elem_type.as_str(), "number" | "int" | "i32" | "float" | "f64" | "bool" | "char")
                                    && elem_type.chars().next().map_or(false, |c| c.is_uppercase()))
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if let Expr::Member { object: ma_obj, property: ma_prop } = object.as_ref() {
                    // Handle obj.field[i] — check class_array_field_types
                    if let Expr::Identifier(obj_name) = ma_obj.as_ref() {
                        let sanitized_obj = self.sanitize_name(obj_name);
                        // B21 fix: Handle this.field[i] — look up current class name
                        let resolved_class = if (obj_name == "this" || obj_name == "self") && self.in_method {
                            self.current_class_name.as_deref()
                        } else {
                            self.var_types.get(&sanitized_obj).map(|s| s.as_str())
                        };
                        if let Some(class_name) = resolved_class {
                            if let Some(field_types) = self.class_array_field_types.get(class_name) {
                                if let Some(elem_type) = field_types.get(ma_prop.as_str()) {
                                    // Clone for non-primitive types
                                    elem_type == "string" || self.class_fields.contains_key(elem_type.as_str())
                                } else {
                                    // Field is in an array class but element type unknown — clone to be safe
                                    true
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else if let Expr::Member { .. } = ma_obj.as_ref() {
                        // SH: Deep member chain like self._type_ctx.type_pool[idx]
                        // Can't determine element type — clone to be safe
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                self.output.push(']');

                // For non-Copy arrays, add .clone() because indexing returns a reference
                // But NOT when this is an assignment target (LHS of =)
                if needs_clone && !self.in_assignment_target {
                    self.output.push_str(".clone()");
                }

                // Convert numeric properties automatically
                if let Expr::Literal(Literal::String(prop)) = index.as_ref() {
                    if prop == "price"
                        || prop == "age"
                        || prop.contains("count")
                        || prop.contains("total")
                        || prop.contains("sum")
                    {
                        self.output.push_str(".as_f64().unwrap_or(0.0)");
                    } else if prop == "name" || prop.contains("text") || prop.contains("data") {
                        self.output.push_str(".as_string().unwrap_or_default()");
                    }
                }
            }
            Expr::ObjectLiteral(fields) => {
                // Generate as a struct initialization or JSON
                self.output.push_str("serde_json::json!({\n");
                self.indent();
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(",\n");
                    }
                    self.write_indent();
                    write!(self.output, "\"{}\": ", key).unwrap();
                    self.generate_expr(value)?;
                }
                self.output.push('\n');
                self.dedent();
                self.write_indent();
                self.output.push_str("})");
            }
            Expr::StructLiteral { type_name, fields } => {
                // Generate Rust struct literal directly instead of constructor call
                // This works for all cases: with or without explicit constructor
                write!(self.output, "{} {{ ", type_name).unwrap();

                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert field name to snake_case
                    let field_name = self.sanitize_name(key);
                    write!(self.output, "{}: ", field_name).unwrap();

                    // Add .to_string() for string literals
                    if let Expr::Literal(Literal::String(_)) = value {
                        self.generate_expr(value)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(value)?;
                    }
                }

                self.output.push_str(" }");
            }
            Expr::ArrayLiteral(elements) => {
                self.output.push_str("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // String literals in array literals need .to_string() to produce Vec<String>
                    // instead of Vec<&str>, which is incompatible with Liva's string type
                    if matches!(elem, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(elem)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(elem)?;
                    }
                }
                self.output.push(']');
            }
            Expr::MapLiteral(entries) => {
                if entries.is_empty() {
                    self.output.push_str("std::collections::HashMap::new()");
                } else {
                    self.output.push_str("std::collections::HashMap::from([");
                    for (i, (key, value)) in entries.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push('(');
                        if matches!(key, Expr::Literal(Literal::String(_))) {
                            self.generate_expr(key)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(key)?;
                        }
                        self.output.push_str(", ");
                        if matches!(value, Expr::Literal(Literal::String(_))) {
                            self.generate_expr(value)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(value)?;
                        }
                        self.output.push(')');
                    }
                    self.output.push_str("])");
                }
            }
            Expr::SetLiteral(elements) => {
                if elements.is_empty() {
                    self.output.push_str("std::collections::HashSet::new()");
                } else {
                    self.output.push_str("std::collections::HashSet::from([");
                    for (i, elem) in elements.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        if matches!(elem, Expr::Literal(Literal::String(_))) {
                            self.generate_expr(elem)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(elem)?;
                        }
                    }
                    self.output.push_str("])");
                }
            }
            Expr::Tuple(elements) => {
                self.output.push('(');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert string literals to String for tuple compatibility
                    if matches!(elem, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(elem)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(elem)?;
                    }
                }
                // Rust requires trailing comma for single-element tuples
                if elements.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Expr::StringTemplate { parts } => {
                self.output.push_str("format!(\"");

                for part in parts.iter() {
                    match part {
                        StringTemplatePart::Text(text) => {
                            for ch in text.chars() {
                                match ch {
                                    '"' => self.output.push_str("\\\""),
                                    '\\' => self.output.push_str("\\\\"),
                                    '\n' => self.output.push_str("\\n"),
                                    '\r' => self.output.push_str("\\r"),
                                    '\t' => self.output.push_str("\\t"),
                                    '{' => self.output.push_str("{{"),
                                    '}' => self.output.push_str("}}"),
                                    _ => self.output.push(ch),
                                }
                            }
                        }
                        StringTemplatePart::Expr(expr) => match expr.as_ref() {
                            // Literals always use Display
                            Expr::Literal(_) => {
                                self.output.push_str("{}");
                            }
                            // Simple identifiers: check if they're arrays or option values
                            Expr::Identifier(name) => {
                                if self.array_vars.contains(name) {
                                    self.output.push_str("{:?}");
                                } else if self.option_value_vars.contains(name)
                                    || self.error_binding_vars.contains(name)
                                {
                                    // Option values need special handling - will be unwrapped
                                    self.output.push_str("{}");
                                } else {
                                    self.output.push_str("{}");
                                }
                            }
                            // Member access uses Display
                            Expr::Member { .. } => {
                                self.output.push_str("{}");
                            }
                            // Index access uses Display
                            Expr::Index { .. } => {
                                self.output.push_str("{}");
                            }
                            // Binary operations (including comparisons) use Display
                            Expr::Binary { .. } => {
                                self.output.push_str("{}");
                            }
                            // Ternary/If expressions with string results use Display
                            Expr::Ternary { .. } => {
                                self.output.push_str("{}");
                            }
                            // Function calls use Display
                            Expr::Call { .. } | Expr::MethodCall { .. } => {
                                self.output.push_str("{}");
                            }
                            // Arrays and objects use Debug
                            Expr::ArrayLiteral(_) | Expr::ObjectLiteral(_) | Expr::Tuple(_) => {
                                self.output.push_str("{:?}");
                            }
                            // B29 fix: Default to Display for all other expression types
                            // (Unary, Cast, Lambda, etc.) — {:?} adds unwanted quotes on strings
                            _ => {
                                self.output.push_str("{}");
                            }
                        },
                    }
                }

                self.output.push('"');

                let exprs: Vec<&Expr> = parts
                    .iter()
                    .filter_map(|part| match part {
                        StringTemplatePart::Expr(expr) => Some(expr.as_ref()),
                        _ => None,
                    })
                    .collect();

                if !exprs.is_empty() {
                    self.output.push_str(", ");

                    // Mark that we're inside a string template for proper member access generation
                    let was_in_template = self.in_string_template;
                    self.in_string_template = true;

                    for (idx, expr) in exprs.iter().enumerate() {
                        if idx > 0 {
                            self.output.push_str(", ");
                        }
                        // Phase 3.5: If expr is an error binding variable, use Display (error trace)
                        if let Expr::Identifier(name) = expr {
                            let sanitized = self.sanitize_name(name);
                            if self.error_binding_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.as_ref().map(|e| format!(\"{{}}\", e)).unwrap_or_default()",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                            if self.option_value_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.as_ref().map(|v| v.to_string()).unwrap_or_default()",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                            // Date variables: auto-format as ISO 8601 string
                            if self.date_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.format(\"%Y-%m-%dT%H:%M:%S\")",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                        }
                        // Phase 3.6: If expr is index access on JSON value
                        if let Expr::Index { object, index } = expr {
                            if let Expr::Identifier(var_name) = object.as_ref() {
                                let sanitized = self.sanitize_name(var_name);
                                if self.option_value_vars.contains(&sanitized) {
                                    // Generate unwrapped index access for string template
                                    match index.as_ref() {
                                        Expr::Literal(Literal::String(key)) => {
                                            write!(self.output, "{}.as_ref().unwrap().get_field(\"{}\").unwrap_or_default()", sanitized, key).unwrap();
                                        }
                                        _ => {
                                            write!(
                                                self.output,
                                                "{}.as_ref().unwrap().get(",
                                                sanitized
                                            )
                                            .unwrap();
                                            self.generate_expr(index)?;
                                            self.output.push_str(").unwrap_or_default()");
                                        }
                                    }
                                    continue;
                                }
                            }
                        }
                        self.generate_expr(expr)?;
                    }

                    self.in_string_template = was_in_template;
                }

                self.output.push(')');
            }
            Expr::Lambda(lambda) => {
                if lambda.is_move {
                    self.output.push_str("move ");
                }
                self.output.push('|');

                for (idx, param) in lambda.params.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }

                    // For destructured parameters, use temporary names
                    let param_name = if param.is_destructuring() {
                        format!("_param_{}", idx)
                    } else {
                        self.sanitize_name(param.name().unwrap())
                    };

                    self.output.push_str(&param_name);
                    if let Some(type_ref) = &param.type_ref {
                        self.output.push_str(": ");
                        self.output.push_str(&type_ref.to_rust_type());
                    }
                }

                self.output.push_str("| ");

                // Check if we need to generate destructuring code
                let has_destructuring = lambda.params.iter().any(|p| p.is_destructuring());

                match &lambda.body {
                    LambdaBody::Expr(expr) => {
                        if has_destructuring {
                            // Need to wrap in a block to insert destructuring
                            self.output.push('{');
                            self.indent();
                            self.output.push('\n');

                            // Capture element type before mutable borrows
                            let element_type_for_destr = self.current_lambda_element_type.clone();

                            // Generate destructuring for lambda params
                            for (idx, param) in lambda.params.iter().enumerate() {
                                if param.is_destructuring() {
                                    let temp_name = format!("_param_{}", idx);
                                    self.write_indent();
                                    self.generate_lambda_param_destructuring(
                                        &param.pattern,
                                        &temp_name,
                                        false,
                                        element_type_for_destr.as_deref(),
                                    )?;
                                }
                            }

                            self.write_indent();
                            self.generate_expr(expr)?;
                            self.output.push('\n');
                            self.dedent();
                            self.write_indent();
                            self.output.push('}');
                        } else {
                            self.generate_expr(expr)?;
                        }
                    }
                    LambdaBody::Block(block) => {
                        // For lambdas, we need to generate a block that returns a value
                        // Check if the last statement is a return statement
                        if let Some(last_stmt) = block.stmts.last() {
                            if let Stmt::Return(return_stmt) = last_stmt {
                                if let Some(expr) = &return_stmt.expr {
                                    // Generate the block with statements except the last return
                                    self.output.push('{');
                                    self.indent();
                                    self.output.push('\n');

                                    // Capture element type before mutable borrows
                                    let element_type_for_destr =
                                        self.current_lambda_element_type.clone();

                                    // Generate destructuring for lambda params (if any)
                                    if has_destructuring {
                                        for (idx, param) in lambda.params.iter().enumerate() {
                                            if param.is_destructuring() {
                                                let temp_name = format!("_param_{}", idx);
                                                self.write_indent();
                                                self.generate_lambda_param_destructuring(
                                                    &param.pattern,
                                                    &temp_name,
                                                    false,
                                                    element_type_for_destr.as_deref(),
                                                )?;
                                                self.output.push('\n');
                                            }
                                        }
                                    }

                                    self.write_indent();

                                    // Generate all statements except the last return
                                    for stmt in &block.stmts[..block.stmts.len() - 1] {
                                        self.generate_stmt(stmt)?;
                                        self.output.push('\n');
                                        self.write_indent();
                                    }

                                    // Generate the return expression without the return keyword
                                    self.generate_expr(expr)?;

                                    self.dedent();
                                    self.output.push('\n');
                                    self.write_indent();
                                    self.output.push('}');
                                } else {
                                    // Empty return, generate unit type
                                    self.output.push_str("()");
                                }
                            } else {
                                // No return statement, generate block as-is
                                self.output.push('{');
                                self.indent();
                                self.output.push('\n');

                                // Capture element type before mutable borrows
                                let element_type_for_destr =
                                    self.current_lambda_element_type.clone();

                                // Generate destructuring for lambda params (if any)
                                if has_destructuring {
                                    for (idx, param) in lambda.params.iter().enumerate() {
                                        if param.is_destructuring() {
                                            let temp_name = format!("_param_{}", idx);
                                            self.write_indent();
                                            self.generate_lambda_param_destructuring(
                                                &param.pattern,
                                                &temp_name,
                                                false,
                                                element_type_for_destr.as_deref(),
                                            )?;
                                            self.output.push('\n');
                                        }
                                    }
                                }

                                self.write_indent();
                                for stmt in &block.stmts {
                                    self.generate_stmt(stmt)?;
                                    self.output.push('\n');
                                    self.write_indent();
                                }
                                self.dedent();
                                self.output.push('}');
                            }
                        } else {
                            // Empty block
                            self.output.push_str("()");
                        }
                    }
                }
            }
            Expr::Fail(expr) => {
                self.write_indent();
                self.output.push_str("return Err(liva_rt::Error::from(");
                self.generate_expr(expr)?;
                self.output.push_str("));\n");
            }
            Expr::MethodCall(method_call) => {
                // TODO: Implement method call code generation (stdlib Phase 2)
                // For now, just generate a placeholder
                self.generate_method_call_expr(method_call)?;
            }
            Expr::Switch(switch_expr) => {
                self.generate_switch_expr(switch_expr)?;
            }
            Expr::MethodRef { object, method } => {
                // Phase 11.4: Generate closure wrapper for method references
                // Utils::validate → |_x| Utils::validate(_x)
                // User::new → |_x| User::new(_x)
                // logger::log → |_x| logger.log(_x)
                let sanitized_obj = self.sanitize_name(object);
                let sanitized_method = self.sanitize_name(method);

                let is_class = object.chars().next().map_or(false, |c| c.is_uppercase());

                if method == "new" {
                    // Constructor reference: User::new → User::new("_x")
                    // Will be wrapped in closure by array method codegen
                    write!(self.output, "{}::new", sanitized_obj).unwrap();
                } else if is_class {
                    // Static method reference: Utils::validate → Utils::validate
                    write!(self.output, "{}::{}", sanitized_obj, sanitized_method).unwrap();
                } else {
                    // Instance method reference: logger::log → |_x| logger.log(_x)
                    // For now, output as a closure that calls the instance method
                    write!(
                        self.output,
                        "|_x| {}.{}(_x)",
                        sanitized_obj, sanitized_method
                    )
                    .unwrap();
                }
            }
            Expr::RustBlock { code } => {
                self.generate_rust_block(code)?;
            }
            Expr::Unwrap(inner) => {
                // Postfix unwrap: expr! → expr.unwrap()
                self.generate_expr(inner)?;
                self.output.push_str(".unwrap()");
            }
            Expr::OptionalChain { object, property } => {
                // Optional chaining: expr?.field → expr.as_ref().map(|__v| __v.field.clone())
                // For string properties, generates .clone() to get owned String
                self.generate_expr(object)?;
                write!(self.output, ".as_ref().map(|__v| __v.{}.clone())", self.sanitize_name(property)).unwrap();
            }
        }
        Ok(())
    }

    /// Generate inline Rust code block.
    /// Extracts `use` statements from the block and hoists them to the top of the file.
    /// The remaining code is emitted as a Rust block expression `{ ... }`.
    fn generate_rust_block(&mut self, code: &str) -> Result<()> {
        let mut remaining_lines = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            // Hoist `use ...;` statements to file top
            if trimmed.starts_with("use ") && trimmed.ends_with(';') {
                let use_stmt = trimmed.to_string();
                if !self.rust_block_uses.contains(&use_stmt) {
                    self.rust_block_uses.push(use_stmt);
                }
            } else {
                remaining_lines.push(line);
            }
        }

        // Emit as a block expression
        self.output.push_str("{\n");
        for line in &remaining_lines {
            self.output.push_str("    ");
            self.output.push_str(line);
            self.output.push('\n');
        }
        self.output.push('}');

        Ok(())
    }

    fn generate_switch_expr(&mut self, switch_expr: &SwitchExpr) -> Result<()> {
        // Check if this is a union type match by examining the first Typed pattern
        let union_type_name = self.detect_union_switch(switch_expr);

        // B96 fix: Detect string-based switch expressions (patterns are string literals)
        let is_string_switch = switch_expr.arms.iter().any(|arm| {
            match &arm.pattern {
                Pattern::Literal(Literal::String(_)) => true,
                Pattern::Or(patterns) => patterns.iter().any(|p| matches!(p, Pattern::Literal(Literal::String(_)))),
                _ => false,
            }
        });

        // B97 fix: When matching on self.field in a &self method, borrow to avoid move
        let needs_ref_b97 = self.in_method && !self.current_method_is_mut
            && matches!(&*switch_expr.discriminant, Expr::Member { object, .. }
                if matches!(object.as_ref(), Expr::Identifier(n) if n == "this"));

        // FIX-3 (ISSUE-003): Borrow discriminant for enum data switches to avoid move
        let is_enum_data_switch_expr = !is_string_switch && switch_expr.arms.iter().any(|arm| {
            matches!(&arm.pattern, Pattern::EnumVariant { bindings, .. } if !bindings.is_empty())
        });
        let needs_ref = needs_ref_b97
            || (is_enum_data_switch_expr
                && matches!(&*switch_expr.discriminant, Expr::Identifier(_)));

        // Generate Rust match expression
        self.output.push_str("match ");
        if needs_ref {
            self.output.push('&');
        }
        self.generate_expr(&switch_expr.discriminant)?;
        if is_string_switch {
            self.output.push_str(".as_str()");
        }
        self.output.push_str(" {");
        self.indent();

        for arm in &switch_expr.arms {
            self.output.push('\n');
            self.write_indent();

            // Generate pattern (with union context if applicable)
            if let Some(ref union_name) = union_type_name {
                self.generate_union_pattern(&arm.pattern, union_name)?;
            } else {
                self.generate_pattern(&arm.pattern)?;
            }

            // Generate guard if present
            if let Some(guard) = &arm.guard {
                self.output.push_str(" if ");
                self.generate_expr(guard)?;
            }

            self.output.push_str(" => ");

            // Check if this pattern has boxed enum bindings that need auto-dereference
            let boxed_bindings = self.get_boxed_pattern_bindings(&arm.pattern);

            // FIX-3: Collect bindings that need cloning when matching by reference
            let ref_clone_bindings = if needs_ref {
                self.get_ref_clone_bindings(&arm.pattern, &boxed_bindings)
            } else {
                Vec::new()
            };

            // GAP-007 fix: Register pattern bindings as class instances for member access
            let registered_bindings = self.register_pattern_bindings(&arm.pattern);

            if !boxed_bindings.is_empty() || !ref_clone_bindings.is_empty() {
                // Wrap body in a block with auto-dereference/clone let statements
                self.output.push('{');
                self.indent();
                for binding in &boxed_bindings {
                    self.output.push('\n');
                    self.write_indent();
                    // When matching by reference (needs_ref), binding is &Box<T>,
                    // clone to Box<T>, then deref to T: `*binding.clone()`
                    if needs_ref {
                        write!(self.output, "let {} = *{}.clone();", binding, binding).unwrap();
                    } else {
                        write!(self.output, "let {} = *{};", binding, binding).unwrap();
                    }
                }
                // FIX-3: Clone ref bindings to get owned values
                for binding in &ref_clone_bindings {
                    self.output.push('\n');
                    self.write_indent();
                    write!(self.output, "let {} = {}.clone();", binding, binding).unwrap();
                }
                match &arm.body {
                    SwitchBody::Expr(expr) => {
                        self.output.push('\n');
                        self.write_indent();
                        if matches!(&**expr, Expr::Literal(Literal::String(_))) {
                            self.generate_expr(expr)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(expr)?;
                        }
                    }
                    SwitchBody::Block(stmts) => {
                        // SH-011 fix: transform last `return expr` into just `expr`
                        // so the switch expression produces a value instead of returning from the function
                        for (i, stmt) in stmts.iter().enumerate() {
                            self.output.push('\n');
                            self.write_indent();
                            if i == stmts.len() - 1 {
                                if let Stmt::Return(ret) = stmt {
                                    if let Some(expr) = &ret.expr {
                                        self.generate_expr(expr)?;
                                        if matches!(expr, Expr::Literal(Literal::String(_))) {
                                            self.output.push_str(".to_string()");
                                        }
                                    }
                                    continue;
                                }
                            }
                            self.generate_stmt(stmt)?;
                        }
                    }
                }
                self.dedent();
                self.output.push('\n');
                self.write_indent();
                self.output.push('}');
            } else {
                // Generate body normally
                match &arm.body {
                    SwitchBody::Expr(expr) => {
                        // Add .to_string() for string literal arms so match returns String
                        if matches!(&**expr, Expr::Literal(Literal::String(_))) {
                            self.generate_expr(expr)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(expr)?;
                        }
                    }
                    SwitchBody::Block(stmts) => {
                        self.output.push('{');
                        self.indent();
                        // SH-011 fix: transform last `return expr` into just `expr`
                        for (i, stmt) in stmts.iter().enumerate() {
                            self.output.push('\n');
                            self.write_indent();
                            if i == stmts.len() - 1 {
                                if let Stmt::Return(ret) = stmt {
                                    if let Some(expr) = &ret.expr {
                                        self.generate_expr(expr)?;
                                        if matches!(expr, Expr::Literal(Literal::String(_))) {
                                            self.output.push_str(".to_string()");
                                        }
                                    }
                                    continue;
                                }
                            }
                            self.generate_stmt(stmt)?;
                        }
                        self.dedent();
                        self.output.push('\n');
                        self.write_indent();
                        self.output.push('}');
                    }
                }
            }

            // GAP-007 fix: Unregister pattern bindings after arm body
            self.unregister_pattern_bindings(&registered_bindings);

            self.output.push(',');
        }

        self.dedent();
        self.output.push('\n');
        self.write_indent();
        self.output.push('}');

        Ok(())
    }

    /// Get the list of binding names in a pattern that correspond to boxed (recursive) enum fields.
    /// These bindings need auto-dereference (`let binding = *binding;`) in the match arm body.
    fn get_boxed_pattern_bindings(&self, pattern: &Pattern) -> Vec<String> {
        if let Pattern::EnumVariant {
            enum_name,
            variant_name,
            bindings,
        } = pattern
        {
            if let Some(variant_map) = self.boxed_enum_fields.get(enum_name) {
                if let Some(boxed_field_names) = variant_map.get(variant_name) {
                    // Get the ordered field names for this variant
                    let field_names = self
                        .enum_variants
                        .get(enum_name)
                        .and_then(|v| v.get(variant_name))
                        .cloned()
                        .unwrap_or_default();

                    let mut result = Vec::new();
                    for (i, binding) in bindings.iter().enumerate() {
                        if binding == "_" {
                            continue; // Skip wildcard bindings
                        }
                        if i < field_names.len() && boxed_field_names.contains(&field_names[i]) {
                            result.push(self.sanitize_name(binding));
                        }
                    }
                    return result;
                }
            }
        }
        Vec::new()
    }

    /// FIX-3: Get bindings that need `.clone()` when matching by reference (`match &expr`).
    /// All non-wildcard, non-Copy bindings that aren't already handled by boxed deref need cloning.
    fn get_ref_clone_bindings(&self, pattern: &Pattern, boxed_bindings: &[String]) -> Vec<String> {
        if let Pattern::EnumVariant { bindings, .. } = pattern {
            // When matching by reference (&e), ALL bindings are references.
            // Both Copy and non-Copy types need to be owned: clone for non-Copy, *deref for Copy.
            // Using .clone() works for both since Clone is supertrait of Copy.
            bindings.iter()
                .filter(|b| *b != "_")
                .map(|b| self.sanitize_name(b))
                .filter(|b| !boxed_bindings.contains(b))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a TypeRef is a Copy type (doesn't need cloning)
    fn is_copy_type(&self, type_ref: &TypeRef) -> bool {
        match type_ref {
            TypeRef::Simple(name) => {
                // Primitive Copy types
                if matches!(name.as_str(),
                    "int" | "float" | "number" | "bool" | "f32" | "f64" | "i32" | "i64" | "u32" | "u64" | "usize"
                ) {
                    return true;
                }
                // FIX-5: Unit enums derive Copy — check if this is a known enum with all-unit variants
                if let Some(variants) = self.enum_variants.get(name.as_str()) {
                    return variants.values().all(|fields| fields.is_empty());
                }
                false
            }
            _ => false,
        }
    }

    /// Detect if this is a union type switch by checking for Typed patterns
    fn detect_union_switch(&mut self, switch_expr: &SwitchExpr) -> Option<String> {
        // Collect types from Typed patterns in order
        let mut pattern_types = Vec::new();

        for arm in &switch_expr.arms {
            if let Pattern::Typed { type_ref, .. } = &arm.pattern {
                let rust_type = self.expand_type_alias(type_ref);
                if !pattern_types.contains(&rust_type) {
                    pattern_types.push(rust_type);
                }
            }
        }

        // If we found typed patterns, construct union name from pattern order
        if pattern_types.len() >= 2 {
            Some(format!("Union_{}", pattern_types.join("_")))
        } else {
            None
        }
    }

    /// Generate a pattern in the context of a union match
    fn generate_union_pattern(&mut self, pattern: &Pattern, union_name: &str) -> Result<()> {
        match pattern {
            Pattern::Typed { name, type_ref } => {
                let rust_type = self.expand_type_alias(type_ref);
                let variant_name = self.type_to_variant_name(&rust_type);
                write!(
                    self.output,
                    "{}::{}({})",
                    union_name,
                    variant_name,
                    self.sanitize_name(name)
                )
                .unwrap();
            }
            Pattern::Wildcard => {
                self.output.push('_');
            }
            _ => {
                // For other patterns, fallback to normal generation
                self.generate_pattern(pattern)?;
            }
        }
        Ok(())
    }

    fn generate_pattern(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Literal(lit) => {
                self.generate_literal(lit)?;
            }
            Pattern::Wildcard => {
                self.output.push('_');
            }
            Pattern::Binding(name) => {
                self.output.push_str(&self.sanitize_name(name));
            }
            Pattern::Range(range) => match (&range.start, &range.end, range.inclusive) {
                (Some(start), Some(end), true) => {
                    self.generate_expr(start)?;
                    self.output.push_str("..=");
                    self.generate_expr(end)?;
                }
                (Some(start), Some(end), false) => {
                    self.generate_expr(start)?;
                    self.output.push_str("..");
                    self.generate_expr(end)?;
                }
                (Some(start), None, _) => {
                    self.generate_expr(start)?;
                    self.output.push_str("..");
                }
                (None, Some(end), true) => {
                    self.output.push_str("..=");
                    self.generate_expr(end)?;
                }
                (None, Some(end), false) => {
                    self.output.push_str("..");
                    self.generate_expr(end)?;
                }
                (None, None, _) => {
                    self.output.push_str("..");
                }
            },
            Pattern::Tuple(patterns) => {
                self.output.push('(');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(pat)?;
                }
                self.output.push(')');
            }
            Pattern::Array(patterns) => {
                self.output.push('[');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(pat)?;
                }
                self.output.push(']');
            }
            Pattern::Or(patterns) => {
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.generate_pattern(pat)?;
                }
            }
            Pattern::Typed { name, type_ref: _ } => {
                // Type pattern for union narrowing: name: type
                // When used outside of union context, just bind the variable
                // (Union context is handled in generate_union_pattern)
                self.output.push_str(&self.sanitize_name(name));
            }
            Pattern::EnumVariant {
                enum_name,
                variant_name,
                bindings,
            } => {
                write!(self.output, "{}::{}", enum_name, variant_name).unwrap();
                if !bindings.is_empty() {
                    self.output.push_str(" { ");
                    // Look up field names for this variant
                    let field_names = self
                        .enum_variants
                        .get(enum_name)
                        .and_then(|v| v.get(variant_name))
                        .cloned()
                        .unwrap_or_default();
                    for (i, binding) in bindings.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        if binding == "_" {
                            // Wildcard binding: field_name: _ or just _
                            if i < field_names.len() {
                                write!(
                                    self.output,
                                    "{}: _",
                                    self.sanitize_name(&field_names[i])
                                )
                                .unwrap();
                            } else {
                                self.output.push('_');
                            }
                        } else if i < field_names.len() && field_names[i] != *binding {
                            // Field name differs from binding: field_name: binding
                            write!(
                                self.output,
                                "{}: {}",
                                self.sanitize_name(&field_names[i]),
                                self.sanitize_name(binding)
                            )
                            .unwrap();
                        } else {
                            self.output.push_str(&self.sanitize_name(binding));
                        }
                    }
                    self.output.push_str(" }");
                }
            }
        }
        Ok(())
    }

    fn generate_call_expr(&mut self, call: &CallExpr) -> Result<()> {
        match call.exec_policy {
            ExecPolicy::Normal => self.generate_normal_call(call),
            ExecPolicy::Async => self.generate_async_call(call),
            ExecPolicy::Par => self.generate_parallel_call(call),
            ExecPolicy::TaskAsync => self.generate_task_call(call, ConcurrencyMode::Async),
            ExecPolicy::TaskPar => self.generate_task_call(call, ConcurrencyMode::Parallel),
        }
    }

    fn generate_normal_call(&mut self, call: &CallExpr) -> Result<()> {
        // Enum variant construction: Shape.Circle(5.0) → Shape::Circle { radius: 5.0 }
        if let Expr::Member { object, property } = call.callee.as_ref() {
            if let Expr::Identifier(enum_name) = object.as_ref() {
                if let Some(variants) = self.enum_variants.get(enum_name).cloned() {
                    if let Some(field_names) = variants.get(property) {
                        // Check if this variant has any boxed (recursive) fields
                        let boxed_fields = self
                            .boxed_enum_fields
                            .get(enum_name)
                            .and_then(|v| v.get(property.as_str()))
                            .cloned()
                            .unwrap_or_default();

                        // Check optional fields for this variant
                        let variant_key = format!("{}::{}", enum_name, property);
                        let variant_optionals = self.enum_variant_optionals.get(&variant_key).cloned();

                        write!(self.output, "{}::{}", enum_name, property).unwrap();
                        if !field_names.is_empty() {
                            self.output.push_str(" { ");
                            for (i, (field_name, arg)) in
                                field_names.iter().zip(call.args.iter()).enumerate()
                            {
                                if i > 0 {
                                    self.output.push_str(", ");
                                }
                                write!(self.output, "{}: ", self.sanitize_name(field_name))
                                    .unwrap();

                                // Check if field is optional and arg is not null
                                let is_opt_field = variant_optionals.as_ref()
                                    .map_or(false, |opts| i < opts.len() && opts[i]);
                                let is_null = matches!(arg, Expr::Literal(Literal::Null));
                                let already_optional = self.init_is_already_optional(arg);
                                let wrap_some = is_opt_field && !is_null && !already_optional;

                                if wrap_some {
                                    self.output.push_str("Some(");
                                }

                                // Auto-boxing: recursive fields get wrapped in Box::new()
                                if boxed_fields.contains(field_name) {
                                    self.output.push_str("Box::new(");
                                    self.generate_expr(arg)?;
                                    // SH-010 fix: clone identifiers being boxed to avoid
                                    // move errors when the same var is used elsewhere
                                    if matches!(arg, Expr::Identifier(_)) {
                                        self.output.push_str(".clone()");
                                    }
                                    self.output.push(')');
                                } else {
                                    self.generate_expr(arg)?;
                                }
                                // SH-009 fix: string literals need .to_string() for String-typed enum fields
                                if matches!(arg, Expr::Literal(Literal::String(_))) {
                                    self.output.push_str(".to_string()");
                                }

                                if wrap_some {
                                    self.output.push(')');
                                }
                            }
                            self.output.push_str(" }");
                        }
                        return Ok(());
                    }
                }
            }
        }

        if let Expr::Identifier(name) = call.callee.as_ref() {
            // ─── liva/test virtual library ───────────────────────────
            // describe("name", () => { ... }) → mod test_name { use super::*; ... }
            if name == "describe" {
                return self.generate_test_describe(call);
            }
            // test("name", () => { ... }) → #[test] fn test_name() { ... }
            if name == "test" {
                return self.generate_test_case(call);
            }
            // expect(value) → handled as part of method chain (expect(x).toBe(y))
            // The actual codegen happens in generate_method_call_expr
            if name == "expect" {
                // expect() by itself is a no-op; it's always used with a matcher
                // If called standalone (no method chain), treat as no-op
                self.output.push_str("/* expect() */()");
                return Ok(());
            }
            // beforeEach/afterEach/beforeAll/afterAll → setup/teardown helpers
            if name == "beforeEach"
                || name == "afterEach"
                || name == "beforeAll"
                || name == "afterAll"
            {
                return self.generate_test_lifecycle(name, call);
            }
            // ─────────────────────────────────────────────────────────

            // Handle parseInt(str) -> (i32, Option<Error>)
            if name == "parseInt" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "parseInt requires 1 argument",
                        "parseInt(str) takes exactly one string argument",
                    )));
                }
                self.output.push_str("match ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str(".parse::<i32>() { Ok(v) => (v, None), Err(e) => (0, Some(liva_rt::Error::from(e.to_string()))) }");
                return Ok(());
            }

            // Handle parseFloat(str) -> (f64, Option<Error>)
            if name == "parseFloat" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "parseFloat requires 1 argument",
                        "parseFloat(str) takes exactly one string argument",
                    )));
                }
                self.output.push_str("match ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str(".parse::<f64>() { Ok(v) => (v, None), Err(e) => (0.0_f64, Some(liva_rt::Error::from(e.to_string()))) }");
                return Ok(());
            }

            // Handle toString(value) -> String
            if name == "toString" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "toString requires 1 argument",
                        "toString(value) takes exactly one argument",
                    )));
                }
                self.output.push_str("format!(\"{}\", ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str(")");
                return Ok(());
            }

            // Handle readLine() -> String (read from stdin)
            if name == "readLine" {
                self.output.push_str("{ let mut input = String::new(); std::io::stdin().read_line(&mut input).expect(\"Failed to read line\"); input.trim().to_string() }");
                return Ok(());
            }

            // Handle prompt(message) -> String (display message and read input)
            if name == "prompt" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "prompt requires 1 argument",
                        "prompt(message) takes exactly one string argument",
                    )));
                }
                self.output.push_str("{ print!(\"{}\", ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str("); std::io::stdout().flush().expect(\"Failed to flush stdout\"); let mut input = String::new(); std::io::stdin().read_line(&mut input).expect(\"Failed to read line\"); input.trim().to_string() }");
                return Ok(());
            }

            if name == "print" {
                if call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    self.output.push_str("println!(\"");
                    for arg in call.args.iter() {
                        // Check if argument is an array type — use {:?} (Debug) for arrays
                        let is_array_arg = match arg {
                            Expr::Identifier(var_name) => {
                                let sanitized = self.sanitize_name(var_name);
                                self.array_vars.contains(&sanitized)
                                    || self.array_vars.contains(var_name)
                            }
                            Expr::ArrayLiteral(_) => true,
                            _ => false,
                        };
                        if is_array_arg {
                            self.output.push_str("{:?}");
                        } else {
                            // print() uses Display format {} for clean, user-facing output
                            self.output.push_str("{}");
                        }
                    }
                    self.output.push_str("\", ");
                    for (i, arg) in call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        // Phase 3.5: If arg is an error binding variable, use Display (error trace)
                        if let Expr::Identifier(name) = arg {
                            let sanitized = self.sanitize_name(name);
                            if self.error_binding_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.as_ref().map(|e| format!(\"{{}}\", e)).unwrap_or_default()",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
                return Ok(());
            }

            // Check if this is a constructor call (starts with uppercase)
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                // Assume it's a constructor call like ClassName(args...)
                let constructor_optionals = self.class_constructor_optionals.get(name).cloned();
                write!(self.output, "{}::new(", name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Check if this arg position corresponds to an optional field
                    let is_optional_param = constructor_optionals.as_ref()
                        .map_or(false, |opts| i < opts.len() && opts[i]);
                    let is_null_arg = matches!(arg, Expr::Literal(Literal::Null));
                    let already_optional = self.init_is_already_optional(arg);
                    let wrap_some = is_optional_param && !is_null_arg && !already_optional;

                    if wrap_some {
                        self.output.push_str("Some(");
                    }

                    // Add .to_string() for string literals (hack for constructor parameters)
                    if let Expr::Literal(Literal::String(_)) = arg {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else if let Expr::Identifier(var_name) = arg {
                        // B17 fix: Clone non-Copy variables when passing to constructors
                        let sanitized = self.sanitize_name(var_name);
                        let is_string_var = self.string_vars.contains(&sanitized);
                        let is_class_instance = self.class_instance_vars.contains(&sanitized);
                        let is_map = self.map_vars.contains(&sanitized);
                        let is_array = self.array_vars.contains(&sanitized);
                        let is_json = self.json_value_vars.contains(&sanitized);
                        if is_string_var || is_class_instance || is_map || is_array || is_json {
                            self.generate_expr(arg)?;
                            self.output.push_str(".clone()");
                        } else {
                            self.generate_expr(arg)?;
                        }
                    } else if let Expr::Index { object, .. } = arg {
                        // B35 fix: Clone array index access when passing to constructors
                        self.generate_expr(arg)?;
                        if let Expr::Identifier(var_name) = object.as_ref() {
                            let sanitized = self.sanitize_name(var_name);
                            if self.array_vars.contains(&sanitized)
                                || self.class_instance_vars.contains(&sanitized)
                                || self.string_vars.contains(&sanitized)
                                || self.map_vars.contains(&sanitized)
                                || self.json_value_vars.contains(&sanitized)
                            {
                                if !self.output.ends_with(".clone()") {
                                    self.output.push_str(".clone()");
                                }
                            }
                        }
                    } else if self.expr_is_self_field(arg) {
                        // B44 fix: Clone non-Copy self fields when passing to constructors
                        self.generate_expr(arg)?;
                        self.output.push_str(".clone()");
                    } else {
                        self.generate_expr(arg)?;
                    }

                    if wrap_some {
                        self.output.push(')');
                    }
                }
                self.output.push(')');
                return Ok(());
            }
        }

        // Check if this is a call to a user-defined async function
        let is_async_call = if let Expr::Identifier(name) = call.callee.as_ref() {
            self.async_functions.contains(name)
        } else {
            false
        };

        self.generate_expr(&call.callee)?;

        // Add type arguments if present (turbofish syntax)
        if !call.type_args.is_empty() {
            self.output.push_str("::<");
            for (i, type_arg) in call.type_args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                write!(self.output, "{}", type_arg.to_rust_type()).unwrap();
            }
            self.output.push('>');
        }

        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            // Convert string literals to String automatically
            if let Expr::Literal(Literal::String(_)) = arg {
                self.generate_expr(arg)?;
                self.output.push_str(".to_string()");
            } else if let Expr::Identifier(name) = arg {
                // B17 fix: Clone non-Copy variables when passing to functions
                // to avoid ownership issues (Rust moves String/struct/Vec/HashMap by default)
                // FIX-4 (ISSUE-004): Broadened to clone ANY non-Copy variable, not just
                // those tracked in specific HashSets. This prevents move errors for
                // enum types, cross-module classes, and any other non-Copy types.
                let sanitized = self.sanitize_name(name);
                let is_known_copy = self.is_copy_var(&sanitized);
                if !is_known_copy
                    && (self.class_instance_vars.contains(&sanitized)
                        || self.string_vars.contains(&sanitized)
                        || self.map_vars.contains(&sanitized)
                        || self.array_vars.contains(&sanitized)
                        || self.json_value_vars.contains(&sanitized)
                        || self.var_types.contains_key(&sanitized)
                        || self.mutated_vars.contains(&sanitized)
                        || self.option_value_vars.contains(&sanitized))
                {
                    self.generate_expr(arg)?;
                    self.output.push_str(".clone()");
                } else if !is_known_copy && self.looks_like_non_copy_var(&sanitized) {
                    // FIX-4: Catch-all for variables not tracked in any set
                    // but that are likely non-Copy (not a known primitive identifier)
                    self.generate_expr(arg)?;
                    self.output.push_str(".clone()");
                } else {
                    self.generate_expr(arg)?;
                }
            } else if let Expr::Index { object, .. } = arg {
                // B35 fix: Clone array index access when passing to functions
                // arr[i] returns a reference in Rust — need .clone() for non-Copy types
                self.generate_expr(arg)?;
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if self.array_vars.contains(&sanitized)
                        || self.class_instance_vars.contains(&sanitized)
                        || self.string_vars.contains(&sanitized)
                        || self.map_vars.contains(&sanitized)
                        || self.json_value_vars.contains(&sanitized)
                    {
                        // Only add .clone() if not already added by generate_expr
                        if !self.output.ends_with(".clone()") {
                            self.output.push_str(".clone()");
                        }
                    }
                }
            } else if self.expr_is_self_field(arg) {
                // B44 fix: Clone non-Copy self fields when passing to functions
                // In &self methods, self.field can't be moved — needs .clone()
                self.generate_expr(arg)?;
                self.output.push_str(".clone()");
            } else {
                self.generate_expr(arg)?;
            }
        }

        // Inject default parameter values for missing arguments
        if let Expr::Identifier(func_name) = call.callee.as_ref() {
            if let Some(defaults) = self.function_defaults.get(func_name).cloned() {
                let num_provided = call.args.len();
                for (param_idx, default_expr) in &defaults {
                    if *param_idx >= num_provided {
                        if num_provided > 0 || *param_idx > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(&default_expr)?;
                        // Add .to_string() for string literal defaults
                        if matches!(default_expr, Expr::Literal(Literal::String(_))) {
                            self.output.push_str(".to_string()");
                        }
                    }
                }
            }
        }

        self.output.push(')');

        // Add .await for async function calls
        if is_async_call {
            self.output.push_str(".await");
        }

        Ok(())
    }

    /// Check if an expression is an async or par call (returns Task)
    fn is_task_expr(&self, expr: &Expr) -> Option<ExecPolicy> {
        match expr {
            Expr::Call(call) => match call.exec_policy {
                ExecPolicy::Async | ExecPolicy::Par => Some(call.exec_policy.clone()),
                ExecPolicy::TaskAsync => Some(ExecPolicy::TaskAsync),
                ExecPolicy::TaskPar => Some(ExecPolicy::TaskPar),
                _ => None,
            },
            _ => None,
        }
    }

    /// Check if an expression is an explicit `await taskVar` for a pending task
    fn is_explicit_await_of_task(&self, expr: &Expr, task_var_name: &str) -> bool {
        if let Expr::Unary {
            op: crate::ast::UnOp::Await,
            operand,
        } = expr
        {
            if let Expr::Identifier(name) = operand.as_ref() {
                return self.sanitize_name(name) == task_var_name;
            }
        }
        false
    }

    /// Check if an expression is `await taskVar` where taskVar is a pending task. Returns the task name.
    fn is_await_of_pending_task(&self, expr: &Expr) -> Option<String> {
        if let Expr::Unary {
            op: crate::ast::UnOp::Await,
            operand,
        } = expr
        {
            if let Expr::Identifier(name) = operand.as_ref() {
                let sanitized = self.sanitize_name(name);
                if self.pending_tasks.contains_key(&sanitized) {
                    return Some(sanitized);
                }
            }
        }
        None
    }

    /// Check if an expression uses a variable (recursively)
    fn expr_uses_var(&self, expr: &Expr, var_name: &str) -> bool {
        match expr {
            Expr::Identifier(name) => {
                let sanitized = if self.in_method && name == "this" {
                    "self"
                } else {
                    name
                };
                self.sanitize_name(sanitized) == var_name
            }
            Expr::Binary { left, right, .. } => {
                self.expr_uses_var(left, var_name) || self.expr_uses_var(right, var_name)
            }
            Expr::Unary { operand, .. } => self.expr_uses_var(operand, var_name),
            Expr::Call(call) => {
                self.expr_uses_var(&call.callee, var_name)
                    || call
                        .args
                        .iter()
                        .any(|arg| self.expr_uses_var(arg, var_name))
            }
            Expr::Member { object, .. } => self.expr_uses_var(object, var_name),
            Expr::Index { object, index } => {
                self.expr_uses_var(object, var_name) || self.expr_uses_var(index, var_name)
            }
            Expr::StringTemplate { parts } => parts.iter().any(|p| {
                if let crate::ast::StringTemplatePart::Expr(e) = p {
                    self.expr_uses_var(e, var_name)
                } else {
                    false
                }
            }),
            Expr::MethodCall(mc) => {
                self.expr_uses_var(&mc.object, var_name)
                    || mc.args.iter().any(|arg| self.expr_uses_var(arg, var_name))
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.expr_uses_var(condition, var_name)
                    || self.expr_uses_var(then_expr, var_name)
                    || self.expr_uses_var(else_expr, var_name)
            }
            _ => false,
        }
    }

    /// Check if a statement uses a pending task variable
    fn stmt_uses_pending_task(&self, stmt: &Stmt) -> Option<String> {
        for (var_name, task_info) in &self.pending_tasks {
            if task_info.awaited {
                continue; // Already awaited
            }

            // For error binding, check if ANY of the binding variables is used
            let check_vars: Vec<&String> = if task_info.is_error_binding {
                task_info.binding_names.iter().collect()
            } else {
                vec![var_name]
            };

            let uses_var = match stmt {
                Stmt::Expr(expr_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&expr_stmt.expr, v)),
                Stmt::If(if_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&if_stmt.condition, v)),
                Stmt::Return(ret_stmt) => ret_stmt.expr.as_ref().map_or(false, |e| {
                    check_vars.iter().any(|v| self.expr_uses_var(e, v))
                }),
                Stmt::While(while_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&while_stmt.condition, v)),
                Stmt::Assign(assign) => check_vars.iter().any(|v| {
                    self.expr_uses_var(&assign.target, v) || self.expr_uses_var(&assign.value, v)
                }),
                Stmt::VarDecl(var) => {
                    // Skip if init is an explicit await of this pending task
                    // (the VarDecl handler will generate the await+destructure directly)
                    if self.is_explicit_await_of_task(&var.init, var_name) {
                        false
                    } else {
                        check_vars.iter().any(|v| self.expr_uses_var(&var.init, v))
                    }
                }
                _ => false,
            };

            if uses_var {
                return Some(var_name.clone());
            }
        }
        None
    }

    /// Phase 4: Get ALL pending tasks used in a statement (for join combining)
    fn stmt_uses_pending_tasks(&self, stmt: &Stmt) -> Vec<String> {
        let mut used_tasks = Vec::new();

        for (var_name, task_info) in &self.pending_tasks {
            if task_info.awaited {
                continue; // Already awaited
            }

            // For error binding, check if ANY of the binding variables is used
            let check_vars: Vec<&String> = if task_info.is_error_binding {
                task_info.binding_names.iter().collect()
            } else {
                vec![var_name]
            };

            let uses_var = match stmt {
                Stmt::Expr(expr_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&expr_stmt.expr, v)),
                Stmt::If(if_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&if_stmt.condition, v)),
                Stmt::Return(ret_stmt) => ret_stmt.expr.as_ref().map_or(false, |e| {
                    check_vars.iter().any(|v| self.expr_uses_var(e, v))
                }),
                Stmt::While(while_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&while_stmt.condition, v)),
                Stmt::Assign(assign) => check_vars.iter().any(|v| {
                    self.expr_uses_var(&assign.target, v) || self.expr_uses_var(&assign.value, v)
                }),
                Stmt::VarDecl(var) => {
                    // Skip if init is an explicit await of this pending task
                    // (the VarDecl handler will generate the await+destructure directly)
                    if self.is_explicit_await_of_task(&var.init, var_name) {
                        false
                    } else {
                        check_vars.iter().any(|v| self.expr_uses_var(&var.init, v))
                    }
                }
                _ => false,
            };

            if uses_var {
                used_tasks.push(var_name.clone());
            }
        }

        used_tasks
    }

    /// Phase 4.2: Check for dead tasks (never awaited) and emit warnings
    fn check_dead_tasks(&self) {
        for (var_name, task_info) in &self.pending_tasks {
            if !task_info.awaited {
                eprintln!(
                    "⚠️  Warning: Task '{}' was created but never used",
                    var_name
                );
                eprintln!("   → Consider removing the task creation or using the variable");
                eprintln!("   → This creates an async/parallel task that does nothing");
            }
        }
    }

    /// Phase 4: Generate tokio::join! for multiple pending tasks (optimization)
    fn generate_tasks_join(&mut self, task_vars: &[String]) -> Result<()> {
        if task_vars.is_empty() {
            return Ok(());
        }

        // Collect task infos and skip already awaited tasks
        let mut tasks_to_join: Vec<(String, TaskInfo)> = Vec::new();
        for var_name in task_vars {
            if let Some(task_info) = self.pending_tasks.get(var_name) {
                if !task_info.awaited {
                    tasks_to_join.push((var_name.clone(), task_info.clone()));
                }
            }
        }

        if tasks_to_join.is_empty() {
            return Ok(());
        }

        // If only one task, use regular await
        if tasks_to_join.len() == 1 {
            return self.generate_task_await(&tasks_to_join[0].0);
        }

        // Generate tokio::join! for multiple tasks
        self.write_indent();
        self.output.push_str("let (");

        // Generate tuple of result variables
        for (i, (var_name, task_info)) in tasks_to_join.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }

            if task_info.is_error_binding {
                // Error binding: (value, err)
                self.output.push('(');
                for (j, binding_name) in task_info.binding_names.iter().enumerate() {
                    if j > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(binding_name);
                }
                self.output.push(')');
            } else {
                // Simple binding: value
                self.output.push_str(var_name);
            }
        }

        self.output.push_str(") = ");

        // Check if all tasks are the same type (all async or all par)
        let all_same_type = tasks_to_join
            .iter()
            .all(|(_, info)| info.exec_policy == tasks_to_join[0].1.exec_policy);

        if !all_same_type {
            // Mixed async/par - fall back to sequential awaits
            // Drop the "let (" we just wrote
            let output_len = self.output.len();
            let last_line_start = self.output[..output_len]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            self.output.truncate(last_line_start);

            // Generate sequential awaits instead
            for (var_name, _) in &tasks_to_join {
                self.generate_task_await(var_name)?;
            }
            return Ok(());
        }

        // Generate tokio::join! macro call
        self.output.push_str("tokio::join!(");

        for (i, (var_name, task_info)) in tasks_to_join.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }

            let task_var_name = format!("{}_task", var_name);

            if task_info.is_error_binding {
                // Error binding: async { match task.await.unwrap() { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) } }
                write!(self.output, "async {{ match {}.await.unwrap() {{ Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) }} }}",
                    task_var_name).unwrap();
            } else {
                // Simple binding: async { task.await.unwrap() }
                write!(self.output, "async {{ {}.await.unwrap() }}", task_var_name).unwrap();
            }
        }

        self.output.push_str(");\n");

        // Mark all tasks as awaited
        for (var_name, _) in &tasks_to_join {
            if let Some(task_info) = self.pending_tasks.get_mut(var_name) {
                task_info.awaited = true;
            }
        }

        Ok(())
    }

    /// Generate the await code for a pending task (Phase 2: Lazy await)
    fn generate_task_await(&mut self, var_name: &str) -> Result<()> {
        let task_info = self.pending_tasks.get(var_name).cloned();
        if task_info.is_none() {
            return Ok(()); // Not a task or already awaited
        }

        let task_info = task_info.unwrap();
        if task_info.awaited {
            return Ok(()); // Already awaited
        }

        let task_var_name = format!("{}_task", var_name);

        self.write_indent();

        if task_info.is_error_binding {
            // Error binding: Check if the function returns a tuple directly or Result
            write!(self.output, "let (").unwrap();
            for (i, binding_name) in task_info.binding_names.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                write!(self.output, "{}", binding_name).unwrap();
            }

            if task_info.returns_tuple {
                // Function returns (Option<T>, String) or (T, String) directly - destructure
                self.output.push_str(") = ");

                if task_info.is_http_call {
                    // HTTP calls return (Option<T>, String), unwrap the Option too
                    write!(self.output, "{{ let (opt, err) = {}.await.unwrap(); (opt.unwrap_or_default(), err) }};\n", task_var_name).unwrap();
                } else {
                    // Other tuple-returning functions return (T, String) directly
                    write!(self.output, "{}.await.unwrap();\n", task_var_name).unwrap();
                }
            } else {
                // Function returns Result - match and convert
                self.output.push_str(") = match ");
                write!(self.output, "{}.await.unwrap()", task_var_name).unwrap();
                self.output.push_str(
                    " { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n",
                );
            }
        } else {
            // Simple binding: let var_name = var_name_task.await.unwrap();
            write!(
                self.output,
                "let mut {} = {}.await.unwrap();\n",
                var_name, task_var_name
            )
            .unwrap();
        }

        // Mark as awaited
        self.pending_tasks.get_mut(var_name).unwrap().awaited = true;

        Ok(())
    }

    /// Generate code for method calls (stdlib Phase 2 - array methods)
    fn generate_method_call_expr(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        use crate::ast::ArrayAdapter;

        // Server request param interception: req.params.get("key") → __params.get(&key).cloned().unwrap_or_default()
        if self.server_request_param.is_some() {
            if method_call.method == "get" {
                if let Expr::Member { object, property } = method_call.object.as_ref() {
                    if property == "params" {
                        if let Expr::Identifier(name) = object.as_ref() {
                            if Some(name.as_str()) == self.server_request_param.as_deref() {
                                self.output.push_str("__params.get(&");
                                if let Some(arg) = method_call.args.first() {
                                    self.generate_expr(arg)?;
                                }
                                self.output.push_str(".to_string()).cloned().unwrap_or_default()");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        // ─── liva/test: expect(x).toBe(y) chain ─────────────────────
        if let Some(result) = self.try_generate_expect_chain(method_call)? {
            self.output.push_str(&result);
            return Ok(());
        }
        // ─────────────────────────────────────────────────────────────

        // Handle .length() as method call → .len() as i32
        if method_call.method == "length" && method_call.args.is_empty() {
            self.output.push('(');
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".len() as i32)");
            return Ok(());
        }

        // Check if this is a Math function call (Math.sqrt, Math.pow, etc.)
        if let Expr::Identifier(name) = method_call.object.as_ref() {
            if name == "Math" {
                return self.generate_math_function_call(method_call);
            }

            // Check if this is a console function call (console.log, console.error, etc.)
            if name == "console" {
                return self.generate_console_function_call(method_call);
            }

            // Check if this is a JSON function call (JSON.parse, JSON.stringify)
            if name == "JSON" {
                return self.generate_json_function_call(method_call);
            }

            // Check if this is a File function call (File.read, File.write, etc.)
            if name == "File" {
                return self.generate_file_function_call(method_call);
            }

            // Check if this is an HTTP function call (HTTP.get, HTTP.post, etc.)
            if name == "HTTP" || name == "Http" {
                return self.generate_http_function_call(method_call);
            }

            // Check if this is a Dir function call (Dir.list, Dir.isDir)
            if name == "Dir" {
                return self.generate_dir_function_call(method_call);
            }

            // Check if this is a Sys function call (Sys.args, Sys.env, etc.)
            if name == "Sys" {
                return self.generate_sys_function_call(method_call);
            }

            // Check if this is a Log function call (Log.info, Log.warn, etc.)
            if name == "Log" {
                return self.generate_log_function_call(method_call);
            }

            // Check if this is a Config function call (Config.load, Config.get, etc.)
            if name == "Config" {
                return self.generate_config_function_call(method_call);
            }

            // Check if this is a Regex function call (Regex.test, Regex.match, etc.)
            if name == "Regex" {
                return self.generate_regex_function_call(method_call);
            }

            // Check if this is a Date constructor call (Date.now, Date.new, etc.)
            if name == "Date" {
                return self.generate_date_function_call(method_call);
            }

            // Check if this is a CSV function call (CSV.read, CSV.write, etc.)
            if name == "CSV" {
                return self.generate_csv_function_call(method_call);
            }

            // Check if this is a Random function call (Random.nextInt, etc.)
            if name == "Random" {
                return self.generate_random_function_call(method_call);
            }

            // Check if this is a Crypto function call (Crypto.sha256, etc.)
            if name == "Crypto" {
                return self.generate_crypto_function_call(method_call);
            }

            // Check if this is a Process function call (Process.exec, etc.)
            if name == "Process" {
                return self.generate_process_function_call(method_call);
            }

            // Check if this is a Server function call (Server.create, etc.)
            if name == "Server" {
                return self.generate_server_function_call(method_call);
            }

            // Check if this is a Response constructor (Response.json, Response.text, etc.)
            if name == "Response" {
                return self.generate_response_function_call(method_call);
            }

            // Check if this is a DB function call (DB.open, DB.exec, etc.)
            if name == "DB" {
                return self.generate_db_function_call(method_call);
            }

            // Bug #40: Check if this is a module alias call (import * as alias from "...")
            // Generate module::function() instead of alias.function()
            if let Some(module_name) = self.module_aliases.get(name).cloned() {
                return self.generate_module_function_call(&module_name, method_call);
            }

            // Enum variant construction: Shape.Circle(5.0) → Shape::Circle { radius: 5.0 }
            if let Some(variants) = self.enum_variants.get(name).cloned() {
                if let Some(field_names) = variants.get(&method_call.method) {
                    // Check if this variant has any boxed (recursive) fields
                    let boxed_fields = self
                        .boxed_enum_fields
                        .get(name)
                        .and_then(|v| v.get(&method_call.method))
                        .cloned()
                        .unwrap_or_default();

                    // Check optional fields for this variant
                    let variant_key = format!("{}::{}", name, method_call.method);
                    let variant_optionals = self.enum_variant_optionals.get(&variant_key).cloned();

                    write!(self.output, "{}::{}", name, method_call.method).unwrap();
                    if !field_names.is_empty() {
                        self.output.push_str(" { ");
                        for (i, (field_name, arg)) in
                            field_names.iter().zip(method_call.args.iter()).enumerate()
                        {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            write!(self.output, "{}: ", self.sanitize_name(field_name)).unwrap();

                            // Check if field is optional and arg is not null
                            let is_opt_field = variant_optionals.as_ref()
                                .map_or(false, |opts| i < opts.len() && opts[i]);
                            let is_null = matches!(arg, Expr::Literal(Literal::Null));
                            let already_optional = self.init_is_already_optional(arg);
                            let wrap_some = is_opt_field && !is_null && !already_optional;

                            if wrap_some {
                                self.output.push_str("Some(");
                            }

                            // Auto-boxing: recursive fields get wrapped in Box::new()
                            if boxed_fields.contains(field_name) {
                                self.output.push_str("Box::new(");
                                self.generate_expr(arg)?;
                                // SH-010 fix: clone identifiers being boxed to avoid
                                // move errors when the same var is used elsewhere
                                if matches!(arg, Expr::Identifier(_)) {
                                    self.output.push_str(".clone()");
                                }
                                self.output.push(')');
                            } else {
                                self.generate_expr(arg)?;
                            }
                            // SH-009 fix: string literals need .to_string() for String-typed enum fields
                            if matches!(arg, Expr::Literal(Literal::String(_))) {
                                self.output.push_str(".to_string()");
                            }

                            if wrap_some {
                                self.output.push(')');
                            }
                        }
                        self.output.push_str(" }");
                    }
                    return Ok(());
                }
            }
        }

        // Check if this is a Date instance method call (format, add, diff, toString)
        {
            let is_date_var = if let Expr::Identifier(name) = method_call.object.as_ref() {
                self.date_vars.contains(&self.sanitize_name(name))
            } else {
                false
            };

            if is_date_var {
                let is_date_method = matches!(
                    method_call.method.as_str(),
                    "format" | "add" | "diff" | "toString"
                );
                if is_date_method {
                    return self.generate_date_method_call(method_call);
                }
            }
        }

        // Check if this is a Server instance method call (get, post, put, delete, listen, use)
        {
            let is_server_var = if let Expr::Identifier(name) = method_call.object.as_ref() {
                self.server_vars.contains(&self.sanitize_name(name))
            } else {
                false
            };

            if is_server_var {
                let is_server_method = matches!(
                    method_call.method.as_str(),
                    "get" | "post" | "put" | "delete" | "listen" | "use"
                );
                if is_server_method {
                    return self.generate_server_method_call(method_call);
                }
            }
        }

        // Check if this is a Map method call (get, set, has, delete, keys, values, entries, clear)
        {
            let is_map_var = if let Expr::Identifier(name) = method_call.object.as_ref() {
                self.map_vars.contains(&self.sanitize_name(name))
            } else if let Expr::Member { object, property } = method_call.object.as_ref() {
                // Bug #76 fix: Also check this._field for Map-typed class fields
                if matches!(object.as_ref(), Expr::Identifier(name) if name == "this" || name == "self") {
                    self.map_vars.contains(&self.sanitize_name(property))
                } else if let Expr::Member { property: inner_prop, .. } = object.as_ref() {
                    // Deep member access: this._ctx.varTypes.has() — check innermost property
                    self.map_vars.contains(&self.sanitize_name(inner_prop))
                        && self.map_vars.contains(&self.sanitize_name(property))
                        || self.map_vars.contains(&self.sanitize_name(property))
                } else {
                    false
                }
            } else {
                false
            };

            let is_map_method = matches!(
                method_call.method.as_str(),
                "get" | "set" | "has" | "delete" | "keys" | "values" | "entries" | "clear" | "forEach"
            );

            if is_map_var && is_map_method {
                return self.generate_map_method_call(method_call);
            }
        }

        // Check if this is a Set method call (add, has, delete, clear, values, forEach, union, intersection, difference)
        {
            let is_set_var = if let Expr::Identifier(name) = method_call.object.as_ref() {
                self.set_vars.contains(&self.sanitize_name(name))
            } else if let Expr::Member { object, property } = method_call.object.as_ref() {
                // Bug #76 fix: Also check this._field for Set-typed class fields
                if matches!(object.as_ref(), Expr::Identifier(name) if name == "this" || name == "self") {
                    self.set_vars.contains(&self.sanitize_name(property))
                } else {
                    false
                }
            } else {
                false
            };

            let is_set_method = matches!(
                method_call.method.as_str(),
                "add" | "has" | "delete" | "clear" | "values" | "forEach" | "union" | "intersection" | "difference"
            );

            if is_set_var && is_set_method {
                return self.generate_set_method_call(method_call);
            }
        }

        // Check if this is a string method (no adapter means it's not an array method)
        // Special case: indexOf can be both string and array method
        // We detect string indexOf if:
        // 1. The argument is a string literal, OR
        // 2. The argument is a known string variable, OR
        // 3. The object is a member access on 'this' (class field likely string)
        let is_string_indexof = method_call.method == "indexOf"
            && !method_call.args.is_empty()
            && {
                // Check if argument is string literal
                let arg_is_string_lit =
                    matches!(&method_call.args[0], Expr::Literal(Literal::String(_)));
                // Check if argument is a known string variable
                let arg_is_string_var = if let Expr::Identifier(var_name) = &method_call.args[0] {
                    self.string_vars.contains(&self.sanitize_name(var_name))
                } else {
                    false
                };
                // Check if object is this.field (member access on this/self)
                let object_is_this_field = if let Expr::Member { object, .. } =
                    method_call.object.as_ref()
                {
                    matches!(object.as_ref(), Expr::Identifier(name) if name == "this" || name == "self")
                } else {
                    false
                };

                arg_is_string_lit || arg_is_string_var || object_is_this_field
            };

        let is_string_method = (matches!(method_call.adapter, ArrayAdapter::Seq)
            && matches!(
                method_call.method.as_str(),
                "split"
                    | "replace"
                    | "replaceAll"
                    | "toUpperCase"
                    | "toLowerCase"
                    | "trim"
                    | "trimStart"
                    | "trimEnd"
                    | "startsWith"
                    | "endsWith"
                    | "substring"
                    | "charAt"
                    | "contains"
                    | "lastIndexOf"
                    | "padStart"
                    | "padEnd"
                    | "repeat"
                    | "chars"
                    | "capitalize"
                    | "isBlank"
                    | "isEmpty"
                    | "reverse"
                    | "truncate"
                    | "countMatches"
                    | "removePrefix"
                    | "removeSuffix"
                    | "toInt"
                    | "toFloat"
            ))
            || is_string_indexof;

        if is_string_method {
            // Handle string methods
            return self.generate_string_method_call(method_call);
        }

        // Handle [T].concat(other) — array concatenation
        // Rust Vec doesn't have .concat() with the same semantics as Liva
        // Translate to: { let mut __v = obj; __v.extend(other); __v }
        if method_call.method == "concat" && !method_call.args.is_empty() {
            self.output.push_str("{ let mut __v = ");
            self.generate_expr(&method_call.object)?;
            self.output.push_str("; __v.extend(");
            self.generate_expr(&method_call.args[0])?;
            self.output.push_str("); __v }");
            return Ok(());
        }

        // Handle [string].join(separator) — array method that produces a string
        if method_call.method == "join" {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".join(");
            if !method_call.args.is_empty() {
                self.generate_expr(&method_call.args[0])?;
            } else {
                self.output.push_str("\"\"");
            }
            self.output.push(')');
            return Ok(());
        }

        // Guard: check if the object is a known class instance
        // If so, skip array-specific handlers to avoid intercepting user class methods
        // (e.g., Color.sum() should call the class method, not treat it as array sum)
        let object_is_class_instance =
            if let Expr::Identifier(var_name) = method_call.object.as_ref() {
                let sanitized = self.sanitize_name(var_name);
                self.class_instance_vars.contains(&sanitized)
            } else {
                false
            };

        // Handle slice() — works for both strings and arrays
        // String: obj[start..end].to_string()
        // Array:  obj[start..end].to_vec()
        if method_call.method == "slice" && !method_call.args.is_empty() {
            let is_string = if let Expr::Identifier(var_name) = method_call.object.as_ref() {
                self.string_vars.contains(&self.sanitize_name(var_name))
            } else if let Expr::Literal(Literal::String(_)) = method_call.object.as_ref() {
                true
            } else {
                false
            };
            self.generate_expr(&method_call.object)?;
            self.output.push_str("[(");
            self.generate_expr(&method_call.args[0])?;
            self.output.push_str(") as usize..");
            if method_call.args.len() >= 2 {
                self.output.push('(');
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") as usize");
            }
            self.output.push(']');
            if is_string {
                self.output.push_str(".to_string()");
            } else {
                self.output.push_str(".to_vec()");
            }
            return Ok(());
        }

        // Handle arr.first() — returns first element (Option<T>)
        if method_call.method == "first" && method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".first().cloned()");
            if !self.suppress_option_unwrap {
                self.output.push_str(".unwrap()");
            }
            return Ok(());
        }

        // Handle arr.last() — returns last element (Option<T>)
        if method_call.method == "last" && method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".last().cloned()");
            if !self.suppress_option_unwrap {
                self.output.push_str(".unwrap()");
            }
            return Ok(());
        }

        // Handle arr.take(n) — first n elements
        if method_call.method == "take" && !method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str("[..(");
            self.generate_expr(&method_call.args[0])?;
            self.output.push_str(") as usize].to_vec()");
            return Ok(());
        }

        // Handle arr.drop(n) — all elements except first n
        if method_call.method == "drop" && !method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str("[(");
            self.generate_expr(&method_call.args[0])?;
            self.output.push_str(") as usize..].to_vec()");
            return Ok(());
        }

        // Handle arr.reversed() — returns new reversed array
        if method_call.method == "reversed" && !object_is_class_instance {
            self.output.push_str("{ let mut __v = ");
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".clone(); __v.reverse(); __v }");
            return Ok(());
        }

        // Handle arr.sort() — returns new sorted array
        if method_call.method == "sort" && method_call.args.is_empty() && !object_is_class_instance {
            self.output.push_str("{ let mut __v = ");
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".clone(); __v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)); __v }");
            return Ok(());
        }

        // Handle arr.sortBy(fn) — sort by key extraction function
        if method_call.method == "sortBy" && !method_call.args.is_empty() && !object_is_class_instance {
            if let Expr::Lambda(lambda) = &method_call.args[0] {
                let param_name = lambda.params.first()
                    .and_then(|p| p.name())
                    .map(|n| self.sanitize_name(n))
                    .unwrap_or_else(|| "__x".to_string());
                self.output.push_str("{ let mut __v = ");
                self.generate_expr(&method_call.object)?;
                write!(self.output, ".clone(); __v.sort_by(|__a, __b| {{ let {} = (*__a).clone(); let __ka = ", param_name).unwrap();
                match &lambda.body {
                    LambdaBody::Expr(expr) => self.generate_expr(expr)?,
                    LambdaBody::Block(block) => self.generate_block_inner(block)?,
                }
                write!(self.output, "; let {} = (*__b).clone(); let __kb = ", param_name).unwrap();
                match &lambda.body {
                    LambdaBody::Expr(expr) => self.generate_expr(expr)?,
                    LambdaBody::Block(block) => self.generate_block_inner(block)?,
                }
                self.output.push_str("; __ka.partial_cmp(&__kb).unwrap_or(std::cmp::Ordering::Equal) }); __v }");
                return Ok(());
            }
        }

        // Handle arr.groupBy(fn) — group elements by key extraction function → HashMap<K, Vec<V>>
        if method_call.method == "groupBy" && !method_call.args.is_empty() && !object_is_class_instance {
            if let Expr::Lambda(lambda) = &method_call.args[0] {
                let param_name = lambda.params.first()
                    .and_then(|p| p.name())
                    .map(|n| self.sanitize_name(n))
                    .unwrap_or_else(|| "__x".to_string());
                self.output.push_str("{ let mut __m: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new(); for __item in ");
                self.generate_expr(&method_call.object)?;
                write!(self.output, ".iter() {{ let {} = (*__item).clone(); let __key = ", param_name).unwrap();
                match &lambda.body {
                    LambdaBody::Expr(expr) => self.generate_expr(expr)?,
                    LambdaBody::Block(block) => self.generate_block_inner(block)?,
                }
                self.output.push_str("; __m.entry(__key).or_insert_with(Vec::new).push((*__item).clone()); } __m }");
                return Ok(());
            }
        }

        // Handle arr.distinct() — removes duplicates preserving order
        if method_call.method == "distinct" && !object_is_class_instance {
            self.output.push_str("{ let mut __seen = std::collections::HashSet::new(); ");
            self.generate_expr(&method_call.object)?;
            self.output.push_str(
                ".iter().filter(|x| __seen.insert((*x).clone())).cloned().collect::<Vec<_>>() }",
            );
            return Ok(());
        }

        // Handle arr.flat() — flattens one level of nesting
        if (method_call.method == "flat" || method_call.method == "flatten") && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".concat()");
            return Ok(());
        }

        // Handle arr.chunks(size) — splits into sub-arrays of given size
        if method_call.method == "chunks" && !method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".chunks((");
            self.generate_expr(&method_call.args[0])?;
            self.output
                .push_str(") as usize).map(|c| c.to_vec()).collect::<Vec<Vec<_>>>()");
            return Ok(());
        }

        // Handle arr.zip(other) — combines two arrays into array of tuples
        if method_call.method == "zip" && !method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".iter().zip(");
            self.generate_expr(&method_call.args[0])?;
            self.output
                .push_str(".iter()).map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>()");
            return Ok(());
        }

        // Handle arr.sum() — sums all elements
        if method_call.method == "sum" && method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            let sum_type =
                if let Some(base_var) = self.get_base_var_name(&method_call.object) {
                    if let Some(elem_type) = self.typed_array_vars.get(&base_var) {
                        if elem_type == "float" || elem_type == "f64" {
                            "f64"
                        } else {
                            "i32"
                        }
                    } else {
                        "i32"
                    }
                } else {
                    "i32"
                };
            write!(self.output, ".iter().sum::<{}>()", sum_type).unwrap();
            return Ok(());
        }

        // Handle arr.min() — returns minimum element
        if method_call.method == "min" && method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(
                ".iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).cloned()",
            );
            if !self.suppress_option_unwrap {
                self.output.push_str(".unwrap()");
            }
            return Ok(());
        }

        // Handle arr.max() — returns maximum element
        if method_call.method == "max" && method_call.args.is_empty() && !object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(
                ".iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)).cloned()",
            );
            if !self.suppress_option_unwrap {
                self.output.push_str(".unwrap()");
            }
            return Ok(());
        }

        // B10 fix: When object is a class instance with a user-defined count() method,
        // skip the array built-in count(fn) pipeline and generate a plain method call
        if method_call.method == "count" && object_is_class_instance {
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".count(");
            for (i, arg) in method_call.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.generate_expr(arg)?;
            }
            self.output.push(')');
            return Ok(());
        }

        // Check if this is a method on HTTP Response (e.g., response.json())
        if let Expr::Identifier(var_name) = method_call.object.as_ref() {
            if self.rust_struct_vars.contains(var_name) && method_call.method == "json" {
                // This is response.json() - generate as method call
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".json()");
                return Ok(());
            }
        }

        // Check if the object is Option<JsonValue> (from JSON.parse with error binding)
        // If so, we need to unwrap it before calling array methods
        let is_option_json_value = if let Expr::Identifier(var_name) = method_call.object.as_ref() {
            let sanitized = self.sanitize_name(var_name);
            self.option_value_vars.contains(&sanitized) && self.json_value_vars.contains(&sanitized)
        } else {
            false
        };

        // Bug #36: For binary expressions as method call object, we need parentheses
        // e.g., (arr.length - 1).toString() should generate ((arr.len() as i32) - 1).to_string()
        // Without parens, `- 1.to_string()` has wrong precedence
        let needs_parens_for_binary = matches!(method_call.object.as_ref(), Expr::Binary { .. });

        if needs_parens_for_binary {
            self.output.push('(');
        }

        // Generate the object
        self.generate_expr(&method_call.object)?;

        if needs_parens_for_binary {
            self.output.push(')');
        }

        // Unwrap Option<JsonValue> before calling methods
        if is_option_json_value {
            self.output.push_str(".as_ref().unwrap()");
        }

        // Check if operating on JsonValue
        let is_json_value = self.is_json_value_expr(&method_call.object);
        let is_direct_json = self.is_direct_json_value(&method_call.object);

        // Pre-compute whether array elements are non-Copy types (String, classes)
        // Used to add .cloned() after .iter()/.par_iter() for map/forEach
        // so lambda params are owned T instead of &T (avoids E0308 when passed to functions)
        let iter_needs_clone = if !is_json_value {
            if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                if let Some(element_type) = self.typed_array_vars.get(&base_var_name) {
                    !matches!(
                        element_type.as_str(),
                        "number" | "int" | "i32" | "float" | "f64" | "bool" | "char"
                    )
                } else if self.string_vars.contains(&base_var_name)
                    || self.native_vec_string_vars.contains(&base_var_name)
                {
                    true
                } else if self.array_vars.contains(&base_var_name)
                    && !self.json_value_vars.contains(&base_var_name)
                {
                    true
                } else {
                    false
                }
            } else {
                true // Default to cloned for safety
            }
        } else {
            false
        };

        // Handle array methods with adapters
        match method_call.adapter {
            ArrayAdapter::Seq | ArrayAdapter::Vec => {
                // Sequential execution (Vec/SIMD falls back to sequential until SIMD codegen is implemented)
                match method_call.method.as_str() {
                    "map" => {
                        // For map, use iter()
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            self.output.push_str(".cloned()");
                        } else if iter_needs_clone {
                            // Non-Copy types: .cloned() converts &T to T so lambda params
                            // can be passed to functions expecting owned values
                            self.output.push_str(".cloned()");
                        }
                    }
                    "filter" => {
                        // For filter, use iter()
                        // For Vec<JsonValue>, add .cloned() to clone elements
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            // Vec<JsonValue> needs cloned()
                            self.output.push_str(".cloned()");
                        }
                    }
                    "reduce" => {
                        // reduce doesn't use .iter() - it operates directly on the vector
                        // We use .fold() which requires initial value and accumulator
                    }
                    "forEach" => {
                        // For forEach, use iter()
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            self.output.push_str(".cloned()");
                        } else if iter_needs_clone {
                            self.output.push_str(".cloned()");
                        }
                    }
                    "find" | "some" | "every" | "indexOf" | "includes" | "findIndex" | "count" => {
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            self.output.push_str(".cloned()");
                        }
                    }
                    "flatMap" => {
                        // flatMap uses iter() like map
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            self.output.push_str(".cloned()");
                        } else if iter_needs_clone {
                            self.output.push_str(".cloned()");
                        }
                    }
                    _ => {
                        // For other methods, call directly
                    }
                }
            }
            ArrayAdapter::Par | ArrayAdapter::ParVec => {
                // Parallel execution (ParVec SIMD layer falls back to parallel-only for now)
                // For JsonValue, need to convert to Vec first
                if is_direct_json {
                    self.output.push_str(".to_vec().into_par_iter()");
                } else {
                    self.output.push_str(".par_iter()");

                    // For non-Copy types (String, classes) with map/forEach, add .cloned()
                    // to get owned values. Without this, par_iter() yields &T and passing
                    // lambda params to functions expecting T causes E0308 type mismatch.
                    if matches!(method_call.method.as_str(), "map" | "forEach" | "flatMap")
                        && iter_needs_clone
                    {
                        self.output.push_str(".cloned()");
                    }
                }

                // TODO: Handle adapter options (threads, chunk, ordered)
                if method_call.adapter_options.threads.is_some()
                    || method_call.adapter_options.chunk.is_some()
                {
                    // For now, just use default parallel iterator
                    // TODO: Configure rayon thread pool with options
                }
            }
        }

        // Generate the method call

        // Special handling for reduce: it uses .iter() on the vector itself
        if method_call.method == "reduce"
            && matches!(method_call.adapter, ArrayAdapter::Seq | ArrayAdapter::Vec)
        {
            self.output.push_str(".iter()");
        }

        // Bug #47-48: For parallel reduce with Copy types, add .copied() to get owned values
        if method_call.method == "reduce"
            && matches!(
                method_call.adapter,
                ArrayAdapter::Par | ArrayAdapter::ParVec
            )
        {
            // Check if we need .copied() (Copy types) or the iterator already has owned values
            let needs_copied =
                if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                    if let Some(element_type) = self.typed_array_vars.get(&base_var_name) {
                        matches!(
                            element_type.as_str(),
                            "number" | "int" | "i32" | "float" | "f64" | "bool" | "char"
                        )
                    } else {
                        false
                    }
                } else {
                    false
                };
            if needs_copied {
                self.output.push_str(".copied()");
            }
        }

        self.output.push('.');

        // Map Liva method names to Rust iterator method names
        let _is_parallel = matches!(
            method_call.adapter,
            ArrayAdapter::Par | ArrayAdapter::ParVec
        );
        let rust_method = match method_call.method.as_str() {
            "forEach" => "for_each".to_string(),
            "indexOf" => {
                // Rayon uses position_first() for ordered parallel position search
                if _is_parallel {
                    "position_first".to_string()
                } else {
                    "position".to_string()
                }
            }
            "find" => {
                // Rayon uses find_first() for ordered parallel find
                if _is_parallel {
                    "find_first".to_string()
                } else {
                    "find".to_string()
                }
            }
            "includes" => "any".to_string(),
            // For parallel reduce, we'll use fold + reduce (handled specially below)
            "reduce" => "fold".to_string(),
            "some" => "any".to_string(),  // Liva: some, Rust: any
            "every" => "all".to_string(), // Liva: every, Rust: all
            "findIndex" => "position".to_string(), // Liva: findIndex, Rust: position
            "flatMap" => "flat_map".to_string(),   // Liva: flatMap, Rust: flat_map
            "count" => "filter".to_string(),       // Liva: count(fn), Rust: filter(fn).count()
            method_name => self.sanitize_name(method_name), // Sanitize custom method names (e.g., isAdult -> is_adult)
        };

        self.output.push_str(&rust_method);
        self.output.push('(');

        // Generate arguments
        // Special case: reduce needs arguments reversed (initial first, then lambda)
        // Also: Rayon parallel fold uses || identity closure, not just identity value
        let is_parallel_reduce = method_call.method == "reduce"
            && matches!(
                method_call.adapter,
                ArrayAdapter::Par | ArrayAdapter::ParVec
            );
        // Note: Liva reduce syntax is .reduce(initial, lambda) - same order as Rust's .fold()
        // No reordering needed
        let args_to_generate: Vec<&Expr> = method_call.args.iter().collect();

        for (i, arg) in args_to_generate.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }

            // Bug #47-48 fix: Rayon parallel fold needs closure for identity: || initial
            if is_parallel_reduce && i == 0 {
                self.output.push_str("|| ");
                self.generate_expr(arg)?;
                // B106 fix: String initial value needs .to_string() for parallel too
                if matches!(arg, Expr::Literal(Literal::String(_))) {
                    self.output.push_str(".to_string()");
                }
                continue;
            }

            // B106 fix: Sequential reduce initial value — string literal needs .to_string()
            // so accumulator type is String (not &str) matching the lambda return type
            if method_call.method == "reduce" && i == 0 && matches!(arg, Expr::Literal(Literal::String(_))) {
                self.generate_expr(arg)?;
                self.output.push_str(".to_string()");
                continue;
            }

            // Special handling for includes/indexOf: wrap value in closure
            if method_call.method == "includes" || method_call.method == "indexOf" {
                self.output.push_str("|x| *x == ");
                self.generate_expr(arg)?;
                continue;
            }

            // Convert string literals to String for methods/functions
            // This avoids "expected String, found &str" errors
            if matches!(arg, Expr::Literal(Literal::String(_))) {
                // For array methods, JsonValue methods, and join (which expects &str), don't convert
                let is_array_or_json_method = matches!(
                    method_call.method.as_str(),
                    "map"
                        | "filter"
                        | "reduce"
                        | "forEach"
                        | "find"
                        | "some"
                        | "every"
                        | "indexOf"
                        | "includes"
                        | "findIndex"
                        | "flatMap"
                        | "count"
                        | "get"
                        | "get_field"
                        | "join"
                );

                if !is_array_or_json_method {
                    self.generate_expr(arg)?;
                    self.output.push_str(".to_string()");
                    continue;
                }
            }

            // Bug #77 fix: Clone string variables and class instances when passed to instance method calls
            // to avoid ownership issues (Rust moves String/struct by default)
            // This matches the behavior in generate_call_expr for regular function calls
            if let Expr::Identifier(var_name) = arg {
                let is_array_or_iterator_method = matches!(
                    method_call.method.as_str(),
                    "map" | "filter" | "reduce" | "forEach" | "find"
                        | "some" | "every" | "indexOf" | "includes"
                        | "findIndex" | "flatMap" | "count"
                );

                if !is_array_or_iterator_method {
                    let sanitized = self.sanitize_name(var_name);
                    if self.string_vars.contains(&sanitized)
                        || self.class_instance_vars.contains(&sanitized)
                    {
                        self.generate_expr(arg)?;
                        self.output.push_str(".clone()");
                        continue;
                    }
                }
            }

            // For map/filter/reduce/forEach/find/some/every with .iter(), we need to dereference in the lambda
            // map: |&x| - filter: |&&x| (for .copied()) or |x| (for .cloned())
            // reduce: |acc, &x| - forEach: |&x| - find: |&&x| or |x| - some: |&&x| or |x| - every: |&&x| or |x|
            // EXCEPTION: JsonValue.iter() and JsonValue.to_vec().into_par_iter() return owned values, so no dereferencing needed
            // For parallel: .par_iter() uses &T, but .into_par_iter() (from .to_vec()) uses T (owned)
            let is_json_value = self.is_json_value_expr(&method_call.object);

            // Determine if we'll use .cloned() (for non-Copy types) which changes the lambda pattern
            // With .copied() (Copy types): filter(|&&x| ...)
            // With .cloned() (Clone but not Copy types): filter(|x| ...)
            let will_use_cloned =
                if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                    if let Some(element_type) = self.typed_array_vars.get(&base_var_name) {
                        // Check if element type is a Copy type (class names are non-Copy)
                        // Bug #35: "string" is not Copy, so forEach needs |p| not |&p|
                        !matches!(
                            element_type.as_str(),
                            "number" | "int" | "i32" | "float" | "f64" | "bool" | "char"
                        )
                    } else if self.string_vars.contains(&base_var_name) {
                        true
                    } else if self.array_vars.contains(&base_var_name)
                        && !self.json_value_vars.contains(&base_var_name)
                    {
                        true
                    } else {
                        // B15 fix: Default to .cloned() for safety — .cloned() works for
                        // both Copy and non-Copy types (Copy implies Clone)
                        true
                    }
                } else {
                    true // Default to cloned for safety
                };

            let needs_lambda_pattern = (method_call.method == "map"
                || method_call.method == "filter"
                || method_call.method == "reduce"
                || method_call.method == "forEach"
                || method_call.method == "find"
                || method_call.method == "some"
                || method_call.method == "every"
                || method_call.method == "findIndex"
                || method_call.method == "flatMap"
                || method_call.method == "count")
                && (matches!(method_call.adapter, ArrayAdapter::Seq | ArrayAdapter::Vec)
                    // Bug #47-48 fix: par_iter() also returns &T, so we need lambda patterns for dereferencing
                    // Exception: JsonValue with .to_vec().into_par_iter() gets owned values (no deref needed only for is_direct_json)
                    || matches!(method_call.adapter, ArrayAdapter::Par | ArrayAdapter::ParVec));

            if needs_lambda_pattern {
                // Phase 11.3: Point-free function references
                // items.forEach(print) → items.forEach(|&_x| println!("{}", _x))
                // nums.map(toString) → nums.map(|&_x| format!("{}", _x))
                // names.filter(isValid) → names.filter(|&_x| is_valid(_x))
                if let Expr::Identifier(func_name) = arg {
                    // Only treat as function reference if it's NOT a variable that holds a closure
                    // Function references are bare identifiers used where a closure is expected
                    let is_callback_method = matches!(
                        method_call.method.as_str(),
                        "forEach" | "map" | "filter" | "find" | "some" | "every" | "findIndex" | "flatMap" | "count"
                    );

                    if is_callback_method {
                        // Generate the appropriate lambda pattern based on method and type
                        let param_pattern = if is_json_value {
                            "_x".to_string()
                        } else if method_call.method == "filter"
                            || method_call.method == "find"
                            || method_call.method == "some"
                            || method_call.method == "every"
                            || method_call.method == "findIndex"
                            || method_call.method == "count"
                        {
                            if will_use_cloned {
                                "_x".to_string()
                            } else {
                                "&&_x".to_string()
                            }
                        } else if method_call.method == "map" || method_call.method == "forEach"
                            || method_call.method == "flatMap"
                        {
                            if will_use_cloned {
                                "_x".to_string()
                            } else {
                                "&_x".to_string()
                            }
                        } else {
                            "&_x".to_string()
                        };

                        // Generate the function call body based on built-in vs user function
                        self.output.push_str(&format!("|{}| ", param_pattern));

                        // B107 fix: For filter/find/some/every/count with will_use_cloned,
                        // _x is &T but the function expects T. Use (*_x).clone() to dereference.
                        let needs_deref_clone = will_use_cloned && !is_json_value
                            && matches!(method_call.method.as_str(),
                                "filter" | "find" | "some" | "every" | "findIndex" | "count");

                        match func_name.as_str() {
                            "print" => {
                                self.output.push_str("println!(\"{}\", _x)");
                            }
                            "toString" => {
                                self.output.push_str("format!(\"{}\", _x)");
                            }
                            _ => {
                                // User-defined function: generate sanitized_name(_x)
                                let sanitized = self.sanitize_name(func_name);
                                if needs_deref_clone {
                                    write!(self.output, "{}((*_x).clone())", sanitized).unwrap();
                                } else {
                                    write!(self.output, "{}(_x)", sanitized).unwrap();
                                }
                            }
                        }

                        continue;
                    }
                }

                // Phase 11.4: Method references with :: syntax in array methods
                // items.filter(Utils::validate) → items.filter(|&_x| Utils::validate(_x))
                // items.map(User::new) → items.map(|&_x| User::new(_x))
                // items.forEach(logger::log) → items.forEach(|&_x| logger.log(_x))
                if let Expr::MethodRef { object, method } = arg {
                    let is_callback_method = matches!(
                        method_call.method.as_str(),
                        "forEach" | "map" | "filter" | "find" | "some" | "every" | "findIndex" | "flatMap" | "count"
                    );

                    if is_callback_method {
                        let param_pattern = if is_json_value {
                            "_x".to_string()
                        } else if method_call.method == "filter"
                            || method_call.method == "find"
                            || method_call.method == "some"
                            || method_call.method == "every"
                            || method_call.method == "findIndex"
                            || method_call.method == "count"
                        {
                            if will_use_cloned {
                                "_x".to_string()
                            } else {
                                "&&_x".to_string()
                            }
                        } else if method_call.method == "map" || method_call.method == "forEach"
                            || method_call.method == "flatMap"
                        {
                            if will_use_cloned {
                                "_x".to_string()
                            } else {
                                "&_x".to_string()
                            }
                        } else {
                            "&_x".to_string()
                        };

                        self.output.push_str(&format!("|{}| ", param_pattern));

                        let sanitized_obj = self.sanitize_name(object);
                        let sanitized_method = self.sanitize_name(method);
                        let is_class = object.chars().next().map_or(false, |c| c.is_uppercase());

                        // Determine if we need .to_string() conversion for the argument
                        // Methods take String params, but iterators yield &str for string arrays
                        // Only skip conversion for class-typed arrays (not primitive types like "string")
                        let is_class_typed_array = if let Some(base_var_name) =
                            self.get_base_var_name(&method_call.object)
                        {
                            self.typed_array_vars
                                .get(&base_var_name)
                                .map(|t| {
                                    !matches!(
                                        t.as_str(),
                                        "string"
                                            | "String"
                                            | "int"
                                            | "i32"
                                            | "float"
                                            | "f64"
                                            | "bool"
                                            | "number"
                                    )
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        let arg_expr = if is_class_typed_array || is_json_value {
                            "_x".to_string()
                        } else {
                            "_x.to_string()".to_string()
                        };

                        // For forEach, we need to discard the return value since for_each expects ()
                        let is_for_each = method_call.method == "forEach";

                        if is_for_each {
                            self.output.push_str("{ ");
                        }

                        if method == "new" {
                            // Constructor: User::new → User::new(_x.to_string())
                            write!(self.output, "{}::new({})", sanitized_obj, arg_expr).unwrap();
                        } else if is_class {
                            // Static method: Utils::validate → Utils::validate(_x.to_string())
                            write!(
                                self.output,
                                "{}::{}({})",
                                sanitized_obj, sanitized_method, arg_expr
                            )
                            .unwrap();
                        } else {
                            // Instance method: logger::log → logger.log(_x.to_string())
                            write!(
                                self.output,
                                "{}.{}({})",
                                sanitized_obj, sanitized_method, arg_expr
                            )
                            .unwrap();
                        }

                        // forEach closures must return (), so wrap in block with semicolon
                        if is_for_each {
                            self.output.push_str("; }");
                        }

                        continue;
                    }
                }

                if let Expr::Lambda(lambda) = arg {
                    // Track lambda parameter types for typed arrays
                    // If the object is a typed array (e.g., posts: [Post]), track that the param is Post
                    let element_type =
                        if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                            self.typed_array_vars.get(&base_var_name).cloned()
                        } else {
                            None
                        };

                    if let Some(ref _elem_type) = element_type {
                        // Track the lambda parameter as an instance of this class type
                        for param in &lambda.params {
                            if let Some(name) = param.name() {
                                let param_name = self.sanitize_name(name);
                                self.class_instance_vars.insert(param_name);
                            }
                        }
                    }

                    // Generate lambda with pattern |&x| or |&&x| or |acc, &x| (unless JsonValue)
                    if lambda.is_move {
                        self.output.push_str("move ");
                    }
                    self.output.push('|');
                    for (idx, param) in lambda.params.iter().enumerate() {
                        if idx > 0 {
                            self.output.push_str(", ");
                        }

                        // Get parameter name (temp name if destructured)
                        let param_name = if param.is_destructuring() {
                            format!("_param_{}", idx)
                        } else {
                            self.sanitize_name(param.name().unwrap())
                        };

                        // reduce: first param (acc) no pattern, second param (&x) gets & (for sequential only)
                        // Bug #47-48: Parallel reduce with .copied() gets owned values - no & needed
                        let is_parallel_adapter = matches!(
                            method_call.adapter,
                            ArrayAdapter::Par | ArrayAdapter::ParVec
                        );
                        if method_call.method == "reduce" {
                            if idx == 0 {
                                // Accumulator: no dereferencing
                                self.output.push_str(&param_name);
                            } else {
                                // Element: dereference once for sequential (unless JsonValue or destructured)
                                // For parallel with .copied(), no dereference needed
                                // B106 fix: Don't add & for non-Copy types (will_use_cloned=true) — would try to move from &ref
                                if !is_json_value
                                    && !param.is_destructuring()
                                    && !is_parallel_adapter
                                    && !will_use_cloned
                                {
                                    self.output.push('&');
                                }
                                self.output.push_str(&param_name);
                            }
                        } else {
                            // filter/find/some/every need different patterns based on whether we'll use .cloned() or .copied()
                            // - With .copied() (Copy types): filter(|&&x| ...) - double deref
                            // - With .cloned() (Clone types): filter(|x| x...) - no deref, closure receives &&T but cloned() handles it
                            // map/forEach need & for Copy types (closure takes &T), but for non-Copy types,
                            // we work with references directly to avoid moving
                            // UNLESS it's JsonValue, then no dereferencing at all
                            // ALSO: if parameter is destructured, don't add & because we'll clone inside
                            if !is_json_value && !param.is_destructuring() {
                                if method_call.method == "filter"
                                    || method_call.method == "find"
                                    || method_call.method == "count"
                                {
                                    // filter/find/count: FnMut(&Self::Item) → extra & on top of iter's &T
                                    // For Copy types: filter(|&&x| ...) - double deref
                                    // For Clone types: filter(|&x| ...) - single deref
                                    if !will_use_cloned {
                                        self.output.push_str("&&");
                                    } else {
                                        self.output.push('&');
                                        // Track this param as a &T reference so we dereference it
                                        // in comparisons (e.g., *item == query instead of item == query)
                                        self.ref_lambda_params.insert(param_name.clone());
                                    }
                                } else if method_call.method == "some"
                                    || method_call.method == "every"
                                    || method_call.method == "findIndex"
                                {
                                    // Bug #78 fix: any/all/position: FnMut(Self::Item) → same as iter output
                                    // For Copy types: any(|&x| ...) - single deref (iter yields &T)
                                    // For Clone types: no prefix - work with &T directly, but track
                                    //   as ref param so comparisons auto-dereference
                                    if !will_use_cloned {
                                        self.output.push('&');
                                    } else {
                                        // Track for dereference in comparisons (x > val where x is &T)
                                        self.ref_lambda_params.insert(param_name.clone());
                                    }
                                    // For cloned types, no prefix needed (work with &T directly)
                                } else if method_call.method == "map"
                                    || method_call.method == "forEach"
                                    || method_call.method == "flatMap"
                                {
                                    // Bug #22/#35 fix: For non-Copy types (class instances, strings),
                                    // don't add & because we can't move out of a shared reference
                                    if !will_use_cloned {
                                        self.output.push('&');
                                    }
                                    // For non-Copy types (will_use_cloned = true), no prefix - work with &T directly
                                } else {
                                    self.output.push('&');
                                }
                            }
                            self.output.push_str(&param_name);
                        }
                    }
                    self.output.push_str("| ");

                    // Check if we need to generate destructuring code for lambda params
                    let has_destructuring = lambda.params.iter().any(|p| p.is_destructuring());

                    match &lambda.body {
                        LambdaBody::Expr(expr) => {
                            if has_destructuring {
                                // Need to wrap in block to add destructuring
                                self.output.push('{');
                                self.indent();
                                self.output.push('\n');

                                // Generate destructuring for each param
                                for (idx, param) in lambda.params.iter().enumerate() {
                                    if param.is_destructuring() {
                                        let temp_name = format!("_param_{}", idx);
                                        self.write_indent();
                                        self.generate_lambda_param_destructuring(
                                            &param.pattern,
                                            &temp_name,
                                            is_json_value,
                                            element_type.as_deref(),
                                        )?;
                                        self.output.push('\n');
                                    }
                                }

                                self.write_indent();
                                self.generate_expr(expr)?;
                                self.output.push('\n');
                                self.dedent();
                                self.write_indent();
                                self.output.push('}');
                            } else {
                                self.generate_expr(expr)?;
                            }
                        }
                        LambdaBody::Block(block) => {
                            self.output.push('{');
                            self.indent();
                            self.output.push('\n');

                            // Generate destructuring for lambda params (if any)
                            if has_destructuring {
                                for (idx, param) in lambda.params.iter().enumerate() {
                                    if param.is_destructuring() {
                                        let temp_name = format!("_param_{}", idx);
                                        self.write_indent();
                                        self.generate_lambda_param_destructuring(
                                            &param.pattern,
                                            &temp_name,
                                            is_json_value,
                                            element_type.as_deref(),
                                        )?;
                                        self.output.push('\n');
                                    }
                                }
                            }

                            self.write_indent();
                            for stmt in &block.stmts[..block.stmts.len().saturating_sub(1)] {
                                self.generate_stmt(stmt)?;
                                self.output.push('\n');
                                self.write_indent();
                            }
                            if let Some(last_stmt) = block.stmts.last() {
                                if let Stmt::Return(return_stmt) = last_stmt {
                                    if let Some(expr) = &return_stmt.expr {
                                        self.generate_expr(expr)?;
                                    }
                                } else {
                                    self.generate_stmt(last_stmt)?;
                                }
                            }
                            self.dedent();
                            self.output.push('\n');
                            self.write_indent();
                            self.output.push('}');
                        }
                    }
                    // Clear ref_lambda_params after lambda body is generated
                    self.ref_lambda_params.clear();
                    continue;
                }
            }

            // Track lambda parameter types for typed arrays BEFORE generating the lambda
            // This handles ParVec/Par forEach/map/etc with typed arrays (not JsonValue)
            if let Expr::Lambda(lambda) = arg {
                if matches!(
                    method_call.method.as_str(),
                    "forEach" | "map" | "filter" | "reduce" | "find" | "some" | "every" | "findIndex" | "flatMap" | "count"
                ) {
                    if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                        if let Some(element_type) =
                            self.typed_array_vars.get(&base_var_name).cloned()
                        {
                            // Set current element type for lambda generation
                            self.current_lambda_element_type = Some(element_type.clone());

                            // Track the lambda parameter as an instance of this class type
                            for param in &lambda.params {
                                if let Some(name) = param.name() {
                                    let param_name = self.sanitize_name(name);
                                    self.class_instance_vars.insert(param_name);
                                }
                            }
                        }
                    }
                }
            }

            self.generate_expr(arg)?;

            // Clear current element type after generating lambda
            self.current_lambda_element_type = None;
        }

        self.output.push(')');

        // Add transformations after the method call
        let is_json_value = self.is_json_value_expr(&method_call.object);

        // Determine if the array contains non-Copy types (String, classes, etc.)
        // Non-Copy types need .cloned() instead of .copied()
        // Copy types: number, int, i32, float, f64, bool, char
        // Non-Copy types: string, String, and any class name
        let needs_clone_not_copy =
            if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                if let Some(element_type) = self.typed_array_vars.get(&base_var_name) {
                    // Check if element type is a Copy type (class names are non-Copy)
                    !matches!(
                        element_type.as_str(),
                        "number" | "int" | "i32" | "float" | "f64" | "bool" | "char"
                    )
                } else if self.string_vars.contains(&base_var_name) {
                    // String arrays explicitly need .cloned()
                    true
                } else if self.array_vars.contains(&base_var_name)
                    && !self.json_value_vars.contains(&base_var_name)
                {
                    // For arrays without explicit type info but not JsonValue,
                    // default to .cloned() as it's safer (works for both Copy and non-Copy)
                    true
                } else {
                    // B15 fix: Default to .cloned() for safety — .cloned() works for
                    // both Copy and non-Copy types (Copy implies Clone)
                    true
                }
            } else {
                // No base variable name - default to .cloned() for safety
                true
            };

        match (method_call.adapter, method_call.method.as_str()) {
            // Sequential/Vec map: just collect (lambda already returns owned values)
            (ArrayAdapter::Seq, "map") | (ArrayAdapter::Vec, "map") => {
                self.output.push_str(".collect::<Vec<_>>()");
            }
            // Sequential/Vec filter: copy/clone values after filtering, then collect
            // - JsonValue: no copy needed (already returns owned values)
            // - Non-Copy types (String, classes): use .cloned()
            // - Copy types (i32, f64, bool, char): use .copied()
            (ArrayAdapter::Seq, "filter") | (ArrayAdapter::Vec, "filter") => {
                if is_json_value {
                    self.output.push_str(".collect::<Vec<_>>()");
                } else if needs_clone_not_copy {
                    self.output.push_str(".cloned().collect::<Vec<_>>()");
                } else {
                    self.output.push_str(".copied().collect::<Vec<_>>()");
                }
            }
            // Parallel map/filter with rayon
            (ArrayAdapter::Par, "map") | (ArrayAdapter::ParVec, "map") => {
                // Map returns owned values (from the lambda), just collect
                self.output.push_str(".collect::<Vec<_>>()");
            }
            (ArrayAdapter::Par, "filter") | (ArrayAdapter::ParVec, "filter") => {
                // Filter returns references, need to clone before collecting
                self.output.push_str(".cloned().collect::<Vec<_>>()");
            }
            // Bug #47-48 fix: Parallel reduce needs .reduce() after fold() to combine partial results
            // Pattern: .fold(|| identity, |acc, x| acc + x).reduce(|| identity, |a, b| a + b)
            (ArrayAdapter::Par, "reduce") | (ArrayAdapter::ParVec, "reduce") => {
                // After fold(|| initial, |acc, x| expr), we need reduce(|| initial, |a, b| combine)
                // Extract the binary operator from the lambda to generate correct combine
                // Liva syntax: reduce(identity, lambda) so args[0] is identity
                if method_call.args.len() >= 2 {
                    let combine_op = Self::extract_reduce_combine_op(&method_call.args[1]);
                    self.output.push_str(".reduce(|| ");
                    self.generate_expr(&method_call.args[0])?; // identity is args[0]
                    write!(self.output, ", |a, b| a {} b)", combine_op).unwrap();
                }
            }
            // Find returns Option<&T>, copy/clone it
            // - JsonValue: no copy needed (already returns owned values)
            // - Non-Copy types: use .cloned()
            // - Copy types: use .copied()
            (_, "find") => {
                if !is_json_value {
                    if needs_clone_not_copy {
                        self.output.push_str(".cloned()");
                    } else {
                        self.output.push_str(".copied()");
                    }
                    if !self.suppress_option_unwrap {
                        self.output.push_str(".unwrap()");
                    }
                }
            }
            // indexOf/position returns Option<usize>
            (_, "indexOf") | (_, "findIndex") => {
                self.output.push_str(".map(|i| i as i32).unwrap_or(-1)");
            }
            // some, every, includes return bool - no transformation needed
            (_, "some") | (_, "every") | (_, "includes") => {}
            // flatMap returns an iterator, collect to Vec
            (_, "flatMap") => {
                self.output.push_str(".collect::<Vec<_>>()");
            }
            // count(fn) uses filter(fn).count() → append .count() as i32
            (_, "count") => {
                self.output.push_str(".count() as i32");
            }
            // Bug #38: JsonValue conversion methods return Option<T>, unwrap to T
            // asString -> as_string().unwrap_or_default()
            // asBool -> as_bool().unwrap_or_default()
            // asInt -> as_i32().unwrap_or_default()
            // asFloat -> as_f64().unwrap_or_default()
            (_, "asString") | (_, "asInt") | (_, "asFloat") | (_, "asBool") => {
                self.output.push_str(".unwrap_or_default()");
            }
            // Bug #41: Vec methods that return Option<T> need unwrap
            // pop() -> Option<T>, unwrap to get T directly
            // In Liva, pop() should return the element, not Option<T>
            (ArrayAdapter::Seq, "pop") => {
                self.output.push_str(".expect(\"pop from empty array\")");
            }
            // Default: no transformation
            _ => {}
        }

        Ok(())
    }

    /// Extract the binary operator from a reduce lambda for parallel combine.
    /// Given `(acc, x) => acc + x`, extracts "+".
    /// Given `(acc, x) => acc * x`, extracts "*".
    /// Falls back to "+" for complex expressions that aren't simple binary ops.
    fn extract_reduce_combine_op(lambda_arg: &Expr) -> &'static str {
        if let Expr::Lambda(lambda) = lambda_arg {
            match &lambda.body {
                LambdaBody::Expr(expr) => Self::binop_to_combine_str(expr),
                LambdaBody::Block(block) => {
                    // Try to extract from the last statement (return or expression)
                    if let Some(last_stmt) = block.stmts.last() {
                        match last_stmt {
                            Stmt::Return(ret) => {
                                if let Some(ref expr) = ret.expr {
                                    return Self::binop_to_combine_str_ref(expr);
                                }
                            }
                            Stmt::Expr(expr_stmt) => {
                                return Self::binop_to_combine_str_ref(&expr_stmt.expr);
                            }
                            _ => {}
                        }
                    }
                    "+" // default fallback
                }
            }
        } else {
            "+" // not a lambda — default to addition
        }
    }

    /// Extract the outermost binary operator from an expression for reduce combine.
    /// For `acc + x * 2`, the outer op is `+`, which is correct for combining partial sums.
    fn binop_to_combine_str(expr: &Box<Expr>) -> &'static str {
        Self::binop_to_combine_str_ref(expr.as_ref())
    }

    fn binop_to_combine_str_ref(expr: &Expr) -> &'static str {
        if let Expr::Binary { op, .. } = expr {
            match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Mod => "%",
                _ => "+",
            }
        } else {
            "+" // non-binary expression (e.g., function call) — default to addition
        }
    }

    /// Generate code for Map method calls (get, set, has, delete, keys, values, entries, clear, forEach)
    fn generate_map_method_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "get" => {
                // map.get(key) → map.get(&key).cloned().unwrap_or_default()
                // Returns String (empty string if key not found)
                // When suppress_map_get_unwrap is true (used with `or`), omits unwrap_or_default()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".get(&");
                if let Some(arg) = method_call.args.first() {
                    if matches!(arg, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push_str(").cloned()");
                if !self.suppress_map_get_unwrap {
                    self.output.push_str(".unwrap_or_default()");
                }
            }
            "set" => {
                // map.set(key, value) → map.insert(key.clone(), value)
                // Bug #76 fix: Clone string variable keys/values to avoid ownership moves
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".insert(");
                if let Some(key) = method_call.args.first() {
                    if matches!(key, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(key)?;
                        self.output.push_str(".to_string()");
                    } else if let Expr::Identifier(name) = key {
                        if self.string_vars.contains(&self.sanitize_name(name)) {
                            self.generate_expr(key)?;
                            self.output.push_str(".clone()");
                        } else {
                            self.generate_expr(key)?;
                        }
                    } else {
                        self.generate_expr(key)?;
                    }
                }
                self.output.push_str(", ");
                if let Some(value) = method_call.args.get(1) {
                    if matches!(value, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(value)?;
                        self.output.push_str(".to_string()");
                    } else if let Expr::Identifier(name) = value {
                        if self.string_vars.contains(&self.sanitize_name(name)) {
                            self.generate_expr(value)?;
                            self.output.push_str(".clone()");
                        } else {
                            self.generate_expr(value)?;
                        }
                    } else {
                        self.generate_expr(value)?;
                    }
                }
                self.output.push(')');
            }
            "has" => {
                // map.has(key) → map.contains_key(&key)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".contains_key(&");
                if let Some(arg) = method_call.args.first() {
                    if matches!(arg, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
            }
            "delete" => {
                // map.delete(key) → map.remove(&key)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".remove(&");
                if let Some(arg) = method_call.args.first() {
                    if matches!(arg, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
            }
            "keys" => {
                // map.keys() → map.keys().cloned().collect::<Vec<_>>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".keys().cloned().collect::<Vec<_>>()");
            }
            "values" => {
                // map.values() → map.values().cloned().collect::<Vec<_>>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".values().cloned().collect::<Vec<_>>()");
            }
            "entries" => {
                // map.entries() → map.iter().map(|(k,v)| (k.clone(), v.clone())).collect::<Vec<_>>()
                self.generate_expr(&method_call.object)?;
                self.output
                    .push_str(".iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()");
            }
            "clear" => {
                // map.clear() → map.clear()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".clear()");
            }
            "forEach" => {
                // map.forEach(callback) → map.iter().for_each(|(k, v)| { ... })
                self.generate_expr(&method_call.object)?;
                if let Some(callback) = method_call.args.first() {
                    match callback {
                        Expr::Lambda(lambda) => {
                            // Use the lambda's parameter names
                            let key_param = lambda.params.get(0).and_then(|p| p.name()).map(|n| self.sanitize_name(n)).unwrap_or_else(|| "k".to_string());
                            let val_param = lambda.params.get(1).and_then(|p| p.name()).map(|n| self.sanitize_name(n)).unwrap_or_else(|| "v".to_string());
                            write!(self.output, ".iter().for_each(|({}, {})| {{\n", key_param, val_param).unwrap();
                            self.indent();
                            // Generate lambda body inline
                            match &lambda.body {
                                LambdaBody::Expr(expr) => {
                                    self.write_indent();
                                    self.generate_expr(expr)?;
                                    self.output.push_str(";\n");
                                }
                                LambdaBody::Block(block) => {
                                    for stmt in &block.stmts {
                                        self.generate_stmt(stmt)?;
                                    }
                                }
                            }
                            self.dedent();
                        }
                        _ => {
                            self.output.push_str(".iter().for_each(|(k, v)| {\n");
                            self.indent();
                            self.write_indent();
                            self.generate_expr(callback)?;
                            self.output.push_str("(k, v);\n");
                            self.dedent();
                        }
                    }
                } else {
                    self.output.push_str(".iter().for_each(|(k, v)| {\n");
                    self.indent();
                    self.dedent();
                }
                self.write_indent();
                self.output.push_str("})");
            }
            _ => {
                // Fallback: generate as normal method call
                self.generate_expr(&method_call.object)?;
                write!(self.output, ".{}(", method_call.method).unwrap();
                for (i, arg) in method_call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
            }
        }
        Ok(())
    }

    /// Generate code for Set method calls (add, has, delete, clear, values, forEach, union, intersection, difference)
    fn generate_set_method_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "add" => {
                // set.add(value) → set.insert(value)
                // Bug #76 fix: Clone string variable values to avoid ownership moves
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".insert(");
                if let Some(arg) = method_call.args.first() {
                    if matches!(arg, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else if let Expr::Identifier(name) = arg {
                        if self.string_vars.contains(&self.sanitize_name(name)) {
                            self.generate_expr(arg)?;
                            self.output.push_str(".clone()");
                        } else {
                            self.generate_expr(arg)?;
                        }
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
            }
            "has" => {
                // set.has(value) → set.contains(&value)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".contains(&");
                if let Some(arg) = method_call.args.first() {
                    if matches!(arg, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
            }
            "delete" => {
                // set.delete(value) → set.remove(&value)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".remove(&");
                if let Some(arg) = method_call.args.first() {
                    if matches!(arg, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
            }
            "clear" => {
                // set.clear() → set.clear()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".clear()");
            }
            "values" => {
                // set.values() → set.iter().cloned().collect::<Vec<_>>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".iter().cloned().collect::<Vec<_>>()");
            }
            "forEach" => {
                // set.forEach(callback) → set.iter().for_each(|v| { ... })
                self.generate_expr(&method_call.object)?;
                if let Some(callback) = method_call.args.first() {
                    match callback {
                        Expr::Lambda(lambda) => {
                            let param = lambda.params.get(0).and_then(|p| p.name()).map(|n| self.sanitize_name(n)).unwrap_or_else(|| "v".to_string());
                            write!(self.output, ".iter().for_each(|{}| {{\n", param).unwrap();
                            self.indent();
                            match &lambda.body {
                                LambdaBody::Expr(expr) => {
                                    self.write_indent();
                                    self.generate_expr(expr)?;
                                    self.output.push_str(";\n");
                                }
                                LambdaBody::Block(block) => {
                                    for stmt in &block.stmts {
                                        self.generate_stmt(stmt)?;
                                    }
                                }
                            }
                            self.dedent();
                        }
                        _ => {
                            self.output.push_str(".iter().for_each(|v| {\n");
                            self.indent();
                            self.write_indent();
                            self.generate_expr(callback)?;
                            self.output.push_str("(v);\n");
                            self.dedent();
                        }
                    }
                } else {
                    self.output.push_str(".iter().for_each(|v| {\n");
                    self.indent();
                    self.dedent();
                }
                self.write_indent();
                self.output.push_str("})");
            }
            "union" => {
                // set.union(other) → set.union(&other).cloned().collect::<HashSet<_>>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".union(&");
                if let Some(arg) = method_call.args.first() {
                    self.generate_expr(arg)?;
                }
                self.output.push_str(").cloned().collect::<std::collections::HashSet<_>>()");
            }
            "intersection" => {
                // set.intersection(other) → set.intersection(&other).cloned().collect::<HashSet<_>>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".intersection(&");
                if let Some(arg) = method_call.args.first() {
                    self.generate_expr(arg)?;
                }
                self.output.push_str(").cloned().collect::<std::collections::HashSet<_>>()");
            }
            "difference" => {
                // set.difference(other) → set.difference(&other).cloned().collect::<HashSet<_>>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".difference(&");
                if let Some(arg) = method_call.args.first() {
                    self.generate_expr(arg)?;
                }
                self.output.push_str(").cloned().collect::<std::collections::HashSet<_>>()");
            }
            _ => {
                // Fallback: generate as normal method call
                self.generate_expr(&method_call.object)?;
                write!(self.output, ".{}(", method_call.method).unwrap();
                for (i, arg) in method_call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
            }
        }
        Ok(())
    }

    fn generate_string_method_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Special handling for methods that need different syntax
        match method_call.method.as_str() {
            "substring" => {
                // substring(start, end) -> &str[(start) as usize..(end) as usize].to_string()
                self.generate_expr(&method_call.object)?;
                self.output.push('[');
                if method_call.args.len() >= 1 {
                    // Wrap in parens to handle expressions like (maxLen - 3) as usize
                    self.output.push('(');
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(") as usize");
                }
                self.output.push_str("..");
                if method_call.args.len() >= 2 {
                    self.output.push('(');
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(") as usize");
                }
                self.output.push_str("].to_string()");
                return Ok(());
            }
            "charAt" => {
                // B95 fix: charAt(index) -> String so it works as Liva string type
                // str.chars().nth((index) as usize).map(|c| c.to_string()).unwrap_or_default()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".chars().nth((");
                if !method_call.args.is_empty() {
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(") as usize");
                }
                self.output.push_str(").map(|c| c.to_string()).unwrap_or_default()");
                return Ok(());
            }
            "indexOf" => {
                // indexOf(substring) -> str.find(substring).map(|i| i as i32).unwrap_or(-1)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".find(");
                if !method_call.args.is_empty() {
                    // If the argument is a String variable, we need &arg for find() to work
                    // find() expects a Pattern, and &String implements Pattern but String doesn't
                    let needs_ref = match &method_call.args[0] {
                        Expr::Identifier(var_name) => {
                            self.string_vars.contains(&self.sanitize_name(var_name))
                        }
                        Expr::Member { .. } => true, // Member access on fields - likely strings
                        Expr::Literal(Literal::String(_)) => false, // String literals are fine
                        _ => true,                   // Be safe and add & for other cases
                    };
                    if needs_ref {
                        self.output.push('&');
                    }
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(").map(|i| i as i32).unwrap_or(-1)");
                return Ok(());
            }
            "lastIndexOf" => {
                // lastIndexOf(substring) -> str.rfind(substring).map(|i| i as i32).unwrap_or(-1)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".rfind(");
                if !method_call.args.is_empty() {
                    let needs_ref = match &method_call.args[0] {
                        Expr::Identifier(var_name) => {
                            self.string_vars.contains(&self.sanitize_name(var_name))
                        }
                        Expr::Member { .. } => true,
                        Expr::Literal(Literal::String(_)) => false,
                        _ => true,
                    };
                    if needs_ref {
                        self.output.push('&');
                    }
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(").map(|i| i as i32).unwrap_or(-1)");
                return Ok(());
            }
            "slice" => {
                // slice(start, end?) -> &str[(start) as usize..(end) as usize].to_string()
                // Same semantics as substring
                self.generate_expr(&method_call.object)?;
                self.output.push('[');
                if method_call.args.len() >= 1 {
                    self.output.push('(');
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(") as usize");
                }
                self.output.push_str("..");
                if method_call.args.len() >= 2 {
                    self.output.push('(');
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(") as usize");
                }
                self.output.push_str("].to_string()");
                return Ok(());
            }
            "padStart" => {
                // padStart(len, char?) -> block expression with padding
                self.output.push_str("{ let __s = &(");
                self.generate_expr(&method_call.object)?;
                self.output.push_str("); let __len = (");
                if !method_call.args.is_empty() {
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output
                    .push_str(") as usize; if __s.len() >= __len { __s.to_string() } else { ");
                if method_call.args.len() >= 2 {
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(".repeat(__len - __s.len()) + __s");
                } else {
                    self.output
                        .push_str("\" \".repeat(__len - __s.len()) + __s");
                }
                self.output.push_str(" } }");
                return Ok(());
            }
            "padEnd" => {
                // padEnd(len, char?) -> block expression with padding
                self.output.push_str("{ let __s = &(");
                self.generate_expr(&method_call.object)?;
                self.output.push_str("); let __len = (");
                if !method_call.args.is_empty() {
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(
                    ") as usize; if __s.len() >= __len { __s.to_string() } else { __s.to_string() + &",
                );
                if method_call.args.len() >= 2 {
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(".repeat(__len - __s.len())");
                } else {
                    self.output
                        .push_str("\" \".repeat(__len - __s.len())");
                }
                self.output.push_str(" } }");
                return Ok(());
            }
            "repeat" => {
                // repeat(n) -> str.repeat(n as usize)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".repeat((");
                if !method_call.args.is_empty() {
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(") as usize)");
                return Ok(());
            }
            "capitalize" => {
                // capitalize() -> { let __s = &(obj); let mut __c = __s.chars(); match __c.next() { ... } }
                self.output.push_str("{ let __s = &(");
                self.generate_expr(&method_call.object)?;
                self.output.push_str("); let mut __c = __s.chars(); match __c.next() { None => String::new(), Some(__f) => __f.to_uppercase().to_string() + __c.as_str() } }");
                return Ok(());
            }
            "isBlank" => {
                // isBlank() -> str.trim().is_empty()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".trim().is_empty()");
                return Ok(());
            }
            "isEmpty" => {
                // isEmpty() -> str.is_empty() (also works for Vec)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".is_empty()");
                return Ok(());
            }
            "toInt" => {
                // toInt() -> str.parse::<i32>().unwrap_or(0)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".parse::<i32>().unwrap_or(0)");
                return Ok(());
            }
            "toFloat" => {
                // toFloat() -> str.parse::<f64>().unwrap_or(0.0)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".parse::<f64>().unwrap_or(0.0)");
                return Ok(());
            }
            "reverse" => {
                // reverse() -> str.chars().rev().collect::<String>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".chars().rev().collect::<String>()");
                return Ok(());
            }
            "truncate" => {
                // truncate(len) -> str.chars().take(len as usize).collect::<String>()
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".chars().take((");
                if !method_call.args.is_empty() {
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(") as usize).collect::<String>()");
                return Ok(());
            }
            "countMatches" => {
                // countMatches(sub) -> str.matches(sub).count() as i32
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".matches(");
                if !method_call.args.is_empty() {
                    let needs_ref = match &method_call.args[0] {
                        Expr::Identifier(var_name) => {
                            self.string_vars.contains(&self.sanitize_name(var_name))
                        }
                        Expr::Member { .. } => true,
                        Expr::Literal(Literal::String(_)) => false,
                        _ => true,
                    };
                    if needs_ref {
                        self.output.push('&');
                    }
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(").count() as i32");
                return Ok(());
            }
            "removePrefix" => {
                // removePrefix(pre) -> { let __s = &(obj); match __s.strip_prefix(pre) { Some(r) => r.to_string(), None => __s.to_string() } }
                self.output.push_str("{ let __s = &(");
                self.generate_expr(&method_call.object)?;
                self.output.push_str("); match __s.strip_prefix(");
                if !method_call.args.is_empty() {
                    let needs_ref = match &method_call.args[0] {
                        Expr::Identifier(var_name) => {
                            self.string_vars.contains(&self.sanitize_name(var_name))
                        }
                        Expr::Member { .. } => true,
                        Expr::Literal(Literal::String(_)) => false,
                        _ => true,
                    };
                    if needs_ref {
                        self.output.push('&');
                    }
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(
                    ") { Some(__r) => __r.to_string(), None => __s.to_string() } }",
                );
                return Ok(());
            }
            "removeSuffix" => {
                // removeSuffix(suf) -> { let __s = &(obj); match __s.strip_suffix(suf) { Some(r) => r.to_string(), None => __s.to_string() } }
                self.output.push_str("{ let __s = &(");
                self.generate_expr(&method_call.object)?;
                self.output.push_str("); match __s.strip_suffix(");
                if !method_call.args.is_empty() {
                    let needs_ref = match &method_call.args[0] {
                        Expr::Identifier(var_name) => {
                            self.string_vars.contains(&self.sanitize_name(var_name))
                        }
                        Expr::Member { .. } => true,
                        Expr::Literal(Literal::String(_)) => false,
                        _ => true,
                    };
                    if needs_ref {
                        self.output.push('&');
                    }
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(
                    ") { Some(__r) => __r.to_string(), None => __s.to_string() } }",
                );
                return Ok(());
            }
            "chars" => {
                // chars() -> str.chars().map(|c| c.to_string()).collect::<Vec<String>>()
                self.generate_expr(&method_call.object)?;
                self.output
                    .push_str(".chars().map(|c| c.to_string()).collect::<Vec<String>>()");
                return Ok(());
            }
            _ => {}
        }

        // Generate the string object
        self.generate_expr(&method_call.object)?;

        // Map Liva string method names to Rust method names
        let rust_method = match method_call.method.as_str() {
            "toUpperCase" => "to_uppercase",
            "toLowerCase" => "to_lowercase",
            "trimStart" => "trim_start",
            "trimEnd" => "trim_end",
            "startsWith" => "starts_with",
            "endsWith" => "ends_with",
            "replaceAll" => "replace",
            method_name => method_name, // split, replace, trim, substring, charAt, contains
        };

        // Generate the method call
        self.output.push('.');
        self.output.push_str(rust_method);
        self.output.push('(');

        // Methods that take Pattern/&str args need & for String variables
        let needs_ref_for_args = matches!(
            method_call.method.as_str(),
            "contains" | "startsWith" | "endsWith" | "split" | "replace" | "replaceAll" | "starts_with" | "ends_with"
        );

        // Generate arguments
        for (i, arg) in method_call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            // Add & for String variable arguments in Pattern-based methods
            if needs_ref_for_args {
                let (is_variable, is_ref_lambda) = match arg {
                    Expr::Identifier(var_name) => {
                        let sanitized = self.sanitize_name(var_name);
                        // Check if the variable is already a &T reference (e.g., lambda param in filter)
                        if self.ref_lambda_params.contains(&sanitized)
                            || self.ref_lambda_params.contains(var_name)
                        {
                            (false, true)
                        } else {
                            let is_var = self.string_vars.contains(&sanitized)
                                || !matches!(var_name.as_str(), "true" | "false" | "null");
                            (is_var, false)
                        }
                    }
                    Expr::Member { .. } => (true, false),
                    Expr::Literal(Literal::String(_)) => (false, false),
                    _ => (false, false),
                };
                if is_variable {
                    self.output.push('&');
                }
                self.generate_expr(arg)?;
                // For ref lambda params (&String), use .as_str() to get &str which implements Pattern
                if is_ref_lambda {
                    self.output.push_str(".as_str()");
                }
            } else {
                self.generate_expr(arg)?;
            }
        }

        self.output.push(')');

        // Post-processing for specific methods
        match method_call.method.as_str() {
            "split" => {
                // split returns an iterator, collect to Vec<String>
                self.output
                    .push_str(".map(|s| s.to_string()).collect::<Vec<String>>()");
            }
            "trim" | "trimStart" | "trimEnd" => {
                // trim/trim_start/trim_end return &str, need .to_string() for String context
                self.output.push_str(".to_string()");
            }
            _ => {}
        }

        Ok(())
    }

    fn generate_math_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Math functions: sqrt, pow, abs, floor, ceil, round, min, max, random
        match method_call.method.as_str() {
            "sqrt" | "abs" => {
                // sqrt(x) -> x.sqrt() or abs(x) -> x.abs()
                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        &format!("Math.{} requires 1 argument", method_call.method),
                        &format!("Math.{} takes exactly one argument", method_call.method),
                    )));
                }

                // Wrap argument in parentheses if it's a unary expression to avoid precedence issues
                let needs_parens = matches!(&method_call.args[0], Expr::Unary { .. });

                if needs_parens {
                    self.output.push('(');
                }
                self.generate_expr(&method_call.args[0])?;
                if needs_parens {
                    self.output.push(')');
                }

                self.output.push('.');
                self.output.push_str(&method_call.method);
                self.output.push_str("()");
            }
            "pow" => {
                // pow(base, exp) -> base.powf(exp)
                if method_call.args.len() < 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Math.pow requires 2 arguments",
                        "Math.pow(base, exponent) takes exactly two arguments",
                    )));
                }
                let needs_parens = matches!(&method_call.args[0], Expr::Unary { .. });
                if needs_parens { self.output.push('('); }
                self.generate_expr(&method_call.args[0])?;
                if needs_parens { self.output.push(')'); }
                self.output.push_str(".powf(");
                self.generate_expr(&method_call.args[1])?;
                self.output.push(')');
            }
            "floor" | "ceil" | "round" => {
                // floor(x) -> x.floor() as i32
                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        &format!("Math.{} requires 1 argument", method_call.method),
                        &format!("Math.{} takes exactly one argument", method_call.method),
                    )));
                }
                let needs_parens = matches!(&method_call.args[0], Expr::Unary { .. });
                if needs_parens { self.output.push('('); }
                self.generate_expr(&method_call.args[0])?;
                if needs_parens { self.output.push(')'); }
                self.output.push('.');
                self.output.push_str(&method_call.method);
                self.output.push_str("() as i32");
            }
            "min" | "max" => {
                // min(a, b) -> a.min(b) or max(a, b) -> a.max(b)
                if method_call.args.len() < 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        &format!("Math.{} requires 2 arguments", method_call.method),
                        &format!("Math.{} takes exactly two arguments", method_call.method),
                    )));
                }
                let needs_parens = matches!(&method_call.args[0], Expr::Unary { .. });
                if needs_parens { self.output.push('('); }
                self.generate_expr(&method_call.args[0])?;
                if needs_parens { self.output.push(')'); }
                self.output.push('.');
                self.output.push_str(&method_call.method);
                self.output.push('(');
                self.generate_expr(&method_call.args[1])?;
                self.output.push(')');
            }
            "random" => {
                // random() -> rand::random::<f64>()
                // Note: requires use rand::Rng in the generated code
                self.output.push_str("rand::random::<f64>()");
            }
            "clamp" => {
                // clamp(val, min, max) -> val.max(min).min(max)
                if method_call.args.len() < 3 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Math.clamp requires 3 arguments",
                        "Math.clamp(value, min, max) takes exactly three arguments",
                    )));
                }
                let needs_parens = matches!(&method_call.args[0], Expr::Unary { .. });
                if needs_parens { self.output.push('('); }
                self.generate_expr(&method_call.args[0])?;
                if needs_parens { self.output.push(')'); }
                self.output.push_str(".max(");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(").min(");
                self.generate_expr(&method_call.args[2])?;
                self.output.push(')');
            }
            "sign" => {
                // sign(val) -> if val > 0 { 1 } else if val < 0 { -1 } else { 0 }
                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Math.sign requires 1 argument",
                        "Math.sign takes exactly one argument",
                    )));
                }
                self.output.push_str("{ let __v: f64 = ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(
                    "; if __v > 0.0 { 1 } else if __v < 0.0 { -1 } else { 0 } }",
                );
            }
            "log" => {
                // log(x) -> (x as f64).ln()
                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Math.log requires 1 argument",
                        "Math.log takes exactly one argument",
                    )));
                }
                self.output.push('(');
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(" as f64).ln()");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown Math function: {}", method_call.method),
                        "Available Math functions: sqrt, pow, abs, floor, ceil, round, min, max, random, clamp, sign, log"
                    )
                ));
            }
        }

        Ok(())
    }

    fn generate_console_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Console functions: log, error, warn
        match method_call.method.as_str() {
            "log" => {
                // console.log(...) -> println!(...)
                if method_call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    // Check if first arg is a string literal with format placeholders
                    // If so, use it as the format string, otherwise generate default format
                    self.output.push_str("println!(");

                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "error" => {
                // console.error(...) -> eprintln!(...) in red color (ANSI escape codes)
                if method_call.args.is_empty() {
                    self.output.push_str("eprintln!()");
                } else {
                    self.output.push_str("eprintln!(\"\\x1b[31m"); // Red color start
                                                                   // Check if first arg is format string
                    if method_call.args.len() == 1 {
                        // Single arg - just print it with reset
                        self.output.push_str("{}\\x1b[0m\", ");
                        self.generate_expr(&method_call.args[0])?;
                    } else if let Expr::Literal(Literal::String(fmt)) = &method_call.args[0] {
                        // First arg is string literal - use as format string
                        // Escape the format string properly
                        for ch in fmt.chars() {
                            match ch {
                                '"' => self.output.push_str("\\\""),
                                '\\' => self.output.push_str("\\\\"),
                                '\n' => self.output.push_str("\\n"),
                                '\r' => self.output.push_str("\\r"),
                                '\t' => self.output.push_str("\\t"),
                                _ => self.output.push(ch),
                            }
                        }
                        self.output.push_str("\\x1b[0m\", ");
                        for (i, arg) in method_call.args.iter().skip(1).enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                    } else {
                        // Multiple args without format string - generate default
                        for (i, _) in method_call.args.iter().enumerate() {
                            if i > 0 {
                                self.output.push(' ');
                            }
                            self.output.push_str("{}");
                        }
                        self.output.push_str("\\x1b[0m\", ");
                        for (i, arg) in method_call.args.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                    }
                    self.output.push(')');
                }
            }
            "warn" => {
                // console.warn(...) -> eprintln!(...) in amber/yellow color (ANSI escape codes)
                // Yellow: \x1b[33m ... \x1b[0m
                if method_call.args.is_empty() {
                    self.output.push_str("eprintln!()");
                } else {
                    self.output.push_str("eprintln!(\"\\x1b[33m"); // Yellow color
                    for (i, _) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push(' '); // Space between arguments
                        }
                        self.output.push_str("{}");
                    }
                    self.output.push_str("\\x1b[0m\", "); // Reset color
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "success" => {
                // console.success(...) -> println!(...) in green color (ANSI escape codes)
                // Green: \x1b[32m ... \x1b[0m
                if method_call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    self.output.push_str("println!(\"\\x1b[32m"); // Green color
                    for (i, _) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push(' '); // Space between arguments
                        }
                        self.output.push_str("{}");
                    }
                    self.output.push_str("\\x1b[0m\", "); // Reset color
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "input" => {
                // console.input() -> reads user input from stdin without prompt
                // console.input(message) -> prints message, then reads input
                // Returns a string with the user input (trimmed)
                // Similar to Python's input() and input("prompt")
                self.output.push_str("{\n");
                self.output.push_str("use std::io::{self, Write};\n");

                // Print the prompt message if provided
                if !method_call.args.is_empty() {
                    // B11: Use print!("{}", expr) to avoid nested print!(format!(...))
                    // when the argument is a template string
                    self.output.push_str("print!(\"{}\", ");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(");\n");
                    // Flush to ensure prompt is displayed before reading
                    self.output.push_str("io::stdout().flush().unwrap();\n");
                }

                // Read user input
                self.output.push_str("let mut input = String::new();\n");
                self.output
                    .push_str("io::stdin().read_line(&mut input).unwrap();\n");
                self.output.push_str("input.trim().to_string()\n");
                self.output.push_str("}");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown console function: {}", method_call.method),
                    "Available console functions: log, error, warn, success, input",
                )));
            }
        }

        Ok(())
    }

    fn generate_json_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // JSON functions: parse, stringify
        match method_call.method.as_str() {
            "parse" => {
                // JSON.parse(json_str) returns (Option<JsonValue>, String)
                // Generates: match serde_json::from_str(...) { Ok(v) => (Some(JsonValue(v)), String::new()), Err(e) => (None, format!("...")) }
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "JSON.parse requires exactly 1 argument",
                        "Usage: JSON.parse(json_string)",
                    )));
                }

                self.output
                    .push_str("match serde_json::from_str::<serde_json::Value>(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(v) => (Some(liva_rt::JsonValue::new(v)), String::new()), Err(e) => (None, format!(\"JSON parse error: {}\", e)) }");
            }
            "stringify" => {
                // JSON.stringify(value) returns (Option<String>, String)
                // Generates: match serde_json::to_string(...) { Ok(s) => (Some(s), String::new()), Err(e) => (None, format!("...")) }
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "JSON.stringify requires exactly 1 argument",
                        "Usage: JSON.stringify(value)",
                    )));
                }

                self.output.push_str("match serde_json::to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(s) => (Some(s), String::new()), Err(e) => (None, format!(\"JSON stringify error: {}\", e)) }");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown JSON function: {}", method_call.method),
                    "Available JSON functions: parse, stringify",
                )));
            }
        }

        Ok(())
    }

    fn generate_file_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // File functions: read, write, append, exists, delete
        match method_call.method.as_str() {
            "read" => {
                // File.read(path) returns (Option<String>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.read requires exactly 1 argument",
                        "Usage: File.read(path)",
                    )));
                }

                self.output.push_str("match std::fs::read_to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(content) => (Some(content), String::new()), Err(e) => (None, format!(\"File read error: {}\", e)) }");
            }
            "write" => {
                // File.write(path, content) returns (Option<bool>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.write requires exactly 2 arguments",
                        "Usage: File.write(path, content)",
                    )));
                }

                self.output.push_str("match std::fs::write(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File write error: {}\", e)) }");
            }
            "append" => {
                // File.append(path, content) returns (Option<bool>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.append requires exactly 2 arguments",
                        "Usage: File.append(path, content)",
                    )));
                }

                self.output.push_str(
                    "match std::fs::OpenOptions::new().create(true).append(true).open(&",
                );
                self.generate_expr(&method_call.args[0])?;
                self.output
                    .push_str(").and_then(|mut file| { use std::io::Write; file.write_all(");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".as_bytes()) }) { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File append error: {}\", e)) }");
            }
            "exists" => {
                // File.exists(path) returns bool (no error binding)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.exists requires exactly 1 argument",
                        "Usage: File.exists(path)",
                    )));
                }

                self.output.push_str("{ let __arg = (");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").to_string(); std::path::Path::new(&__arg).exists() }");
            }
            "delete" => {
                // File.delete(path) returns (Option<bool>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.delete requires exactly 1 argument",
                        "Usage: File.delete(path)",
                    )));
                }

                self.output.push_str("(match std::fs::remove_file(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File delete error: {}\", e)) })");
            }
            "copy" => {
                // File.copy(src, dest) returns (Option<bool>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.copy requires exactly 2 arguments",
                        "Usage: File.copy(src, dest)",
                    )));
                }

                self.output.push_str("match std::fs::copy(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File copy error: {}\", e)) }");
            }
            "move" => {
                // File.move(src, dest) returns (Option<bool>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.move requires exactly 2 arguments",
                        "Usage: File.move(src, dest)",
                    )));
                }

                self.output.push_str("match std::fs::rename(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File move error: {}\", e)) }");
            }
            "size" => {
                // File.size(path) returns (Option<i64>, String)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.size requires exactly 1 argument",
                        "Usage: File.size(path)",
                    )));
                }

                self.output.push_str("match std::fs::metadata(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(m) => (Some(m.len() as i64), String::new()), Err(e) => (None, format!(\"File size error: {}\", e)) }");
            }
            "extension" => {
                // File.extension(path) returns string (no error binding)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.extension requires exactly 1 argument",
                        "Usage: File.extension(path)",
                    )));
                }

                self.output.push_str("{ let __arg = (");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").to_string(); std::path::Path::new(&__arg).extension().and_then(|e| e.to_str()).unwrap_or(\"\").to_string() }");
            }
            "readLines" => {
                // File.readLines(path) returns (Option<Vec<String>>, String)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.readLines requires exactly 1 argument",
                        "Usage: File.readLines(path)",
                    )));
                }

                self.output.push_str("match std::fs::read_to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(content) => (Some(content.lines().map(|l| l.to_string()).collect::<Vec<String>>()), String::new()), Err(e) => (None, format!(\"File readLines error: {}\", e)) }");
            }
            "writeLines" => {
                // File.writeLines(path, lines) returns (Option<bool>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "File.writeLines requires exactly 2 arguments",
                        "Usage: File.writeLines(path, lines)",
                    )));
                }

                self.output.push_str("match std::fs::write(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".join(\"\\n\")) { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File writeLines error: {}\", e)) }");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown File function: {}", method_call.method),
                    "Available File functions: read, write, append, exists, delete, copy, move, size, extension, readLines, writeLines",
                )));
            }
        }

        Ok(())
    }

    fn generate_dir_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Dir functions: list, isDir
        match method_call.method.as_str() {
            "list" => {
                // Dir.list(path) returns ([string], String) - error binding
                // Returns list of file/directory names in the given path
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Dir.list requires exactly 1 argument",
                        "Usage: Dir.list(path)",
                    )));
                }

                self.output.push_str("match std::fs::read_dir(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(entries) => { let mut names: Vec<String> = entries.filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string())).collect(); names.sort(); (Some(names), String::new()) }, Err(e) => (None, format!(\"Dir.list error: {}\", e)) }");
            }
            "isDir" => {
                // Dir.isDir(path) returns bool (no error binding, like File.exists)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Dir.isDir requires exactly 1 argument",
                        "Usage: Dir.isDir(path)",
                    )));
                }

                self.output.push_str("{ let __arg = (");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").to_string(); std::path::Path::new(&__arg).is_dir() }");
            }
            "exists" => {
                // Dir.exists(path) returns bool (no error binding)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Dir.exists requires exactly 1 argument",
                        "Usage: Dir.exists(path)",
                    )));
                }

                self.output.push_str("{ let __arg = (");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").to_string(); let p = std::path::Path::new(&__arg); p.exists() && p.is_dir() }");
            }
            "create" => {
                // Dir.create(path) returns (Option<bool>, String)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Dir.create requires exactly 1 argument",
                        "Usage: Dir.create(path)",
                    )));
                }

                self.output.push_str("match std::fs::create_dir_all(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"Dir.create error: {}\", e)) }");
            }
            "delete" => {
                // Dir.delete(path) returns (Option<bool>, String)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Dir.delete requires exactly 1 argument",
                        "Usage: Dir.delete(path)",
                    )));
                }

                self.output.push_str("match std::fs::remove_dir_all(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"Dir.delete error: {}\", e)) }");
            }
            "listRecursive" | "walk" => {
                // Dir.listRecursive(path) / Dir.walk(path) returns ([string], String) - error binding
                // Returns all files recursively
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        &format!("Dir.{} requires exactly 1 argument", method_call.method),
                        &format!("Usage: Dir.{}(path)", method_call.method),
                    )));
                }

                self.output.push_str("{ fn walk_dir(dir: &std::path::Path, result: &mut Vec<String>, base: &std::path::Path) -> Result<(), std::io::Error> { for entry in std::fs::read_dir(dir)? { let entry = entry?; let path = entry.path(); if let Some(rel) = path.strip_prefix(base).ok().and_then(|p| p.to_str()) { result.push(rel.to_string()); } if path.is_dir() { walk_dir(&path, result, base)?; } } Ok(()) } let __arg = (");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").to_string(); let base = std::path::Path::new(&__arg); let mut names = Vec::new(); match walk_dir(base, &mut names, base) { Ok(_) => { names.sort(); (Some(names), String::new()) }, Err(e) => (None, format!(\"Dir.");
                self.output.push_str(method_call.method.as_str());
                self.output.push_str(" error: {}\", e)) } }");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Dir function: {}", method_call.method),
                    "Available Dir functions: list, isDir, exists, create, delete, listRecursive, walk",
                )));
            }
        }

        Ok(())
    }

    fn generate_http_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // HTTP functions: get, post, put, delete
        // All return (Option<LivaHttpResponse>, Option<String>)
        match method_call.method.as_str() {
            "get" => {
                // HTTP.get(url) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "HTTP.get requires exactly 1 argument",
                        "Usage: HTTP.get(url)",
                    )));
                }

                self.output.push_str("liva_rt::liva_http_get(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string())");
            }
            "post" => {
                // HTTP.post(url, body) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "HTTP.post requires exactly 2 arguments",
                        "Usage: HTTP.post(url, body)",
                    )));
                }

                self.output.push_str("liva_rt::liva_http_post(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string(), ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".to_string())");
            }
            "put" => {
                // HTTP.put(url, body) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "HTTP.put requires exactly 2 arguments",
                        "Usage: HTTP.put(url, body)",
                    )));
                }

                self.output.push_str("liva_rt::liva_http_put(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string(), ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".to_string())");
            }
            "delete" => {
                // HTTP.delete(url) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "HTTP.delete requires exactly 1 argument",
                        "Usage: HTTP.delete(url)",
                    )));
                }

                self.output.push_str("liva_rt::liva_http_delete(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string())");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown HTTP function: {}", method_call.method),
                    "Available HTTP functions: get, post, put, delete",
                )));
            }
        }

        Ok(())
    }

    fn generate_sys_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Sys functions: args, env, exit
        match method_call.method.as_str() {
            "args" => {
                // Sys.args() returns [string] - command line arguments
                // Returns all args including program name at index 0
                self.output
                    .push_str("std::env::args().collect::<Vec<String>>()");
            }
            "env" => {
                // Sys.env(key) returns string - environment variable value
                // Returns empty string if not found
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Sys.env requires exactly 1 argument",
                        "Usage: Sys.env(\"VAR_NAME\")",
                    )));
                }

                self.output.push_str("std::env::var(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").unwrap_or_default()");
            }
            "exit" => {
                // Sys.exit(code) - exit program with code
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Sys.exit requires exactly 1 argument",
                        "Usage: Sys.exit(0)",
                    )));
                }

                self.output.push_str("std::process::exit(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(" as i32)");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Sys function: {}", method_call.method),
                    "Available Sys functions: args, env, exit",
                )));
            }
        }

        Ok(())
    }

    /// Generate Log module function calls (Log.info, Log.warn, Log.error, Log.debug, Log.setLevel)
    fn generate_log_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "info" | "warn" | "error" | "debug" => {
                let level: u8 = match method_call.method.as_str() {
                    "debug" => 0,
                    "info" => 1,
                    "warn" => 2,
                    "error" => 3,
                    _ => unreachable!(),
                };
                let label = method_call.method.to_uppercase();

                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        &format!("Log.{} requires at least 1 argument", method_call.method),
                        &format!("Usage: Log.{}(\"message\", arg1, arg2, ...)", method_call.method),
                    )));
                }

                // Classify arguments: Scalar, InlineMap (≤3 keys), TableMap (4+ keys), TableArray, Json
                #[derive(Debug)]
                enum ArgKind { Scalar, InlineMap, TableMap, TableArray, Json }

                let arg_kinds: Vec<ArgKind> = method_call.args.iter().map(|arg| {
                    match arg {
                        Expr::MapLiteral(entries) if entries.len() >= 4 => ArgKind::TableMap,
                        Expr::ArrayLiteral(elements) if !elements.is_empty()
                            && elements.iter().all(|e| matches!(e, Expr::MapLiteral(_))) =>
                        {
                            ArgKind::TableArray
                        }
                        Expr::MapLiteral(_) => ArgKind::InlineMap,
                        _ => ArgKind::Scalar,
                    }
                }).collect();

                // Post-pass: reclassify Scalar args that are JsonValue as Json
                let arg_kinds: Vec<ArgKind> = arg_kinds.into_iter().enumerate().map(|(i, kind)| {
                    if matches!(kind, ArgKind::Scalar) && self.is_json_value_expr(&method_call.args[i]) {
                        ArgKind::Json
                    } else {
                        kind
                    }
                }).collect();

                let has_tables = arg_kinds.iter().any(|k| matches!(k, ArgKind::TableMap | ArgKind::TableArray | ArgKind::Json));

                // Build format string for message (non-table args only)
                let mut fmt_parts: Vec<String> = Vec::new();
                for (arg, kind) in method_call.args.iter().zip(arg_kinds.iter()) {
                    match kind {
                        ArgKind::Scalar => fmt_parts.push("{}".to_string()),
                        ArgKind::InlineMap => {
                            if let Expr::MapLiteral(entries) = arg {
                                let parts: Vec<String> = entries.iter().map(|(k, _)| {
                                    if let Expr::Literal(Literal::String(key)) = k {
                                        format!("{}: {{}}", key)
                                    } else {
                                        "?: {}".to_string()
                                    }
                                }).collect();
                                fmt_parts.push(format!("{{{{{}}}}}", parts.join(", ")));
                            }
                        }
                        _ => {} // Table args don't contribute to message format
                    }
                }

                if has_tables {
                    self.output.push_str("{ ");
                }

                // Generate liva_log() call
                write!(self.output, "liva_log({}, \"{}\", ", level, label).unwrap();

                if fmt_parts.is_empty() {
                    self.output.push_str("\"\"");
                } else {
                    let fmt_str = fmt_parts.join(" ");
                    write!(self.output, "&format!(\"{}\"", fmt_str).unwrap();

                    // Generate format arguments for non-table args
                    for (arg, kind) in method_call.args.iter().zip(arg_kinds.iter()) {
                        match kind {
                            ArgKind::Scalar => {
                                self.output.push_str(" , ");
                                self.generate_expr(arg)?;
                            }
                            ArgKind::InlineMap => {
                                if let Expr::MapLiteral(entries) = arg {
                                    for (_, v) in entries {
                                        self.output.push_str(", ");
                                        self.generate_expr(v)?;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    self.output.push(')');
                }
                self.output.push(')'); // close liva_log(

                if has_tables {
                    // Generate table calls for TableMap and TableArray args
                    for (arg, kind) in method_call.args.iter().zip(arg_kinds.iter()) {
                        match kind {
                            ArgKind::TableMap => {
                                if let Expr::MapLiteral(entries) = arg {
                                    write!(self.output, "; liva_log_table_kv({}, &[", level).unwrap();
                                    for (i, (k, _)) in entries.iter().enumerate() {
                                        if i > 0 { self.output.push_str(", "); }
                                        if let Expr::Literal(Literal::String(key)) = k {
                                            write!(self.output, "\"{}\"", key).unwrap();
                                        } else {
                                            self.output.push_str("\"?\"");
                                        }
                                    }
                                    self.output.push_str("], &[");
                                    for (i, (_, v)) in entries.iter().enumerate() {
                                        if i > 0 { self.output.push_str(", "); }
                                        self.output.push_str("format!(\"{}\", ");
                                        self.generate_expr(v)?;
                                        self.output.push(')');
                                    }
                                    self.output.push_str("])");
                                }
                            }
                            ArgKind::TableArray => {
                                if let Expr::ArrayLiteral(elements) = arg {
                                    if let Some(Expr::MapLiteral(first_entries)) = elements.first() {
                                        write!(self.output, "; liva_log_table_rows({}, &[", level).unwrap();
                                        for (i, (k, _)) in first_entries.iter().enumerate() {
                                            if i > 0 { self.output.push_str(", "); }
                                            if let Expr::Literal(Literal::String(key)) = k {
                                                write!(self.output, "\"{}\"", key).unwrap();
                                            } else {
                                                self.output.push_str("\"?\"");
                                            }
                                        }
                                        self.output.push_str("], &[");
                                        for (j, elem) in elements.iter().enumerate() {
                                            if j > 0 { self.output.push_str(", "); }
                                            if let Expr::MapLiteral(row_entries) = elem {
                                                self.output.push_str("vec![");
                                                for (i, (_, v)) in row_entries.iter().enumerate() {
                                                    if i > 0 { self.output.push_str(", "); }
                                                    self.output.push_str("format!(\"{}\", ");
                                                    self.generate_expr(v)?;
                                                    self.output.push(')');
                                                }
                                                self.output.push(']');
                                            }
                                        }
                                        self.output.push_str("])");
                                    }
                                }
                            }
                            ArgKind::Json => {
                                write!(self.output, "; liva_log_json({}, ", level).unwrap();
                                // Check if the arg is an Option<JsonValue> (from JSON.parse)
                                let is_option = if let Expr::Identifier(name) = arg {
                                    self.option_value_vars.contains(name)
                                } else {
                                    false
                                };
                                if is_option {
                                    self.output.push_str("&");
                                    // Temporarily remove from option_value_vars to avoid double-unwrap
                                    let name = if let Expr::Identifier(n) = arg { n.clone() } else { String::new() };
                                    self.option_value_vars.remove(&name);
                                    self.generate_expr(arg)?;
                                    self.option_value_vars.insert(name);
                                    self.output.push_str(".as_ref().unwrap_or(&liva_rt::JsonValue::default())");
                                } else {
                                    self.output.push_str("&");
                                    self.generate_expr(arg)?;
                                }
                                self.output.push(')');
                            }
                            _ => {}
                        }
                    }
                    self.output.push_str("; }");
                }
            }
            "setLevel" => {
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Log.setLevel requires exactly 1 argument",
                        "Usage: Log.setLevel(\"info\") — levels: debug, info, warn, error",
                    )));
                }

                self.output.push_str("liva_log_set_level(&format!(\"{}\", ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("))");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Log function: {}", method_call.method),
                    "Available Log functions: info, warn, error, debug, setLevel",
                )));
            }
        }

        Ok(())
    }

    fn generate_config_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "load" => {
                // Config.load(path) returns (Option<HashMap<String, String>>, String)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Config.load requires exactly 1 argument",
                        "Usage: Config.load(\"path/to/.env\")",
                    )));
                }
                self.output.push_str("liva_config_load(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(")");
            }
            "get" => {
                // Config.get(map, key) returns (Option<String>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Config.get requires exactly 2 arguments",
                        "Usage: Config.get(config, \"KEY\")",
                    )));
                }
                self.output.push_str("liva_config_get(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(")");
            }
            "getInt" => {
                // Config.getInt(map, key) returns (Option<i32>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Config.getInt requires exactly 2 arguments",
                        "Usage: Config.getInt(config, \"PORT\")",
                    )));
                }
                self.output.push_str("liva_config_get_int(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(")");
            }
            "getBool" => {
                // Config.getBool(map, key) returns (Option<bool>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Config.getBool requires exactly 2 arguments",
                        "Usage: Config.getBool(config, \"VERBOSE\")",
                    )));
                }
                self.output.push_str("liva_config_get_bool(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(")");
            }
            "getAll" => {
                // Config.getAll(map) returns BTreeMap<String, String> as Map<string, string>
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Config.getAll requires exactly 1 argument",
                        "Usage: Config.getAll(config)",
                    )));
                }
                self.output.push_str("liva_config_get_all(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(")");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Config function: {}", method_call.method),
                    "Available Config functions: load, get, getInt, getBool, getAll",
                )));
            }
        }

        Ok(())
    }

    fn generate_regex_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Regex functions: test, match, findAll, replace, split
        // All use the `regex` crate (auto-injected when Regex.* is used)
        match method_call.method.as_str() {
            "test" => {
                // Regex.test(pattern, text) → bool
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Regex.test requires exactly 2 arguments",
                        "Usage: Regex.test(pattern, text)",
                    )));
                }

                self.output.push_str("regex::Regex::new(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").map(|re| re.is_match(&");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(")).unwrap_or(false)");
            }
            "match" => {
                // Regex.match(pattern, text) → (Option<String>, String)
                // Returns first match or error
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Regex.match requires exactly 2 arguments",
                        "Usage: Regex.match(pattern, text)",
                    )));
                }

                self.output.push_str("match regex::Regex::new(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(re) => match re.find(&");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") { Some(m) => (Some(m.as_str().to_string()), String::new()), None => (None, String::new()) }, Err(e) => (None, format!(\"Regex error: {}\", e)) }");
            }
            "findAll" => {
                // Regex.findAll(pattern, text) → [string]
                // Returns all matches (empty array if no matches or invalid pattern)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Regex.findAll requires exactly 2 arguments",
                        "Usage: Regex.findAll(pattern, text)",
                    )));
                }

                self.output.push_str("regex::Regex::new(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").map(|re| re.find_iter(&");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(").map(|m| m.as_str().to_string()).collect::<Vec<String>>()).unwrap_or_default()");
            }
            "replace" => {
                // Regex.replace(pattern, text, replacement) → string
                if method_call.args.len() != 3 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Regex.replace requires exactly 3 arguments",
                        "Usage: Regex.replace(pattern, text, replacement)",
                    )));
                }

                self.output.push_str("{ let __repl = (");
                self.generate_expr(&method_call.args[2])?;
                self.output.push_str(").to_string(); regex::Regex::new(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").map(|re| re.replace_all(&");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(", __repl.as_str()).to_string()).unwrap_or_else(|_| ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".to_string()) }");
            }
            "split" => {
                // Regex.split(pattern, text) → [string]
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Regex.split requires exactly 2 arguments",
                        "Usage: Regex.split(pattern, text)",
                    )));
                }

                self.output.push_str("regex::Regex::new(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").map(|re| re.split(&");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(").map(|s| s.to_string()).collect::<Vec<String>>()).unwrap_or_else(|_| vec![");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".to_string()])");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Regex function: {}", method_call.method),
                    "Available Regex functions: test, match, findAll, replace, split",
                )));
            }
        }

        Ok(())
    }

    fn generate_date_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "now" => {
                // Date.now() → chrono::Local::now().naive_local()
                if !method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.now takes no arguments",
                        "Usage: Date.now()",
                    )));
                }
                self.output.push_str("chrono::Local::now().naive_local()");
            }
            "new" => {
                // Date.new(year, month, day) → NaiveDate::from_ymd_opt(y,m,d).unwrap().and_hms_opt(0,0,0).unwrap()
                // Date.new(year, month, day, hour, minute, second) → full datetime
                if method_call.args.len() != 3 && method_call.args.len() != 6 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.new requires 3 or 6 arguments",
                        "Usage: Date.new(year, month, day) or Date.new(year, month, day, hour, minute, second)",
                    )));
                }
                self.output.push_str("chrono::NaiveDate::from_ymd_opt(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(" as u32, ");
                self.generate_expr(&method_call.args[2])?;
                self.output.push_str(" as u32).unwrap().and_hms_opt(");
                if method_call.args.len() == 6 {
                    self.generate_expr(&method_call.args[3])?;
                    self.output.push_str(" as u32, ");
                    self.generate_expr(&method_call.args[4])?;
                    self.output.push_str(" as u32, ");
                    self.generate_expr(&method_call.args[5])?;
                    self.output.push_str(" as u32");
                } else {
                    self.output.push_str("0, 0, 0");
                }
                self.output.push_str(").unwrap()");
            }
            "parse" => {
                // Date.parse(str, pattern) → (Option<NaiveDateTime>, String)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.parse requires exactly 2 arguments",
                        "Usage: Date.parse(\"2026-03-11\", \"%Y-%m-%d\")",
                    )));
                }
                // Convert Liva-style patterns to chrono strftime patterns
                self.output.push_str("{ let __liva_pattern = ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".replace(\"YYYY\", \"%Y\").replace(\"MM\", \"%m\").replace(\"DD\", \"%d\").replace(\"HH\", \"%H\").replace(\"mm\", \"%M\").replace(\"ss\", \"%S\"); ");
                self.output.push_str("match chrono::NaiveDateTime::parse_from_str(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &__liva_pattern) { Ok(dt) => (Some(dt), String::new()), Err(_) => match chrono::NaiveDate::parse_from_str(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &__liva_pattern) { Ok(d) => (Some(d.and_hms_opt(0, 0, 0).unwrap()), String::new()), Err(e) => (None, format!(\"Date parse error: {}\", e)) } } }");
            }
            "timestamp" => {
                // Date.timestamp() → chrono::Local::now().timestamp_millis() as i32
                if !method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.timestamp takes no arguments",
                        "Usage: Date.timestamp()",
                    )));
                }
                self.output.push_str("(chrono::Local::now().timestamp_millis() as i32)");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Date function: {}", method_call.method),
                    "Available: Date.now(), Date.new(y,m,d), Date.parse(str, pattern), Date.timestamp()",
                )));
            }
        }

        Ok(())
    }

    fn generate_date_method_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "format" => {
                // d.format(pattern) → d.format(chrono_pattern).to_string()
                // Convert Liva-style patterns (YYYY, MM, DD, HH, mm, ss) to chrono strftime (%Y, %m, %d, %H, %M, %S)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.format requires exactly 1 argument",
                        "Usage: d.format(\"DD/MM/YYYY\")",
                    )));
                }
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".format(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".replace(\"YYYY\", \"%Y\").replace(\"MM\", \"%m\").replace(\"DD\", \"%d\").replace(\"HH\", \"%H\").replace(\"mm\", \"%M\").replace(\"ss\", \"%S\")).to_string()");
            }
            "add" => {
                // d.add(n, unit) → d + chrono::Duration::xxx(n)
                // units: "days", "hours", "minutes", "seconds", "weeks"
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.add requires exactly 2 arguments",
                        "Usage: d.add(7, \"days\")",
                    )));
                }
                // Generate: d + chrono::Duration::days(n as i64) (etc.)
                self.output.push_str("{ let __liva_unit = (");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(").to_string(); let __liva_n = ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(" as i64; let __liva_dur = match __liva_unit.as_str() { ");
                self.output.push_str("\"days\" => chrono::Duration::days(__liva_n), ");
                self.output.push_str("\"hours\" => chrono::Duration::hours(__liva_n), ");
                self.output.push_str("\"minutes\" => chrono::Duration::minutes(__liva_n), ");
                self.output.push_str("\"seconds\" => chrono::Duration::seconds(__liva_n), ");
                self.output.push_str("\"weeks\" => chrono::Duration::weeks(__liva_n), ");
                self.output.push_str("_ => chrono::Duration::days(__liva_n), }; ");
                self.generate_expr(&method_call.object)?;
                self.output.push_str(" + __liva_dur }");
            }
            "diff" => {
                // d.diff(other, unit) → difference in specified unit
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.diff requires exactly 2 arguments",
                        "Usage: d.diff(other, \"days\")",
                    )));
                }
                self.output.push_str("{ let __liva_diff = ");
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".signed_duration_since(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("); let __liva_unit = (");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(").to_string(); (match __liva_unit.as_str() { ");
                self.output.push_str("\"days\" => __liva_diff.num_days(), ");
                self.output.push_str("\"hours\" => __liva_diff.num_hours(), ");
                self.output.push_str("\"minutes\" => __liva_diff.num_minutes(), ");
                self.output.push_str("\"seconds\" => __liva_diff.num_seconds(), ");
                self.output.push_str("\"weeks\" => __liva_diff.num_weeks(), ");
                self.output.push_str("\"years\" => __liva_diff.num_days() / 365, ");
                self.output.push_str("\"months\" => __liva_diff.num_days() / 30, ");
                self.output.push_str("_ => __liva_diff.num_days(), }) as i32 }");
            }
            "toString" => {
                // d.toString() → d.format("%Y-%m-%dT%H:%M:%S").to_string()
                if !method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Date.toString takes no arguments",
                        "Usage: d.toString()",
                    )));
                }
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".format(\"%Y-%m-%dT%H:%M:%S\").to_string()");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Date method: {}", method_call.method),
                    "Available: format(pattern), add(n, unit), diff(other, unit), toString()",
                )));
            }
        }

        Ok(())
    }

    fn generate_csv_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "read" => {
                // CSV.read(path) or CSV.read(path, separator: "\t")
                // Returns (Option<Vec<Vec<String>>>, String) — fallible
                if method_call.args.is_empty() || method_call.args.len() > 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.read requires 1-2 arguments",
                        "Usage: CSV.read(path) or CSV.read(path, separator)",
                    )));
                }
                // Determine separator — default ','
                let sep = if method_call.args.len() == 2 {
                    // Second arg is separator string
                    "custom"
                } else {
                    "comma"
                };
                self.output.push_str("{\n");
                // Generate inline CSV parser as a closure
                self.output.push_str("let __liva_sep: char = ");
                if sep == "custom" {
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(".chars().next().unwrap_or(',');\n");
                } else {
                    self.output.push_str("',';\n");
                }
                self.output.push_str("match std::fs::read_to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") {\n");
                self.output.push_str("Ok(content) => {\n");
                self.output.push_str("let rows: Vec<Vec<String>> = content.lines().filter(|l| !l.is_empty()).map(|line| {\n");
                self.output.push_str("let mut fields = Vec::new();\n");
                self.output.push_str("let mut current = String::new();\n");
                self.output.push_str("let mut in_quotes = false;\n");
                self.output.push_str("let mut chars = line.chars().peekable();\n");
                self.output.push_str("while let Some(c) = chars.next() {\n");
                self.output.push_str("if in_quotes {\n");
                self.output.push_str("if c == '\"' { if chars.peek() == Some(&'\"') { current.push('\"'); chars.next(); } else { in_quotes = false; } } else { current.push(c); }\n");
                self.output.push_str("} else if c == '\"' { in_quotes = true;\n");
                self.output.push_str("} else if c == __liva_sep { fields.push(current.trim().to_string()); current = String::new();\n");
                self.output.push_str("} else { current.push(c); }\n");
                self.output.push_str("}\n");
                self.output.push_str("fields.push(current.trim().to_string());\n");
                self.output.push_str("fields\n");
                self.output.push_str("}).collect();\n");
                self.output.push_str("(Some(rows), String::new())\n");
                self.output.push_str("},\n");
                self.output.push_str("Err(e) => (None, format!(\"CSV.read error: {}\", e))\n");
                self.output.push_str("}\n");
                self.output.push_str("}");
            }
            "write" => {
                // CSV.write(path, data) — data is Vec<Vec<String>>
                // Returns (Option<bool>, String) — fallible
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.write requires exactly 2 arguments",
                        "Usage: CSV.write(path, data)",
                    )));
                }
                self.output.push_str("{\n");
                self.output.push_str("let __csv_content: String = ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".iter().map(|row| {\n");
                self.output.push_str("row.iter().map(|field| {\n");
                self.output.push_str("if field.contains(',') || field.contains('\"') || field.contains('\\n') {\n");
                self.output.push_str("format!(\"\\\"{}\\\"\" , field.replace('\"', \"\\\"\\\"\"))\n");
                self.output.push_str("} else { field.clone() }\n");
                self.output.push_str("}).collect::<Vec<_>>().join(\",\")\n");
                self.output.push_str("}).collect::<Vec<_>>().join(\"\\n\");\n");
                self.output.push_str("match std::fs::write(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &__csv_content) { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"CSV.write error: {}\", e)) }\n");
                self.output.push_str("}");
            }
            "readTable" => {
                // CSV.readTable(path) — reads CSV with first row as headers
                // Returns (Option<Vec<std::collections::HashMap<String,String>>>, String) — fallible
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.readTable requires exactly 1 argument",
                        "Usage: CSV.readTable(path)",
                    )));
                }
                self.output.push_str("{\n");
                self.output.push_str("match std::fs::read_to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") {\n");
                self.output.push_str("Ok(content) => {\n");
                // Inline CSV parser
                self.output.push_str("let __rows: Vec<Vec<String>> = content.lines().filter(|l| !l.is_empty()).map(|line| {\n");
                self.output.push_str("let mut fields = Vec::new();\n");
                self.output.push_str("let mut current = String::new();\n");
                self.output.push_str("let mut in_quotes = false;\n");
                self.output.push_str("let mut chars = line.chars().peekable();\n");
                self.output.push_str("while let Some(c) = chars.next() {\n");
                self.output.push_str("if in_quotes {\n");
                self.output.push_str("if c == '\"' { if chars.peek() == Some(&'\"') { current.push('\"'); chars.next(); } else { in_quotes = false; } } else { current.push(c); }\n");
                self.output.push_str("} else if c == '\"' { in_quotes = true;\n");
                self.output.push_str("} else if c == ',' { fields.push(current.trim().to_string()); current = String::new();\n");
                self.output.push_str("} else { current.push(c); }\n");
                self.output.push_str("}\n");
                self.output.push_str("fields.push(current.trim().to_string());\n");
                self.output.push_str("fields\n");
                self.output.push_str("}).collect();\n");
                // First row = headers, rest = data rows as HashMap
                self.output.push_str("if __rows.is_empty() { (Some(Vec::new()), String::new()) } else {\n");
                self.output.push_str("let headers = &__rows[0];\n");
                self.output.push_str("let table: Vec<std::collections::HashMap<String, String>> = __rows[1..].iter().map(|row| {\n");
                self.output.push_str("let mut map = std::collections::HashMap::new();\n");
                self.output.push_str("for (i, header) in headers.iter().enumerate() {\n");
                self.output.push_str("map.insert(header.clone(), row.get(i).cloned().unwrap_or_default());\n");
                self.output.push_str("}\n");
                self.output.push_str("map\n");
                self.output.push_str("}).collect();\n");
                self.output.push_str("(Some(table), String::new())\n");
                self.output.push_str("}\n");
                self.output.push_str("},\n");
                self.output.push_str("Err(e) => (None, format!(\"CSV.readTable error: {}\", e))\n");
                self.output.push_str("}\n");
                self.output.push_str("}");
            }
            "writeTable" => {
                // CSV.writeTable(path, table) — table is Vec<HashMap<String,String>>
                // Returns (Option<bool>, String) — fallible
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.writeTable requires exactly 2 arguments",
                        "Usage: CSV.writeTable(path, table)",
                    )));
                }
                self.output.push_str("{\n");
                self.output.push_str("let __table = &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(";\n");
                // Collect headers from the first row's keys (sorted for determinism)
                self.output.push_str("if __table.is_empty() { match std::fs::write(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", \"\") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"CSV.writeTable error: {}\", e)) } } else {\n");
                self.output.push_str("let mut __headers: Vec<String> = __table[0].keys().cloned().collect();\n");
                self.output.push_str("__headers.sort();\n");
                self.output.push_str("let mut __csv_lines: Vec<String> = vec![__headers.iter().map(|h| {\n");
                self.output.push_str("if h.contains(',') || h.contains('\"') || h.contains('\\n') { format!(\"\\\"{}\\\"\" , h.replace('\"', \"\\\"\\\"\")) } else { h.clone() }\n");
                self.output.push_str("}).collect::<Vec<_>>().join(\",\")];\n");
                self.output.push_str("for row in __table.iter() {\n");
                self.output.push_str("__csv_lines.push(__headers.iter().map(|h| {\n");
                self.output.push_str("let val = row.get(h).cloned().unwrap_or_default();\n");
                self.output.push_str("if val.contains(',') || val.contains('\"') || val.contains('\\n') { format!(\"\\\"{}\\\"\" , val.replace('\"', \"\\\"\\\"\")) } else { val }\n");
                self.output.push_str("}).collect::<Vec<_>>().join(\",\"));\n");
                self.output.push_str("}\n");
                self.output.push_str("let __csv_content = __csv_lines.join(\"\\n\");\n");
                self.output.push_str("match std::fs::write(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &__csv_content) { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"CSV.writeTable error: {}\", e)) }\n");
                self.output.push_str("}\n");
                self.output.push_str("}");
            }
            "parse" => {
                // CSV.parse(text) → Vec<Vec<String>> — pure, no error binding
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.parse requires exactly 1 argument",
                        "Usage: CSV.parse(text)",
                    )));
                }
                self.output.push_str("{\n");
                self.output.push_str("let __text: &str = &");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(";\n");
                self.output.push_str("__text.lines().filter(|l| !l.is_empty()).map(|line| {\n");
                self.output.push_str("let mut fields = Vec::new();\n");
                self.output.push_str("let mut current = String::new();\n");
                self.output.push_str("let mut in_quotes = false;\n");
                self.output.push_str("let mut chars = line.chars().peekable();\n");
                self.output.push_str("while let Some(c) = chars.next() {\n");
                self.output.push_str("if in_quotes {\n");
                self.output.push_str("if c == '\"' { if chars.peek() == Some(&'\"') { current.push('\"'); chars.next(); } else { in_quotes = false; } } else { current.push(c); }\n");
                self.output.push_str("} else if c == '\"' { in_quotes = true;\n");
                self.output.push_str("} else if c == ',' { fields.push(current.trim().to_string()); current = String::new();\n");
                self.output.push_str("} else { current.push(c); }\n");
                self.output.push_str("}\n");
                self.output.push_str("fields.push(current.trim().to_string());\n");
                self.output.push_str("fields\n");
                self.output.push_str("}).collect::<Vec<Vec<String>>>()\n");
                self.output.push_str("}");
            }
            "stringify" => {
                // CSV.stringify(rows) → String — pure, no error binding
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.stringify requires exactly 1 argument",
                        "Usage: CSV.stringify(rows)",
                    )));
                }
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".iter().map(|row| {\n");
                self.output.push_str("row.iter().map(|field| {\n");
                self.output.push_str("if field.contains(',') || field.contains('\"') || field.contains('\\n') {\n");
                self.output.push_str("format!(\"\\\"{}\\\"\" , field.replace('\"', \"\\\"\\\"\"))\n");
                self.output.push_str("} else { field.clone() }\n");
                self.output.push_str("}).collect::<Vec<_>>().join(\",\")\n");
                self.output.push_str("}).collect::<Vec<_>>().join(\"\\n\")");
            }
            "headers" => {
                // CSV.headers(table) → Vec<String> — gets sorted keys from first row
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.headers requires exactly 1 argument",
                        "Usage: CSV.headers(table)",
                    )));
                }
                self.output.push_str("{ let __t = &");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("; if __t.is_empty() { Vec::<String>::new() } else { let mut __h: Vec<String> = __t[0].keys().cloned().collect(); __h.sort(); __h } }");
            }
            "column" => {
                // CSV.column(table, colName) → Vec<String> — extract a column from table
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "CSV.column requires exactly 2 arguments",
                        "Usage: CSV.column(table, columnName)",
                    )));
                }
                self.output.push_str("{ let __col_name = &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str("; ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".iter().map(|row| row.get(__col_name.as_str()).cloned().unwrap_or_default()).collect::<Vec<String>>() }");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown CSV function: {}", method_call.method),
                    "Available: read, write, readTable, writeTable, parse, stringify, headers, column",
                )));
            }
        }

        Ok(())
    }

    fn generate_server_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "create" => {
                // Server.create() → axum::Router::new()
                if !method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Server.create takes no arguments",
                        "Usage: let app = Server.create()",
                    )));
                }
                self.output.push_str("axum::Router::new()");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Server function: {}", method_call.method),
                    "Available: create()",
                )));
            }
        }

        Ok(())
    }

    fn generate_server_method_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "get" | "post" | "put" | "delete" => {
                // app.get("/path", handler) → app = app.route("/path", axum::routing::get(handler))
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        &format!("Server.{} requires exactly 2 arguments", method_call.method),
                        &format!("Usage: app.{}(path, handler)", method_call.method),
                    )));
                }

                let http_method = method_call.method.as_str();
                let var_name = if let Expr::Identifier(name) = method_call.object.as_ref() {
                    self.sanitize_name(name)
                } else {
                    "app".to_string()
                };

                // Determine if path has params (contains :)
                let path_str = if let Expr::Literal(Literal::String(s)) = &method_call.args[0] {
                    Some(s.clone())
                } else {
                    None
                };

                let has_params = path_str.as_ref().map_or(false, |p| p.contains(':'));

                // Bug #81 fix: Clone Arc<Mutex<Connection>> db vars before each route handler
                // so each async closure gets its own Arc clone
                if !self.db_vars.is_empty() {
                    self.output.push_str("{ ");
                    for db_var in self.db_vars.iter() {
                        write!(self.output, "let {} = {}.clone(); ", db_var, db_var).unwrap();
                    }
                }

                // Generate: var = var.route("path", axum::routing::METHOD(|...| async move { ... }))
                write!(self.output, "{} = {}.route(", var_name, var_name).unwrap();

                // Generate path — Bug #85 fix: convert :param to {param} for axum 0.8+ syntax
                if let Some(ref path) = path_str {
                    // Convert :param segments to {param} for axum 0.8
                    let converted = path.split('/')
                        .map(|segment| {
                            if let Some(stripped) = segment.strip_prefix(':') {
                                format!("{{{}}}", stripped)
                            } else {
                                segment.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("/");
                    write!(self.output, "\"{}\"", converted).unwrap();
                } else {
                    self.generate_expr(&method_call.args[0])?;
                }
                write!(self.output, ", axum::routing::{}(", http_method).unwrap();

                // Generate the handler closure
                if let Expr::Lambda(lambda) = &method_call.args[1] {
                    // Generate axum handler
                    if has_params && (http_method == "post" || http_method == "put") {
                        // With path params AND body
                        self.output.push_str("|axum::extract::Path(__params): axum::extract::Path<std::collections::HashMap<String, String>>, body: String| async move {\n");
                    } else if has_params {
                        // With path params only
                        self.output.push_str("|axum::extract::Path(__params): axum::extract::Path<std::collections::HashMap<String, String>>| async move {\n");
                    } else if http_method == "post" || http_method == "put" {
                        // With body only
                        self.output.push_str("|body: String| async move {\n");
                    } else {
                        // No extractors
                        self.output.push_str("|| async move {\n");
                    }

                    // Generate handler body — need to translate req.params, req.body, Response.*
                    // Save the lambda param name as the "req" alias
                    let req_param = if !lambda.params.is_empty() {
                        lambda.params[0].name().unwrap_or("_req").to_string()
                    } else {
                        "_req".to_string()
                    };

                    // Push req param context for the handler body
                    let saved_req_param = self.current_function_name.clone();
                    // We'll store the req param name for resolution during body generation
                    // Use a temporary approach: generate body statements manually
                    self.generate_server_handler_body(&lambda.body, &req_param, has_params, http_method == "post" || http_method == "put")?;

                    self.current_function_name = saved_req_param;

                    self.output.push_str("}))");
                    // Close the db clone scope block
                    if !self.db_vars.is_empty() {
                        self.output.push_str(" }");
                    }
                } else {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Server route handler must be a lambda/function",
                        &format!("Usage: app.{}(path, (req) => {{ ... }})", http_method),
                    )));
                }
            }
            "listen" => {
                // app.listen(port) → start axum server
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Server.listen requires exactly 1 argument",
                        "Usage: app.listen(port)",
                    )));
                }

                let var_name = if let Expr::Identifier(name) = method_call.object.as_ref() {
                    self.sanitize_name(name)
                } else {
                    "app".to_string()
                };

                write!(self.output, "{{ let __addr = format!(\"0.0.0.0:{{}}\", ", ).unwrap();
                self.generate_expr(&method_call.args[0])?;
                write!(self.output, "); let __listener = tokio::net::TcpListener::bind(&__addr).await.unwrap(); axum::serve(__listener, {}).await.unwrap(); }}", var_name).unwrap();
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown server method: {}", method_call.method),
                    "Available: get(path, handler), post(path, handler), put(path, handler), delete(path, handler), listen(port)",
                )));
            }
        }

        Ok(())
    }

    fn generate_server_handler_body(
        &mut self,
        body: &LambdaBody,
        req_param: &str,
        _has_params: bool,
        _has_body: bool,
    ) -> Result<()> {
        // Set the request param so that Member access (req.params, req.body)
        // gets intercepted during normal codegen
        let saved = self.server_request_param.take();
        self.server_request_param = Some(req_param.to_string());

        match body {
            LambdaBody::Block(block) => {
                let len = block.stmts.len();
                for (i, stmt) in block.stmts.iter().enumerate() {
                    let is_last = i == len - 1;
                    self.indent_level += 1;
                    if is_last {
                        // Last statement: generate as expression (no trailing semicolon)
                        // so it becomes the return value of the async handler
                        if let Stmt::Expr(expr_stmt) = stmt {
                            self.write_indent();
                            self.generate_expr(&expr_stmt.expr)?;
                            self.output.push('\n');
                        } else {
                            self.generate_stmt(stmt)?;
                        }
                    } else {
                        self.generate_stmt(stmt)?;
                    }
                    self.indent_level -= 1;
                }
            }
            LambdaBody::Expr(expr) => {
                self.generate_expr(expr)?;
                self.output.push('\n');
            }
        }

        self.server_request_param = saved;
        Ok(())
    }

    fn generate_response_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "json" => {
                // Response.json(data) or Response.json(data, status: 201)
                // Express-like: accepts Map literal, Map var, [Map] var, or string
                // → (StatusCode::OK, axum::Json(...))
                if method_call.args.is_empty() || method_call.args.len() > 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Response.json requires 1-2 arguments",
                        "Usage: Response.json(data) or Response.json(data, statusCode)",
                    )));
                }

                let status = if method_call.args.len() == 2 {
                    "custom"
                } else {
                    "ok"
                };

                self.output.push_str("(");
                if status == "custom" {
                    self.output.push_str("axum::http::StatusCode::from_u16(");
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(" as u16).unwrap_or(axum::http::StatusCode::OK)");
                } else {
                    self.output.push_str("axum::http::StatusCode::OK");
                }
                self.output.push_str(", axum::Json(");

                let arg = &method_call.args[0];

                match arg {
                    // Map literal: Response.json({ "key": "value" })
                    // → serde_json::json!({"key": "value"})
                    Expr::MapLiteral(entries) => {
                        self.output.push_str("serde_json::json!({");
                        for (i, (key, value)) in entries.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(key)?;
                            self.output.push_str(": ");
                            self.generate_expr(value)?;
                        }
                        self.output.push_str("})");
                    }
                    // Object literal: Response.json({ status: "ok" })
                    // → serde_json::json!({"status": "ok"})
                    Expr::ObjectLiteral(fields) => {
                        self.output.push_str("serde_json::json!({");
                        for (i, (key, value)) in fields.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            write!(self.output, "\"{}\"", key).unwrap();
                            self.output.push_str(": ");
                            self.generate_expr(value)?;
                        }
                        self.output.push_str("})");
                    }
                    // Variable: check if it's a Map, [Map], or string
                    Expr::Identifier(name) => {
                        let sname = self.sanitize_name(name);
                        if self.map_vars.contains(&sname) || self.map_array_vars.contains(&sname) {
                            // Map or [Map] → serialize directly via Serialize trait
                            // → serde_json::to_value(&var).unwrap_or_default()
                            self.output.push_str("serde_json::to_value(&");
                            self.output.push_str(&sname);
                            self.output.push_str(").unwrap_or_default()");
                        } else if self.string_vars.contains(&sname) {
                            // String (likely from JSON.stringify) → parse back to Value
                            // → serde_json::from_str::<serde_json::Value>(&var).unwrap_or_default()
                            self.output.push_str("serde_json::from_str::<serde_json::Value>(&");
                            self.output.push_str(&sname);
                            self.output.push_str(").unwrap_or_default()");
                        } else {
                            // Unknown var → try serde_json::to_value (generic fallback)
                            self.output.push_str("serde_json::to_value(&");
                            self.output.push_str(&sname);
                            self.output.push_str(").unwrap_or_default()");
                        }
                    }
                    // String literal: Response.json("raw string")
                    // → serde_json::json!("raw string") — backwards compatible
                    Expr::Literal(Literal::String(_)) => {
                        self.output.push_str("serde_json::json!(");
                        self.generate_expr(arg)?;
                        self.output.push_str(")");
                    }
                    // Anything else: wrap with serde_json::json!()
                    _ => {
                        self.output.push_str("serde_json::json!(");
                        self.generate_expr(arg)?;
                        self.output.push_str(")");
                    }
                }

                self.output.push_str("))");
            }
            "text" => {
                // Response.text(text) or Response.text(text, status)
                // → (StatusCode::OK, text.to_string())
                if method_call.args.is_empty() || method_call.args.len() > 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Response.text requires 1-2 arguments",
                        "Usage: Response.text(text) or Response.text(text, statusCode)",
                    )));
                }

                let status = if method_call.args.len() == 2 {
                    "custom"
                } else {
                    "ok"
                };

                self.output.push_str("(");
                if status == "custom" {
                    self.output.push_str("axum::http::StatusCode::from_u16(");
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(" as u16).unwrap_or(axum::http::StatusCode::OK)");
                } else {
                    self.output.push_str("axum::http::StatusCode::OK");
                }
                self.output.push_str(", ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string())");
            }
            "status" => {
                // Response.status(code) → StatusCode only (for empty responses)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Response.status requires exactly 1 argument",
                        "Usage: Response.status(code)",
                    )));
                }
                self.output.push_str("axum::http::StatusCode::from_u16(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(" as u16).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Response function: {}", method_call.method),
                    "Available: json(data[, status]), text(msg[, status]), status(code)",
                )));
            }
        }

        Ok(())
    }

    fn generate_db_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "open" => {
                // DB.open(path) → (Option<Connection>, String) — fallible
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "DB.open requires exactly 1 argument",
                        "Usage: let db, err = DB.open(path)",
                    )));
                }
                self.output.push_str("{ let __path = ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("; match rusqlite::Connection::open(&__path) { Ok(conn) => (Some(conn), String::new()), Err(e) => (None, format!(\"DB.open error: {}\", e)) } }");
            }
            "exec" => {
                // DB.exec(db, sql) or DB.exec(db, sql, params) → (Option<String>, String) — fallible
                if method_call.args.len() < 2 || method_call.args.len() > 3 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "DB.exec requires 2-3 arguments",
                        "Usage: DB.exec(db, sql) or DB.exec(db, sql, params)",
                    )));
                }
                self.output.push_str("{ let __sql = ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str("; ");
                if method_call.args.len() == 3 {
                    // With params — Bug #83 fix: use .to_string() per element to avoid moving variables
                    self.output.push_str("let __params: Vec<String> = ");
                    self.generate_db_params_vec(&method_call.args[2])?;
                    self.output.push_str("; let __param_refs: Vec<&dyn rusqlite::types::ToSql> = __params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect(); match ");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(".lock().unwrap().execute(&__sql, __param_refs.as_slice()) { Ok(_) => (Some(String::new()), String::new()), Err(e) => (None, format!(\"DB.exec error: {}\", e)) } }");
                } else {
                    // No params
                    self.output.push_str("match ");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(".lock().unwrap().execute_batch(&__sql) { Ok(_) => (Some(String::new()), String::new()), Err(e) => (None, format!(\"DB.exec error: {}\", e)) } }");
                }
            }
            "query" => {
                // DB.query(db, sql) or DB.query(db, sql, params) → (Option<Vec<HashMap>>, String) — fallible
                if method_call.args.len() < 2 || method_call.args.len() > 3 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "DB.query requires 2-3 arguments",
                        "Usage: DB.query(db, sql) or DB.query(db, sql, params)",
                    )));
                }
                self.output.push_str("{ let __sql = ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str("; ");

                if method_call.args.len() == 3 {
                    // With params — Bug #83 fix: use .to_string() per element to avoid moving variables
                    self.output.push_str("let __params: Vec<String> = ");
                    self.generate_db_params_vec(&method_call.args[2])?;
                    self.output.push_str("; let __param_refs: Vec<&dyn rusqlite::types::ToSql> = __params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect(); match ");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(".lock().unwrap().prepare(&__sql) { Ok(mut stmt) => { match stmt.query_map(__param_refs.as_slice(), |row| { let count = row.as_ref().column_count(); let mut map = std::collections::HashMap::<String, String>::new(); for i in 0..count { let col_name = row.as_ref().column_name(i).unwrap_or(\"\").to_string(); let val: String = row.get::<_, rusqlite::types::Value>(i).map(|v| match v { rusqlite::types::Value::Null => String::new(), rusqlite::types::Value::Integer(n) => n.to_string(), rusqlite::types::Value::Real(f) => f.to_string(), rusqlite::types::Value::Text(s) => s, rusqlite::types::Value::Blob(b) => format!(\"{:?}\", b), }).unwrap_or_default(); map.insert(col_name, val); } Ok(map) }) { Ok(rows) => { let result: Vec<std::collections::HashMap<String, String>> = rows.filter_map(|r| r.ok()).collect(); (Some(result), String::new()) }, Err(e) => (None, format!(\"DB.query error: {}\", e)) } }, Err(e) => (None, format!(\"DB.query error: {}\", e)) } }");
                } else {
                    // No params
                    self.output.push_str("match ");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(".lock().unwrap().prepare(&__sql) { Ok(mut stmt) => { match stmt.query_map([], |row| { let count = row.as_ref().column_count(); let mut map = std::collections::HashMap::<String, String>::new(); for i in 0..count { let col_name = row.as_ref().column_name(i).unwrap_or(\"\").to_string(); let val: String = row.get::<_, rusqlite::types::Value>(i).map(|v| match v { rusqlite::types::Value::Null => String::new(), rusqlite::types::Value::Integer(n) => n.to_string(), rusqlite::types::Value::Real(f) => f.to_string(), rusqlite::types::Value::Text(s) => s, rusqlite::types::Value::Blob(b) => format!(\"{:?}\", b), }).unwrap_or_default(); map.insert(col_name, val); } Ok(map) }) { Ok(rows) => { let result: Vec<std::collections::HashMap<String, String>> = rows.filter_map(|r| r.ok()).collect(); (Some(result), String::new()) }, Err(e) => (None, format!(\"DB.query error: {}\", e)) } }, Err(e) => (None, format!(\"DB.query error: {}\", e)) } }");
                }
            }
            "close" => {
                // DB.close(db) → drop the connection (no-op, Rust drops automatically)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "DB.close requires exactly 1 argument",
                        "Usage: DB.close(db)",
                    )));
                }
                self.output.push_str("drop(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push(')');
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown DB function: {}", method_call.method),
                    "Available: open(path), exec(db, sql[, params]), query(db, sql[, params]), close(db)",
                )));
            }
        }

        Ok(())
    }

    /// Bug #83 fix: Generate DB params vec with .to_string() per element instead of moving variables
    /// Generates: vec![a.to_string(), b.to_string()] instead of vec![a, b].iter().map(|s| s.to_string()).collect()
    fn generate_db_params_vec(&mut self, expr: &Expr) -> Result<()> {
        if let Expr::ArrayLiteral(elements) = expr {
            self.output.push_str("vec![");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.generate_expr(elem)?;
                self.output.push_str(".to_string()");
            }
            self.output.push(']');
        } else {
            // Fallback: if not an array literal, use the old approach
            self.generate_expr(expr)?;
            self.output.push_str(".iter().map(|s| s.to_string()).collect()");
        }
        Ok(())
    }

    fn generate_random_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "nextInt" => {
                // Random.nextInt(min, max) → i64
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Random.nextInt requires exactly 2 arguments",
                        "Usage: Random.nextInt(min, max)",
                    )));
                }
                self.output.push_str("{ use rand::Rng; rand::thread_rng().gen_range(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("..=");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") }");
            }
            "nextFloat" => {
                // Random.nextFloat(min, max) → f64, or Random.nextFloat() → 0.0..1.0
                if method_call.args.is_empty() {
                    self.output.push_str("rand::random::<f64>()");
                } else if method_call.args.len() == 2 {
                    self.output.push_str("{ use rand::Rng; let __min: f64 = ");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(" as f64; let __max: f64 = ");
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(" as f64; rand::thread_rng().gen_range(__min..__max) }");
                } else {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Random.nextFloat requires 0 or 2 arguments",
                        "Usage: Random.nextFloat() or Random.nextFloat(min, max)",
                    )));
                }
            }
            "choice" => {
                // Random.choice(array) → Option<T> element
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Random.choice requires exactly 1 argument",
                        "Usage: Random.choice(array)",
                    )));
                }
                self.output.push_str("{ use rand::Rng; let __arr = &");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("; if __arr.is_empty() { panic!(\"Random.choice: empty array\") } else { __arr[rand::thread_rng().gen_range(0..__arr.len())].clone() } }");
            }
            "shuffle" => {
                // Random.shuffle(array) → new shuffled array
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Random.shuffle requires exactly 1 argument",
                        "Usage: Random.shuffle(array)",
                    )));
                }
                self.output.push_str("{ use rand::seq::SliceRandom; let mut __v = ");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".clone(); __v.shuffle(&mut rand::thread_rng()); __v }");
            }
            "uuid" => {
                // Random.uuid() → String (UUID v4)
                if !method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Random.uuid takes no arguments",
                        "Usage: Random.uuid()",
                    )));
                }
                self.output.push_str("uuid::Uuid::new_v4().to_string()");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Random function: {}", method_call.method),
                    "Available: nextInt(min, max), nextFloat([min, max]), choice(arr), shuffle(arr), uuid()",
                )));
            }
        }

        Ok(())
    }

    fn generate_crypto_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "sha256" => {
                // Crypto.sha256(input) → String (hex)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Crypto.sha256 requires exactly 1 argument",
                        "Usage: Crypto.sha256(input)",
                    )));
                }
                self.output.push_str("{ use sha2::Digest; let mut hasher = sha2::Sha256::new(); hasher.update(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".as_bytes()); format!(\"{:x}\", hasher.finalize()) }");
            }
            "md5" => {
                // Crypto.md5(input) → String (hex)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Crypto.md5 requires exactly 1 argument",
                        "Usage: Crypto.md5(input)",
                    )));
                }
                self.output.push_str("{ use md5::Digest; let mut hasher = md5::Md5::new(); hasher.update(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".as_bytes()); format!(\"{:x}\", hasher.finalize()) }");
            }
            "base64Encode" => {
                // Crypto.base64Encode(input) → String
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Crypto.base64Encode requires exactly 1 argument",
                        "Usage: Crypto.base64Encode(input)",
                    )));
                }
                self.output.push_str("{ use base64::Engine; base64::engine::general_purpose::STANDARD.encode(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".as_bytes()) }");
            }
            "base64Decode" => {
                // Crypto.base64Decode(input) → (Option<String>, String) — fallible
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Crypto.base64Decode requires exactly 1 argument",
                        "Usage: Crypto.base64Decode(input)",
                    )));
                }
                self.output.push_str("{ use base64::Engine; match base64::engine::general_purpose::STANDARD.decode(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".as_bytes()) { Ok(bytes) => match String::from_utf8(bytes) { Ok(s) => (Some(s), String::new()), Err(e) => (None, format!(\"Crypto.base64Decode UTF-8 error: {}\", e)) }, Err(e) => (None, format!(\"Crypto.base64Decode error: {}\", e)) } }");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Crypto function: {}", method_call.method),
                    "Available: sha256(input), md5(input), base64Encode(input), base64Decode(input)",
                )));
            }
        }

        Ok(())
    }

    fn generate_process_function_call(
        &mut self,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        match method_call.method.as_str() {
            "exec" => {
                // Process.exec(cmd) → (Option<String>, String) — fallible
                // Runs command via shell and captures output
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Process.exec requires exactly 1 argument",
                        "Usage: Process.exec(command)",
                    )));
                }
                self.output.push_str("{ let __cmd = &");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("; match std::process::Command::new(\"sh\").arg(\"-c\").arg(__cmd).output() { Ok(output) => { if output.status.success() { (Some(String::from_utf8_lossy(&output.stdout).trim().to_string()), String::new()) } else { let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string(); (None, if stderr.is_empty() { format!(\"Process.exec failed with exit code: {}\", output.status) } else { stderr }) } }, Err(e) => (None, format!(\"Process.exec error: {}\", e)) } }");
            }
            "spawn" => {
                // Process.spawn(cmd) → (Option<i64>, String) — fallible, returns PID
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Process.spawn requires exactly 1 argument",
                        "Usage: Process.spawn(command)",
                    )));
                }
                self.output.push_str("{ let __cmd = &");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str("; match std::process::Command::new(\"sh\").arg(\"-c\").arg(__cmd).spawn() { Ok(child) => (Some(child.id() as i64), String::new()), Err(e) => (None, format!(\"Process.spawn error: {}\", e)) } }");
            }
            "pid" => {
                // Process.pid() → i64 (current process PID)
                if !method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Process.pid takes no arguments",
                        "Usage: Process.pid()",
                    )));
                }
                self.output.push_str("(std::process::id() as i64)");
            }
            "exit" => {
                // Process.exit(code) — terminates process
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                        "E3000",
                        "Process.exit requires exactly 1 argument",
                        "Usage: Process.exit(code)",
                    )));
                }
                self.output.push_str("std::process::exit(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(" as i32)");
            }
            _ => {
                return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E3000",
                    &format!("Unknown Process function: {}", method_call.method),
                    "Available: exec(cmd), spawn(cmd), pid(), exit(code)",
                )));
            }
        }

        Ok(())
    }

    /// Generate a function call through a module alias
    /// e.g., `mathlib.add(1, 2)` -> `math::add(1, 2)`
    fn generate_module_function_call(
        &mut self,
        module_name: &str,
        method_call: &crate::ast::MethodCallExpr,
    ) -> Result<()> {
        // Convert method name to snake_case for Rust
        let rust_method = to_snake_case(&method_call.method);

        // Generate module::function(args)
        self.output.push_str(module_name);
        self.output.push_str("::");
        self.output.push_str(&rust_method);
        self.output.push('(');

        // Generate arguments with proper type conversions
        for (i, arg) in method_call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            // Add .to_string() for string literals to convert &str to String
            if let Expr::Literal(Literal::String(_)) = arg {
                self.generate_expr(arg)?;
                self.output.push_str(".to_string()");
            } else if let Expr::Identifier(var_name) = arg {
                // Clone string variables when passing to module functions
                let sanitized = self.sanitize_name(var_name);
                if self.string_vars.contains(&sanitized) {
                    self.generate_expr(arg)?;
                    self.output.push_str(".clone()");
                } else {
                    self.generate_expr(arg)?;
                }
            } else {
                self.generate_expr(arg)?;
            }
        }

        self.output.push(')');

        Ok(())
    }

    fn generate_async_call(&mut self, call: &CallExpr) -> Result<()> {
        // Phase 2: NO await here - just create the Task
        // The await will be inserted at first use of the variable
        self.output.push_str("liva_rt::spawn_async(async move { ");

        // Check if callee is a MethodCall (e.g., HTTP.get())
        if let Expr::MethodCall(_) = &*call.callee {
            // MethodCall already generates the full call, just output it with .await
            self.generate_expr(&call.callee)?;
            self.output.push_str(".await");
        } else {
            // Regular function call
            self.generate_expr(&call.callee)?;
            self.output.push('(');
            for (i, arg) in call.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                // Convert string literals to String automatically
                if let Expr::Literal(Literal::String(_)) = arg {
                    self.generate_expr(arg)?;
                    self.output.push_str(".to_string()");
                } else {
                    self.generate_expr(arg)?;
                }
            }
            self.output.push(')');
            // B04 fix: add .await for user-defined async functions inside spawn_async
            // Without .await, the function returns a Future<T> instead of T
            if let Expr::Identifier(name) = &*call.callee {
                if self.async_functions.contains(name) {
                    self.output.push_str(".await");
                }
            }
        }

        self.output.push_str(" })");
        // Note: NO .await.unwrap() here anymore!
        Ok(())
    }

    fn generate_parallel_call(&mut self, call: &CallExpr) -> Result<()> {
        // Phase 2: NO await here - just create the Task
        // The await will be inserted at first use of the variable
        self.output.push_str("liva_rt::spawn_parallel(move || ");
        self.generate_expr(&call.callee)?;
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            // Convert string literals to String automatically
            if let Expr::Literal(Literal::String(_)) = arg {
                self.generate_expr(arg)?;
                self.output.push_str(".to_string()");
            } else {
                self.generate_expr(arg)?;
            }
        }
        self.output.push(')');
        self.output.push(')');
        // Note: NO .await.unwrap() here anymore!
        Ok(())
    }

    fn generate_task_call(&mut self, call: &CallExpr, mode: ConcurrencyMode) -> Result<()> {
        let callee_name = match call.callee.as_ref() {
            Expr::Identifier(name) => name.clone(),
            _ => {
                return Err(CompilerError::CodegenError(
                    "Task calls currently only support simple function names".into(),
                ));
            }
        };

        let rust_name = self.sanitize_name(&callee_name);

        match mode {
            ConcurrencyMode::Async => {
                self.output.push_str("liva_rt::spawn_async(async move { ");
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert string literals to String automatically
                    if let Expr::Literal(Literal::String(_)) = arg {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push_str(") })");
            }
            ConcurrencyMode::Parallel => {
                self.output.push_str("liva_rt::spawn_parallel(move || { ");
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert string literals to String automatically
                    if let Expr::Literal(Literal::String(_)) = arg {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push_str(") })");
            }
        }
        Ok(())
    }

    fn generate_fire_call(&mut self, call: &CallExpr, mode: ConcurrencyMode) -> Result<()> {
        let callee_name = match call.callee.as_ref() {
            Expr::Identifier(name) => name.clone(),
            _ => {
                return Err(CompilerError::CodegenError(
                    "Fire calls currently only support simple function names".into(),
                ));
            }
        };

        let rust_name = self.sanitize_name(&callee_name);

        match mode {
            ConcurrencyMode::Async => {
                self.output.push_str("liva_rt::fire_async(async move {");
                self.output.push(' ');
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert string literals to String automatically
                    if let Expr::Literal(Literal::String(_)) = arg {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push_str("); })");
            }
            ConcurrencyMode::Parallel => {
                self.output.push_str("liva_rt::fire_parallel(move || {");
                self.output.push(' ');
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert string literals to String automatically
                    if let Expr::Literal(Literal::String(_)) = arg {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push_str("); })");
            }
        }
        Ok(())
    }

    fn generate_condition_expr(&mut self, expr: &Expr) -> Result<()> {
        // Special handling for error variables in conditions
        // if error_var -> error_var.is_some()
        // if !error_var -> error_var.is_none() (handled in Unary)
        if let Expr::Identifier(name) = expr {
            let sanitized = self.sanitize_name(name);
            if self.error_binding_vars.contains(&sanitized) {
                write!(self.output, "{}.is_some()", sanitized).unwrap();
                return Ok(());
            }
            // String error vars (from HTTP/File): if err -> !err.is_empty()
            if self.string_error_vars.contains(&sanitized) {
                write!(self.output, "!{}.is_empty()", sanitized).unwrap();
                return Ok(());
            }
        }

        // Otherwise, generate normally
        self.generate_expr(expr)
    }

    fn generate_binary_operation(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Result<()> {
        // Optional chaining or default: expr?.field or default → expr.as_ref().map(...).unwrap_or(default)
        // Also handles option_value_vars: optVar or default → optVar.unwrap_or(default)
        if matches!(op, BinOp::Or) && (matches!(left, Expr::OptionalChain { .. }) || matches!(left, Expr::Identifier(name) if self.option_value_vars.contains(&self.sanitize_name(name)))) {
            self.generate_expr(left)?;
            self.output.push_str(".unwrap_or(");
            if matches!(right, Expr::Literal(Literal::String(_))) {
                self.generate_expr(right)?;
                self.output.push_str(".to_string()");
            } else {
                self.generate_expr(right)?;
            }
            self.output.push(')');
            return Ok(());
        }

        // Bug #76 fix: map.get(key) or default → map.get(&key).cloned().unwrap_or(default)
        // When `or` is used with a Map.get expression, it's not boolean OR but Option unwrap
        if matches!(op, BinOp::Or) && self.is_map_get_call(left) {
            self.suppress_map_get_unwrap = true;
            self.generate_expr(left)?;
            self.suppress_map_get_unwrap = false;
            self.output.push_str(".unwrap_or(");
            if matches!(right, Expr::Literal(Literal::String(_))) {
                self.generate_expr(right)?;
                self.output.push_str(".to_string()");
            } else {
                self.generate_expr(right)?;
            }
            self.output.push(')');
            return Ok(());
        }

        // Phase 3: Special handling for error binding variable comparisons with ""
        // Transform: err != "" to err.is_some()
        // Transform: err == "" to err.is_none()
        if matches!(op, BinOp::Ne | BinOp::Eq) {
            let is_error_var_comparison = match (left, right) {
                (Expr::Identifier(name), Expr::Literal(Literal::String(s))) if s.is_empty() => {
                    let sanitized = self.sanitize_name(name);
                    self.error_binding_vars.contains(&sanitized)
                }
                (Expr::Literal(Literal::String(s)), Expr::Identifier(name)) if s.is_empty() => {
                    let sanitized = self.sanitize_name(name);
                    self.error_binding_vars.contains(&sanitized)
                }
                _ => false,
            };

            if is_error_var_comparison {
                // Generate err.is_some() or err.is_none()
                if let Expr::Identifier(name) = left {
                    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                } else if let Expr::Identifier(name) = right {
                    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                }

                if matches!(op, BinOp::Ne) {
                    self.output.push_str(".is_some()");
                } else {
                    self.output.push_str(".is_none()");
                }
                return Ok(());
            }

            // Special handling for Option variable comparison with null
            // Transform: x != null -> x.is_some()
            // Transform: x == null -> x.is_none()
            let is_option_null_comparison = match (left, right) {
                (Expr::Identifier(name), Expr::Literal(Literal::Null)) => {
                    let sanitized = self.sanitize_name(name);
                    self.option_value_vars.contains(&sanitized)
                }
                (Expr::Literal(Literal::Null), Expr::Identifier(name)) => {
                    let sanitized = self.sanitize_name(name);
                    self.option_value_vars.contains(&sanitized)
                }
                _ => false,
            };

            if is_option_null_comparison {
                // Generate x.is_some() or x.is_none()
                let var_name = match (left, right) {
                    (Expr::Identifier(name), _) => name,
                    (_, Expr::Identifier(name)) => name,
                    _ => unreachable!(),
                };
                write!(self.output, "{}", self.sanitize_name(var_name)).unwrap();

                if matches!(op, BinOp::Ne) {
                    self.output.push_str(".is_some()");
                } else {
                    self.output.push_str(".is_none()");
                }
                return Ok(());
            }

            // Special handling for JsonValue comparison with null
            // Transform: jsonVar != null -> !jsonVar.is_null()
            // Transform: jsonVar == null -> jsonVar.is_null()
            let is_json_null_comparison = match (left, right) {
                (Expr::Identifier(name), Expr::Literal(Literal::Null)) => {
                    let sanitized = self.sanitize_name(name);
                    self.json_value_vars.contains(&sanitized)
                }
                (Expr::Literal(Literal::Null), Expr::Identifier(name)) => {
                    let sanitized = self.sanitize_name(name);
                    self.json_value_vars.contains(&sanitized)
                }
                _ => false,
            };

            if is_json_null_comparison {
                // Generate !x.is_null() or x.is_null()
                let var_name = match (left, right) {
                    (Expr::Identifier(name), _) => name,
                    (_, Expr::Identifier(name)) => name,
                    _ => unreachable!(),
                };

                if matches!(op, BinOp::Ne) {
                    self.output.push_str("!");
                }
                write!(self.output, "{}.is_null()", self.sanitize_name(var_name)).unwrap();
                return Ok(());
            }
        }

        // Special handling for ref_lambda_params: when comparing &T with T,
        // dereference the lambda param: *item == query (or *item > value, etc.)
        if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge) && !self.ref_lambda_params.is_empty() {
            let left_is_ref = if let Expr::Identifier(name) = left {
                self.ref_lambda_params.contains(name)
            } else {
                false
            };
            let right_is_ref = if let Expr::Identifier(name) = right {
                self.ref_lambda_params.contains(name)
            } else {
                false
            };

            if left_is_ref || right_is_ref {
                if left_is_ref {
                    self.output.push('*');
                }
                self.generate_expr(left)?;
                write!(self.output, " {} ", op).unwrap();
                if right_is_ref {
                    self.output.push('*');
                }
                self.generate_expr(right)?;
                return Ok(());
            }
        }

        // Special handling for string multiplication (String * int or int * String)
        if matches!(op, BinOp::Mul) {
            let has_string_literal = matches!(left, Expr::Literal(Literal::String(_)))
                || matches!(right, Expr::Literal(Literal::String(_)));

            if has_string_literal {
                // Use string_mul helper only when we have a string literal
                self.output.push_str("liva_rt::string_mul(");
                self.generate_expr(left)?;
                self.output.push_str(", ");
                self.generate_expr(right)?;
                self.output.push(')');
                return Ok(());
            }
        }

        // B40 fix: String ordering comparisons (>, <, >=, <=) need both sides as &str
        // because PartialOrd<&str> is NOT implemented for String (unlike PartialEq)
        if matches!(op, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) {
            let left_is_string = self.expr_is_stringy(left)
                || matches!(left, Expr::Literal(Literal::String(_)))
                || matches!(left, Expr::Identifier(name) if self.string_vars.contains(name));
            let right_is_string = self.expr_is_stringy(right)
                || matches!(right, Expr::Literal(Literal::String(_)))
                || matches!(right, Expr::Identifier(name) if self.string_vars.contains(name));

            if left_is_string || right_is_string {
                // Generate: left.as_str() >= "literal" or left.as_str() >= right.as_str()
                let left_needs_as_str = matches!(left, Expr::Identifier(_));
                let right_needs_as_str = matches!(right, Expr::Identifier(_));

                self.generate_expr(left)?;
                if left_needs_as_str {
                    self.output.push_str(".as_str()");
                }
                write!(self.output, " {} ", op).unwrap();
                self.generate_expr(right)?;
                if right_needs_as_str {
                    self.output.push_str(".as_str()");
                }
                return Ok(());
            }
        }

        // B32 fix: Mixed float/int arithmetic — auto-cast .length to f64 when other side is float
        // Rust doesn't allow f64 / i32 — both sides must be same type
        if matches!(op, BinOp::Div | BinOp::Mul | BinOp::Add | BinOp::Sub) {
            let left_is_float = match left {
                Expr::Literal(Literal::Float(_)) => true,
                Expr::Identifier(name) => self.float_vars.contains(name),
                _ => false,
            };
            let right_is_float = match right {
                Expr::Literal(Literal::Float(_)) => true,
                Expr::Identifier(name) => self.float_vars.contains(name),
                _ => false,
            };
            let left_is_length = matches!(left, Expr::Member { property, .. } if property == "length");
            let right_is_length = matches!(right, Expr::Member { property, .. } if property == "length");

            if (left_is_float && right_is_length) || (right_is_float && left_is_length) {
                // Generate with f64 cast on the .length side
                if left_is_length {
                    self.output.push('(');
                    self.generate_expr(left)?;
                    self.output.push_str(" as f64)");
                } else {
                    self.generate_expr(left)?;
                }
                write!(self.output, " {} ", op).unwrap();
                if right_is_length {
                    self.output.push('(');
                    self.generate_expr(right)?;
                    self.output.push_str(" as f64)");
                } else {
                    self.generate_expr(right)?;
                }
                return Ok(());
            }
        }

        // Original logic for other binary operations
        // Only add parentheses when necessary for precedence
        let left_needs_parens = self.expr_needs_parens_for_binop(left, op);
        let right_needs_parens = self.expr_needs_parens_for_binop(right, op);

        if left_needs_parens {
            self.output.push('(');
        }
        self.generate_expr(left)?;
        if left_needs_parens {
            self.output.push(')');
        }

        write!(self.output, " {} ", op).unwrap();

        if right_needs_parens {
            self.output.push('(');
        }
        self.generate_expr(right)?;
        if right_needs_parens {
            self.output.push(')');
        }

        Ok(())
    }

    fn expr_needs_parens_for_binop(&self, expr: &Expr, parent_op: &BinOp) -> bool {
        match expr {
            Expr::Literal(_) | Expr::Identifier(_) => false,
            // Member access, calls, method calls, and indexing bind tighter than any binop
            Expr::Member { .. }
            | Expr::Call(_)
            | Expr::MethodCall(_)
            | Expr::Index { .. } => false,
            Expr::Binary { op, .. } => {
                // Parentheses needed if this expression has lower precedence than parent
                self.binop_precedence(op) < self.binop_precedence(parent_op)
            }
            _ => true, // Default to needing parentheses for complex expressions
        }
    }

    fn binop_precedence(&self, op: &BinOp) -> i32 {
        match op {
            BinOp::Mul | BinOp::Div | BinOp::Mod => 100,
            BinOp::Add | BinOp::Sub => 90,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 80,
            BinOp::Eq | BinOp::Ne => 70,
            BinOp::And => 60,
            BinOp::Or => 50,
            BinOp::Range | BinOp::RangeInclusive => 40,
        }
    }

    /// FIX-4: Check if a variable is known to be a Copy type (doesn't need cloning)
    fn is_copy_var(&self, name: &str) -> bool {
        // If we know the variable's Liva type, check directly
        if let Some(type_name) = self.var_types.get(name) {
            if matches!(type_name.as_str(), "int" | "float" | "number" | "bool" | "f32" | "f64") {
                return true;
            }
            // Check if it's a unit enum (Copy from FIX-5)
            if let Some(variants) = self.enum_variants.get(type_name.as_str()) {
                if variants.values().all(|fields| fields.is_empty()) {
                    return true;
                }
            }
        }
        // Known integer/bool literals get inferred as Copy
        false
    }

    /// FIX-4: Heuristic to detect likely non-Copy variables not tracked in any HashSet.
    /// Returns true only if the variable is a known non-Copy type from var_types
    /// or enum_variants that wasn't caught by the primary HashSet checks.
    fn looks_like_non_copy_var(&self, name: &str) -> bool {
        // Check if it's a known enum variable (from enum construction tracking)
        if let Some(type_name) = self.var_types.get(name) {
            // Check if it's an enum with data (non-Copy)
            if let Some(variants) = self.enum_variants.get(type_name.as_str()) {
                return !variants.values().all(|fields| fields.is_empty());
            }
            // If it's in var_types and not a known Copy type, it's likely non-Copy
            return !matches!(type_name.as_str(),
                "int" | "float" | "number" | "bool" | "f32" | "f64" | "i32" | "i64"
            );
        }
        false
    }

    fn block_has_return(&self, block: &BlockStmt) -> bool {
        block
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Return(_)))
    }

    /// Infer return type from the first return statement in a block
    fn infer_return_type_from_block(&self, block: &BlockStmt) -> Option<String> {
        for stmt in &block.stmts {
            if let Stmt::Return(return_stmt) = stmt {
                if let Some(expr) = &return_stmt.expr {
                    return self.infer_expr_type(expr, None);
                }
            }
        }
        None
    }

    fn block_ends_with_return(&self, block: &BlockStmt) -> bool {
        block
            .stmts
            .last()
            .map(|stmt| matches!(stmt, Stmt::Return(_) | Stmt::Fail(_)))
            .unwrap_or(false)
    }

    fn is_map_get_call(&self, expr: &Expr) -> bool {
        if let Expr::MethodCall(mc) = expr {
            if mc.method == "get" {
                if let Expr::Identifier(obj_name) = &*mc.object {
                    let sanitized = self.sanitize_name(obj_name);
                    return self.map_vars.contains(&sanitized);
                }
                // Bug #76 fix: Also check this._field for Map-typed class fields
                if let Expr::Member { object, property } = &*mc.object {
                    if matches!(object.as_ref(), Expr::Identifier(name) if name == "this" || name == "self") {
                        return self.map_vars.contains(&self.sanitize_name(property));
                    }
                }
            }
        }
        false
    }

    /// Check if expression is a method call that returns Option<T> (find, first, last, min, max)
    /// BUG-007: Detect `x != null` where x is an Option variable.
    /// Returns the sanitized variable name if this is a null-check on an Option var.
    fn extract_option_null_check(&self, condition: &Expr) -> Option<String> {
        if let Expr::Binary { op: BinOp::Ne, left, right } = condition {
            // x != null → always generate if let Some(x) = x { ... }
            // Any variable compared to null is Optional by definition
            if let (Expr::Identifier(name), Expr::Literal(Literal::Null)) = (left.as_ref(), right.as_ref()) {
                let sanitized = self.sanitize_name(name);
                return Some(sanitized);
            }
            // null != x
            if let (Expr::Literal(Literal::Null), Expr::Identifier(name)) = (left.as_ref(), right.as_ref()) {
                let sanitized = self.sanitize_name(name);
                return Some(sanitized);
            }
        }
        None
    }

    fn is_option_returning_method(&self, expr: &Expr) -> bool {
        if let Expr::MethodCall(mc) = expr {
            matches!(mc.method.as_str(), "find" | "first" | "last" | "min" | "max")
        } else {
            false
        }
    }

    /// FIX-1: Check if an init expression already produces an Option<T> value,
    /// so we should NOT double-wrap it in Some().
    fn init_is_already_optional(&self, expr: &Expr) -> bool {
        match expr {
            // Variable already tracked as optional
            Expr::Identifier(name) => {
                let san = self.sanitize_name(name);
                self.option_value_vars.contains(&san)
            }
            // Member access on a class instance: check if the field is Optional in the class definition
            Expr::Member { object, property } => {
                // Get the variable type from var_types
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let san = self.sanitize_name(var_name);
                    if let Some(type_name) = self.var_types.get(&san) {
                        // Look up the class in class_optional_fields
                        let sanitized_prop = self.sanitize_name(property);
                        if let Some(optionals) = self.class_optional_fields.get(type_name) {
                            return optionals.contains(&sanitized_prop);
                        }
                    }
                }
                false
            }
            // Function call to an optional-returning function
            Expr::Call(call) => {
                if let Expr::Identifier(fn_name) = &*call.callee {
                    self.optional_returning_functions.contains(fn_name)
                } else {
                    false
                }
            }
            // Method call that returns Option<T>
            Expr::MethodCall(mc) => {
                matches!(mc.method.as_str(), "find" | "first" | "last" | "min" | "max")
            }
            // Optional chaining already produces Option<T>
            Expr::OptionalChain { .. } => true,
            _ => false,
        }
    }

    fn is_fallible_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if calling a fallible function
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    self.fallible_functions.contains(name)
                } else {
                    false
                }
            }
            // B19 fix: Method calls can also be fallible
            Expr::MethodCall(mc) => {
                self.fallible_methods.contains(&mc.method)
            }
            Expr::Ternary {
                condition: _,
                then_expr,
                else_expr,
            } => {
                // A ternary is fallible if either branch contains a fail or calls a fallible function
                self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr)
            }
            _ => false,
        }
    }

    /// Check if expression is a built-in conversion function call that returns (value, Option<Error>)
    fn is_builtin_conversion_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if callee is an identifier (parseInt, parseFloat)
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    return name == "parseInt" || name == "parseFloat";
                }
                // Check if callee is a MethodCall (async HTTP.get, etc.)
                // This handles the async/par wrapper around method calls
                if let Expr::MethodCall(_) = call.callee.as_ref() {
                    return self.is_builtin_conversion_call(call.callee.as_ref());
                }
                false
            }
            Expr::MethodCall(method_call) => {
                // Check for .json() method on HTTP responses
                if method_call.method == "json" {
                    // response.json() returns (Option<JsonValue>, String)
                    return true;
                }

                if let Expr::Identifier(object_name) = method_call.object.as_ref() {
                    // Check for JSON methods
                    if object_name == "JSON"
                        && (method_call.method == "parse" || method_call.method == "stringify")
                    {
                        return true;
                    }
                    // Check for File methods (all except exists return tuples)
                    if object_name == "File"
                        && (method_call.method == "read"
                            || method_call.method == "write"
                            || method_call.method == "append"
                            || method_call.method == "delete")
                    {
                        return true;
                    }
                    // Check for Dir methods (list returns tuple)
                    if object_name == "Dir" && method_call.method == "list" {
                        return true;
                    }
                    // Check for HTTP methods (all return tuples)
                    if (object_name == "HTTP" || object_name == "Http")
                        && (method_call.method == "get"
                            || method_call.method == "post"
                            || method_call.method == "put"
                            || method_call.method == "delete")
                    {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn expr_contains_fail(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Fail(_) => true,
            Expr::Call(call) => self.is_fallible_expr(&Expr::Call(call.clone())),
            Expr::Ternary {
                then_expr,
                else_expr,
                ..
            } => self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr),
            _ => false,
        }
    }

    /// Check if a binary Add expression involves arrays (for array concatenation)
    fn expr_is_array(&self, left: &Expr, right: &Expr) -> bool {
        let is_array_expr = |expr: &Expr| -> bool {
            match expr {
                Expr::ArrayLiteral(_) => true,
                Expr::Identifier(name) => {
                    let sanitized = self.sanitize_name(name);
                    self.array_vars.contains(&sanitized)
                        || self.typed_array_vars.contains_key(&sanitized)
                }
                _ => false,
            }
        };
        is_array_expr(left) || is_array_expr(right)
    }

    fn expr_is_stringy(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(Literal::String(_)) => true,
            Expr::StringTemplate { .. } => true,
            Expr::Binary {
                op: BinOp::Add,
                left,
                right,
            } => self.expr_is_stringy(left) || self.expr_is_stringy(right),
            // Bug #18 fix: Variables known to be strings should trigger format! usage
            Expr::Identifier(name) => {
                let sanitized = self.sanitize_name(name);
                self.string_vars.contains(&sanitized)
            }
            // Detect string-returning method calls: .toString(), .toUpperCase(), .toLowerCase(), etc.
            Expr::MethodCall(mc) => {
                matches!(
                    mc.method.as_str(),
                    "toString"
                        | "toUpperCase"
                        | "toLowerCase"
                        | "trim"
                        | "trimStart"
                        | "trimEnd"
                        | "replace"
                        | "substring"
                        | "join"
                )
            }
            // Detect string-returning function calls like toString(x) or user-defined string functions
            Expr::Call(call) => {
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    name == "toString" || self.string_returning_functions.contains(name)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Generate an expression with special handling for error binding variables in string context
    fn generate_expr_for_string_concat(&mut self, expr: &Expr) -> Result<()> {
        // Handle map.get() calls in string concatenation → unwrap Option<String> to String
        if let Expr::MethodCall(mc) = expr {
            if mc.method == "get" {
                let is_map = if let Expr::Identifier(name) = mc.object.as_ref() {
                    self.map_vars.contains(&self.sanitize_name(name))
                } else {
                    false
                };
                if is_map {
                    self.generate_map_method_call(mc)?;
                    self.output.push_str(".unwrap_or_default()");
                    return Ok(());
                }
            }
        }
        // Check if this is an error binding variable
        if let Expr::Identifier(name) = expr {
            let sanitized = self.sanitize_name(name);
            if self.error_binding_vars.contains(&sanitized) {
                // For error variables, extract the message: err.as_ref().map(|e| e.message.as_str()).unwrap_or("")
                write!(
                    self.output,
                    "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"\")",
                    sanitized
                )
                .unwrap();
                return Ok(());
            }
            if self.option_value_vars.contains(&sanitized) {
                // For option value variables, unwrap with default: value.as_ref().map(|v| v.to_string()).unwrap_or_default()
                write!(
                    self.output,
                    "{}.as_ref().map(|v| v.to_string()).unwrap_or_default()",
                    sanitized
                )
                .unwrap();
                return Ok(());
            }
            if self.struct_destructured_vars.contains(&sanitized) {
                // For struct destructured variables (may be Option<T>), use as_ref().map().unwrap_or_default()
                write!(
                    self.output,
                    "{}.as_ref().map(|v| format!(\"{{}}\", v)).unwrap_or_default()",
                    sanitized
                )
                .unwrap();
                return Ok(());
            }
        }
        // Otherwise, generate normally
        self.generate_expr(expr)
    }

    fn generate_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => {
                // Use context-aware suffix (f64 by default, f32 when in f32-typed context)
                write!(self.output, "{}_{}", f, self.float_literal_suffix).unwrap();
            }
            Literal::String(s) => {
                // Write string with proper escape sequences interpreted
                // Don't use escape_default() as it would escape the escapes (\\n instead of \n)
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            Literal::Char(c) => write!(self.output, "'{}'", c.escape_default()).unwrap(),
            Literal::Bool(b) => write!(self.output, "{}", b).unwrap(),
            Literal::Null => self.output.push_str("None"),
        }
        Ok(())
    }

    fn infer_const_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(Literal::Int(_)) => "i32".to_string(),
            Expr::Literal(Literal::Float(_)) => "f64".to_string(),
            Expr::Literal(Literal::String(_)) => "&str".to_string(),
            Expr::Literal(Literal::Bool(_)) => "bool".to_string(),
            Expr::Literal(Literal::Char(_)) => "char".to_string(),
            Expr::Literal(Literal::Null) => "Option<()>".to_string(),
            _ => "i32".to_string(),
        }
    }

    /// Generate destructuring code for a binding pattern
    fn generate_destructuring_pattern(
        &mut self,
        pattern: &BindingPattern,
        init_expr: &Expr,
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(name) => {
                // Simple binding - should not reach here since we check is_simple() before calling
                // But handle it anyway for completeness
                let var_name = self.sanitize_name(name);

                // Track variable types based on init expression
                if let Expr::ObjectLiteral(_) = init_expr {
                    self.bracket_notation_vars.insert(name.clone());
                }
                if let Expr::ArrayLiteral(_) = init_expr {
                    self.array_vars.insert(name.clone());
                }
                if let Expr::Call(call) = init_expr {
                    if let Expr::Identifier(class_name) = &*call.callee {
                        self.class_instance_vars.insert(name.clone());
                        self.var_types.insert(name.clone(), class_name.clone());
                    }
                }

                write!(self.output, "let mut {}", var_name).unwrap();
                self.output.push_str(" = ");
                self.generate_expr(init_expr)?;
                self.output.push_str(";\n");
            }
            BindingPattern::Object(obj_pattern) => {
                // Object destructuring: let {x, y} = point
                // First, generate temporary variable for the object
                let temp_var = format!("_temp_{}", self.gen_unique_id());
                write!(self.output, "let {} = ", temp_var).unwrap();
                self.generate_expr(init_expr)?;
                self.output.push_str(";\n");

                // Then extract each field
                for field in &obj_pattern.fields {
                    self.write_indent();
                    let binding_name = self.sanitize_name(&field.binding);

                    // Check if temp var is a JsonValue or needs bracket notation
                    // We need to check the init_expr to determine the type
                    let needs_bracket_notation = matches!(init_expr, Expr::ObjectLiteral(_))
                        || (matches!(init_expr, Expr::Identifier(id) if
                            self.bracket_notation_vars.contains(id) ||
                            self.json_value_vars.contains(id)));

                    if needs_bracket_notation {
                        // JSON object access using bracket notation
                        write!(
                            self.output,
                            "let mut {} = {}[\"{}\"].clone();\n",
                            binding_name, temp_var, field.key
                        )
                        .unwrap();
                    } else {
                        // Struct field access - clone to handle non-Copy types (String, Vec, etc.)
                        write!(
                            self.output,
                            "let mut {} = {}.{}.clone();\n",
                            binding_name, temp_var, field.key
                        )
                        .unwrap();
                    }
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Array destructuring: let [first, second] = array
                // First, generate temporary variable for the array (clone to avoid move)
                let temp_var = format!("_temp_{}", self.gen_unique_id());
                write!(self.output, "let {} = ", temp_var).unwrap();
                self.generate_expr(init_expr)?;
                self.output.push_str(".clone();\n");

                // Extract individual elements
                for (i, element) in arr_pattern.elements.iter().enumerate() {
                    if let Some(name) = element {
                        self.write_indent();
                        let binding_name = self.sanitize_name(name);
                        write!(
                            self.output,
                            "let mut {} = {}[{}].clone();\n",
                            binding_name, temp_var, i
                        )
                        .unwrap();
                    }
                }

                // Handle rest pattern: [...rest]
                if let Some(rest_name) = &arr_pattern.rest {
                    self.write_indent();
                    let binding_name = self.sanitize_name(rest_name);
                    let start_index = arr_pattern.elements.len();
                    write!(
                        self.output,
                        "let mut {}: Vec<_> = {}[{}..].to_vec();\n",
                        binding_name, temp_var, start_index
                    )
                    .unwrap();

                    // Track the rest variable as an array
                    self.array_vars.insert(rest_name.clone());
                }
            }
            BindingPattern::Tuple(tuple_pattern) => {
                // Tuple destructuring: let (x, y, z) = tuple
                // Generate tuple pattern on left side
                self.output.push_str("let (");
                for (i, name) in tuple_pattern.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let var_name = self.sanitize_name(name);
                    write!(self.output, "mut {}", var_name).unwrap();
                }
                self.output.push_str(") = ");
                self.generate_expr(init_expr)?;
                self.output.push_str(";\n");
            }
        }

        Ok(())
    }

    /// Generate destructuring code for function parameters at the start of function body
    /// For each destructured parameter, generate let statements to extract values
    fn generate_param_destructuring(&mut self, params: &[Param]) -> Result<()> {
        for (param_idx, param) in params.iter().enumerate() {
            if param.is_destructuring() {
                // Parameter is destructured, need to generate let statements
                // The parameter in the function signature uses a temp name: _param_0, _param_1, etc.
                let temp_name = format!("_param_{}", param_idx);

                match &param.pattern {
                    BindingPattern::Object(obj_pattern) => {
                        // Object destructuring: {x, y} => extract each field
                        for field in &obj_pattern.fields {
                            self.write_indent();
                            let binding_name = self.sanitize_name(&field.binding);

                            // For parameters, we always use struct field access (not JSON)
                            // Clone to handle non-Copy types
                            write!(
                                self.output,
                                "let mut {} = {}.{}.clone();\n",
                                binding_name, temp_name, field.key
                            )
                            .unwrap();
                        }
                    }
                    BindingPattern::Array(arr_pattern) => {
                        // Array destructuring: [first, second] => extract by index
                        for (i, element) in arr_pattern.elements.iter().enumerate() {
                            if let Some(name) = element {
                                self.write_indent();
                                let binding_name = self.sanitize_name(name);
                                write!(
                                    self.output,
                                    "let mut {} = {}[{}].clone();\n",
                                    binding_name, temp_name, i
                                )
                                .unwrap();
                            }
                        }

                        // Handle rest pattern: [...rest]
                        if let Some(rest_name) = &arr_pattern.rest {
                            self.write_indent();
                            let binding_name = self.sanitize_name(rest_name);
                            let start_index = arr_pattern.elements.len();
                            write!(
                                self.output,
                                "let mut {}: Vec<_> = {}[{}..].to_vec();\n",
                                binding_name, temp_name, start_index
                            )
                            .unwrap();

                            // Track as array variable
                            self.array_vars.insert(rest_name.clone());
                        }
                    }
                    BindingPattern::Tuple(tuple_pattern) => {
                        // Tuple destructuring: (x, y, z) => extract by position
                        self.write_indent();
                        self.output.push_str("let (");
                        for (i, name) in tuple_pattern.elements.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            let var_name = self.sanitize_name(name);
                            write!(self.output, "mut {}", var_name).unwrap();
                        }
                        write!(self.output, ") = {};\n", temp_name).unwrap();
                    }
                    BindingPattern::Identifier(_) => {
                        // Not destructured, nothing to do
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate destructuring code for lambda parameters
    /// Similar to generate_param_destructuring but for lambdas (no indent on first line)
    fn generate_lambda_param_destructuring(
        &mut self,
        pattern: &BindingPattern,
        temp_name: &str,
        is_json_value: bool,
        class_name: Option<&str>,
    ) -> Result<()> {
        match pattern {
            BindingPattern::Object(obj_pattern) => {
                // Object destructuring: extract each field
                for field in &obj_pattern.fields {
                    let binding_name = self.sanitize_name(&field.binding);

                    if is_json_value {
                        // For JsonValue, use .get("field") access
                        write!(
                            self.output,
                            "let {} = {}[\"{}\"].clone();\n",
                            binding_name, temp_name, field.key
                        )
                        .unwrap();
                    } else {
                        // For structs, check if field is optional in the class definition
                        let is_field_optional = if let Some(cls_name) = class_name {
                            if let Some(optional_fields) = self.class_optional_fields.get(cls_name)
                            {
                                optional_fields.contains(&field.key)
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_field_optional {
                            // For optional fields, unwrap or use default
                            write!(
                                self.output,
                                "let {} = {}.{}.as_ref().map(|v| v.clone()).unwrap_or_default();\n",
                                binding_name, temp_name, field.key
                            )
                            .unwrap();
                            // Only register optional fields for special string template handling
                            self.struct_destructured_vars.insert(binding_name.clone());
                        } else {
                            // For required fields, just clone
                            write!(
                                self.output,
                                "let {} = {}.{}.clone();\n",
                                binding_name, temp_name, field.key
                            )
                            .unwrap();
                        }

                        // Check if this field is itself a class type, and register it as a class instance
                        // This is important for nested struct access (e.g., address.zipcode)
                        if let Some(_cls_name) = class_name {
                            // Try to get the field type from class_fields metadata
                            // For now, we'll use a heuristic: if the field name starts with lowercase
                            // and there's a corresponding capitalized class, mark it as class instance
                            let potential_class = capitalize_first_letter(&field.key);
                            if self.class_fields.contains_key(&potential_class) {
                                self.class_instance_vars.insert(binding_name.clone());
                                self.var_types.insert(binding_name.clone(), potential_class);
                            }
                        }
                    }

                    if field != obj_pattern.fields.last().unwrap() {
                        self.write_indent();
                    }
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Array destructuring: extract by index
                for (i, element) in arr_pattern.elements.iter().enumerate() {
                    if let Some(name) = element {
                        let binding_name = self.sanitize_name(name);
                        write!(
                            self.output,
                            "let {} = {}[{}].clone();\n",
                            binding_name, temp_name, i
                        )
                        .unwrap();
                        if i < arr_pattern.elements.len() - 1 || arr_pattern.rest.is_some() {
                            self.write_indent();
                        }
                    }
                }

                // Handle rest pattern
                if let Some(rest_name) = &arr_pattern.rest {
                    let binding_name = self.sanitize_name(rest_name);
                    let start_index = arr_pattern.elements.len();
                    write!(
                        self.output,
                        "let {}: Vec<_> = {}[{}..].to_vec();\n",
                        binding_name, temp_name, start_index
                    )
                    .unwrap();
                }
            }
            BindingPattern::Tuple(tuple_pattern) => {
                // Tuple destructuring: extract by position
                self.output.push_str("let (");
                for (i, name) in tuple_pattern.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let var_name = self.sanitize_name(name);
                    write!(self.output, "{}", var_name).unwrap();
                }
                write!(self.output, ") = {};\n", temp_name).unwrap();
            }
            BindingPattern::Identifier(_) => {
                // Not destructured, nothing to do
            }
        }

        Ok(())
    }

    /// Generate a unique ID for temporary variables
    fn gen_unique_id(&self) -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn sanitize_name(&self, name: &str) -> String {
        // Convert to snake_case, preserving leading underscore for private fields
        let has_leading_underscore = name.starts_with('_');
        let name_without_prefix = name.trim_start_matches('_');
        let snake = self.to_snake_case(name_without_prefix);

        let result = if has_leading_underscore {
            format!("_{}", snake)
        } else {
            snake
        };

        // B37: Escape Rust reserved keywords with r# prefix
        escape_rust_keyword(&result)
    }

    fn sanitize_test_name(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
    }

    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut prev_lowercase = false;

        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && prev_lowercase {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
                prev_lowercase = false;
            } else {
                result.push(ch);
                prev_lowercase = ch.is_lowercase();
            }
        }

        result
    }

    /// Check if a string is in camelCase (has uppercase letters that aren't at the start)
    #[allow(dead_code)]
    fn is_camel_case(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .any(|(i, ch)| i > 0 && ch.is_uppercase())
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lowercase = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && prev_lowercase {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_lowercase = false;
        } else {
            result.push(ch);
            prev_lowercase = ch.is_lowercase();
        }
    }

    if result.is_empty() {
        "_".into()
    } else {
        result
    }
}

/// B37: Escape Rust reserved keywords with r# prefix.
/// This allows Liva identifiers like `type`, `match`, `mod` to compile as valid Rust.
fn escape_rust_keyword(name: &str) -> String {
    match name {
        // Strict keywords (cannot be used as identifiers without r#)
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern"
        | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match"
        | "mod" | "move" | "mut" | "pub" | "ref" | "return" | "static" | "struct"
        | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"
        | "async" | "await" | "dyn"
        // Reserved keywords (reserved for potential future use)
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override"
        | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" => {
            format!("r#{}", name)
        }
        _ => name.to_string(),
    }
}

// ===== AST-level async detection (for test framework) =====

/// Check if an AST LambdaBody contains any async calls or await expressions
fn ast_lambda_body_has_async(body: &LambdaBody) -> bool {
    match body {
        LambdaBody::Block(block) => block.stmts.iter().any(ast_stmt_has_async),
        LambdaBody::Expr(expr) => ast_expr_has_async(expr),
    }
}

fn ast_stmt_has_async(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::VarDecl(decl) => ast_expr_has_async(&decl.init),
        Stmt::ConstDecl(decl) => ast_expr_has_async(&decl.init),
        Stmt::Assign(assign) => {
            ast_expr_has_async(&assign.target) || ast_expr_has_async(&assign.value)
        }
        Stmt::Return(ret) => ret.expr.as_ref().map(ast_expr_has_async).unwrap_or(false),
        Stmt::Break | Stmt::Continue => false,
        Stmt::Throw(t) => ast_expr_has_async(&t.expr),
        Stmt::Fail(f) => ast_expr_has_async(&f.expr),
        Stmt::Expr(e) => ast_expr_has_async(&e.expr),
        Stmt::If(if_stmt) => {
            ast_expr_has_async(&if_stmt.condition)
                || ast_if_body_has_async(&if_stmt.then_branch)
                || if_stmt
                    .else_branch
                    .as_ref()
                    .map(ast_if_body_has_async)
                    .unwrap_or(false)
        }
        Stmt::While(w) => {
            ast_expr_has_async(&w.condition) || w.body.stmts.iter().any(ast_stmt_has_async)
        }
        Stmt::For(f) => {
            ast_expr_has_async(&f.iterable) || f.body.stmts.iter().any(ast_stmt_has_async)
        }
        Stmt::Block(block) => block.stmts.iter().any(ast_stmt_has_async),
        Stmt::Defer(defer_stmt) => ast_stmt_has_async(&defer_stmt.body),
        Stmt::TryCatch(tc) => {
            tc.try_block.stmts.iter().any(ast_stmt_has_async)
                || tc.catch_block.stmts.iter().any(ast_stmt_has_async)
        }
        Stmt::Switch(sw) => {
            ast_expr_has_async(&sw.discriminant)
                || sw
                    .cases
                    .iter()
                    .any(|c| ast_expr_has_async(&c.value) || c.body.iter().any(ast_stmt_has_async))
                || sw
                    .default
                    .as_ref()
                    .map(|stmts| stmts.iter().any(ast_stmt_has_async))
                    .unwrap_or(false)
        }
    }
}

fn ast_if_body_has_async(body: &IfBody) -> bool {
    match body {
        IfBody::Block(block) => block.stmts.iter().any(ast_stmt_has_async),
        IfBody::Stmt(stmt) => ast_stmt_has_async(stmt),
    }
}

fn ast_expr_has_async(expr: &Expr) -> bool {
    match expr {
        // Direct async indicators
        Expr::Call(call)
            if call.exec_policy == ExecPolicy::Async
                || call.exec_policy == ExecPolicy::TaskAsync =>
        {
            true
        }
        Expr::Unary {
            op: UnOp::Await, ..
        } => true,
        // Recursive traversal
        Expr::Call(call) => {
            ast_expr_has_async(&call.callee) || call.args.iter().any(ast_expr_has_async)
        }
        Expr::MethodCall(mc) => {
            ast_expr_has_async(&mc.object) || mc.args.iter().any(ast_expr_has_async)
        }
        Expr::Binary { left, right, .. } => ast_expr_has_async(left) || ast_expr_has_async(right),
        Expr::Unary { operand, .. } => ast_expr_has_async(operand),
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            ast_expr_has_async(condition)
                || ast_expr_has_async(then_expr)
                || ast_expr_has_async(else_expr)
        }
        Expr::Member { object, .. } => ast_expr_has_async(object),
        Expr::Index { object, index } => ast_expr_has_async(object) || ast_expr_has_async(index),
        Expr::ObjectLiteral(fields) => fields.iter().any(|(_, v)| ast_expr_has_async(v)),
        Expr::StructLiteral { fields, .. } => fields.iter().any(|(_, v)| ast_expr_has_async(v)),
        Expr::ArrayLiteral(elements) => elements.iter().any(ast_expr_has_async),
        Expr::Tuple(elements) => elements.iter().any(ast_expr_has_async),
        Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
            StringTemplatePart::Text(_) => false,
            StringTemplatePart::Expr(e) => ast_expr_has_async(e),
        }),
        Expr::Fail(inner) => ast_expr_has_async(inner),
        Expr::Lambda(lambda) => ast_lambda_body_has_async(&lambda.body),
        Expr::Switch(sw) => {
            ast_expr_has_async(&sw.discriminant)
                || sw.arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .map(|g| ast_expr_has_async(g))
                        .unwrap_or(false)
                        || match &arm.body {
                            SwitchBody::Expr(e) => ast_expr_has_async(e),
                            SwitchBody::Block(stmts) => stmts.iter().any(ast_stmt_has_async),
                        }
                })
        }
        Expr::MapLiteral(entries) => entries
            .iter()
            .any(|(k, v)| ast_expr_has_async(k) || ast_expr_has_async(v)),
        Expr::SetLiteral(elements) => elements.iter().any(ast_expr_has_async),
        Expr::Literal(_) | Expr::Identifier(_) | Expr::MethodRef { .. } => false,
        Expr::Unwrap(inner) => ast_expr_has_async(inner),
        Expr::OptionalChain { object, .. } => ast_expr_has_async(object),
        // B24 fix: check rust { } blocks for .await
        Expr::RustBlock { code } => code.contains(".await"),
    }
}

// ===== Multi-file project generation =====
pub fn generate_multifile_project(
    modules: &[&crate::module::Module],
    entry_module: &crate::module::Module,
    ctx: DesugarContext,
) -> Result<std::collections::HashMap<std::path::PathBuf, String>> {
    use std::collections::HashMap;
    use std::path::PathBuf;

    let mut files = HashMap::new();
    let mut mod_declarations = Vec::new();

    // Generate code for each module
    for module in modules {
        let module_name = module
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module");

        // Skip main/entry module - it will be handled separately
        if module.path == entry_module.path {
            continue;
        }

        // Generate Rust code for this module
        let rust_code = generate_module_code(module, &ctx, modules)?;

        // Determine output path: src/module_name.rs
        let output_path = PathBuf::from("src").join(format!("{}.rs", module_name));
        files.insert(output_path, rust_code);

        // Add mod declaration
        mod_declarations.push(format!("mod {};", module_name));
    }

    // Generate main.rs (entry point)
    let main_code = generate_entry_point(entry_module, &mod_declarations, &ctx, modules)?;
    files.insert(PathBuf::from("src/main.rs"), main_code);

    Ok(files)
}

/// Generate Rust code for a single Liva module
fn generate_module_code(module: &crate::module::Module, ctx: &DesugarContext, all_modules: &[&crate::module::Module]) -> Result<String> {
    let mut codegen = CodeGenerator::new(ctx.clone());

    // B06 fix: Pre-populate enum metadata so enum variants are recognized as
    // expressions (e.g., Priority.Alta) instead of falling through to get_field()
    // B98 fix: Also pre-populate from ALL imported modules, not just the current one
    for m in std::iter::once(&module as &crate::module::Module).chain(all_modules.iter().copied()) {
        for item in &m.ast.items {
            if let TopLevel::Enum(enum_decl) = item {
                codegen.enum_names.insert(enum_decl.name.clone());
                let mut variants_map = std::collections::HashMap::new();
                let mut variants_type_map = std::collections::HashMap::new();
                for variant in &enum_decl.variants {
                    let field_names: Vec<String> =
                        variant.fields.iter().map(|f| f.name.clone()).collect();
                    let field_types: Vec<(String, TypeRef)> =
                        variant.fields.iter().map(|f| (f.name.clone(), f.type_ref.clone())).collect();
                    variants_map.insert(variant.name.clone(), field_names);
                    variants_type_map.insert(variant.name.clone(), field_types);

                    // Pre-populate optional fields per variant for Some() wrapping
                    let variant_optionals: Vec<bool> = variant.fields.iter()
                        .map(|f| matches!(&f.type_ref, TypeRef::Optional(_)))
                        .collect();
                    if variant_optionals.iter().any(|&o| o) {
                        let key = format!("{}::{}", enum_decl.name, variant.name);
                        codegen.enum_variant_optionals.insert(key, variant_optionals);
                    }
                }
                codegen.enum_variants.insert(enum_decl.name.clone(), variants_map);
                codegen.enum_variant_field_types.insert(enum_decl.name.clone(), variants_type_map);

                // SH-006 fix: Pre-populate boxed_enum_fields for recursive enum auto-boxing
                let mut boxed_fields_for_enum: std::collections::HashMap<
                    String,
                    std::collections::HashSet<String>,
                > = std::collections::HashMap::new();
                for variant in &enum_decl.variants {
                    for field in &variant.fields {
                        if CodeGenerator::is_recursive_field(&field.type_ref, &enum_decl.name) {
                            boxed_fields_for_enum
                                .entry(variant.name.clone())
                                .or_default()
                                .insert(field.name.clone());
                        }
                    }
                }
                if !boxed_fields_for_enum.is_empty() {
                    codegen.boxed_enum_fields
                        .insert(enum_decl.name.clone(), boxed_fields_for_enum);
                }
            }
            // B98 fix: Also pre-populate fallible functions/methods from imported modules
            if let TopLevel::Function(func) = item {
                if func.contains_fail {
                    codegen.fallible_functions.insert(func.name.clone());
                }
                // B100 fix: Pre-populate array/string returning functions from imported modules
                if let Some(ret_type) = &func.return_type {
                    if let TypeRef::Array(elem) = ret_type {
                        let elem_type = match elem.as_ref() {
                            TypeRef::Simple(name) => name.clone(),
                            _ => String::new(),
                        };
                        codegen.array_returning_functions.insert(func.name.clone(), elem_type);
                    }
                    if matches!(ret_type, TypeRef::Simple(name) if name == "string") {
                        codegen.string_returning_functions.insert(func.name.clone());
                    }
                    // BUG-007: Track functions returning T? (Option<T>)
                    if matches!(ret_type, TypeRef::Optional(_)) {
                        codegen.optional_returning_functions.insert(func.name.clone());
                    }
                }
            }
            if let TopLevel::Class(class) = item {
                // BUG-003 fix: Pre-populate class_fields from all modules so
                // register_pattern_bindings recognizes imported class types
                let mut fields = std::collections::HashSet::new();
                let mut optional_fields = std::collections::HashSet::new();
                let mut array_field_types = std::collections::HashMap::new();
                for m2 in &class.members {
                    if let crate::ast::Member::Field(f) = m2 {
                        fields.insert(f.name.clone());
                        if f.is_optional || matches!(&f.type_ref, Some(TypeRef::Optional(_))) {
                            optional_fields.insert(f.name.clone());
                        }
                        // BUG-003 fix: Track array field element types for for-loop var typing
                        if let Some(TypeRef::Array(element_type)) = &f.type_ref {
                            if let TypeRef::Simple(type_name) = element_type.as_ref() {
                                array_field_types.insert(f.name.clone(), type_name.clone());
                            }
                        }
                    }
                }
                codegen.class_fields.insert(class.name.clone(), fields);
                codegen.class_optional_fields.insert(class.name.clone(), optional_fields);
                if !array_field_types.is_empty() {
                    codegen.class_array_field_types.insert(class.name.clone(), array_field_types);
                }

                for member in &class.members {
                    if let crate::ast::Member::Method(method) = member {
                        if method.contains_fail {
                            codegen.fallible_methods.insert(method.name.clone());
                        }
                        // B100 fix: Pre-populate string/array returning methods from imported classes
                        if let Some(ret_type) = &method.return_type {
                            if matches!(ret_type, TypeRef::Simple(name) if name == "string") {
                                codegen.string_returning_methods.insert(method.name.clone());
                            }
                            if let TypeRef::Array(elem) = ret_type {
                                let elem_type = match elem.as_ref() {
                                    TypeRef::Simple(name) => name.clone(),
                                    _ => String::new(),
                                };
                                codegen.array_returning_methods.insert(method.name.clone(), elem_type);
                            }
                        }
                    }
                }
            }
        }
    }

    // First, collect use statements from imports
    let mut use_statements = String::new();
    for import_decl in &module.imports {
        let use_stmt = generate_use_statement(import_decl, &module.path)?;
        use_statements.push_str(&use_stmt);
        use_statements.push('\n');
    }

    // Generate code for each top-level item into a separate buffer
    let mut module_body = String::new();

    for item in &module.ast.items {
        match item {
            TopLevel::Import(_) => {
                // Already handled above
                continue;
            }
            TopLevel::Function(func) => {
                let is_public = !func.name.starts_with('_');

                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_function(func)?;
                let func_code = codegen.output.clone();

                if is_public {
                    module_body.push_str("pub ");
                }
                module_body.push_str(&func_code);
                module_body.push('\n');
            }
            TopLevel::Class(class) => {
                let is_public = !class.name.starts_with('_');

                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_class(class)?;
                let class_code = codegen.output.clone();

                if is_public {
                    // Add pub to struct definition
                    let lines: Vec<&str> = class_code.lines().collect();
                    if let Some(first_line) = lines.first() {
                        if first_line.starts_with("struct") {
                            module_body.push_str("pub ");
                        }
                    }
                }
                module_body.push_str(&class_code);
                module_body.push('\n');
            }
            TopLevel::Type(type_decl) => {
                let is_public = !type_decl.name.starts_with('_');

                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_type_decl(type_decl)?;
                let type_code = codegen.output.clone();

                if is_public {
                    module_body.push_str("pub ");
                }
                module_body.push_str(&type_code);
                module_body.push('\n');
            }
            TopLevel::TypeAlias(_) => {
                // Type aliases are expanded inline, no code generation needed
                continue;
            }
            TopLevel::ConstDecl(const_decl) => {
                let is_public = !const_decl.name.starts_with('_');

                codegen.output.clear();
                codegen.generate_top_level(item)?;
                let code = codegen.output.clone();

                if is_public {
                    module_body.push_str("pub ");
                }
                module_body.push_str(&code);
                module_body.push('\n');
            }
            TopLevel::Enum(enum_decl) => {
                let is_public = !enum_decl.name.starts_with('_');

                codegen.output.clear();
                codegen.generate_top_level(item)?;
                let code = codegen.output.clone();

                if is_public {
                    // Add pub to enum definition and its Display impl
                    let code = code.replacen("enum ", "pub enum ", 1);
                    module_body.push_str(&code);
                } else {
                    module_body.push_str(&code);
                }
                module_body.push('\n');
            }
            TopLevel::UseRust(_)
            | TopLevel::Test(_)
            | TopLevel::ExprStmt(_) => {
                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_top_level(item)?;
                let code = codegen.output.clone();

                module_body.push_str(&code);
                module_body.push('\n');
            }
        }
    }

    // Now build the final output, only adding liva_rt import if needed
    let mut output = String::new();

    // Only add liva_rt import if the module actually uses it
    if module_body.contains("liva_rt::") || use_statements.contains("liva_rt::") {
        output.push_str("use crate::liva_rt;\n\n");
    }

    // Add use statements (with allow(unused_imports) to suppress warnings for pass-through types)
    if !use_statements.is_empty() {
        for line in use_statements.lines() {
            if line.starts_with("use ") {
                output.push_str("#[allow(unused_imports)]\n");
            }
            output.push_str(line);
            output.push('\n');
        }
        output.push('\n');
    }

    // Add module body
    output.push_str(&module_body);

    Ok(output)
}

/// Convert a Liva import to a Rust use statement
/// Examples:
/// - `import { add } from "./math.liva"` → `use crate::math::add;`
/// - `import * as math from "./math.liva"` → `use crate::math;`
fn generate_use_statement(
    import_decl: &ImportDecl,
    _current_module_path: &std::path::Path,
) -> Result<String> {
    use std::path::Path;

    // Virtual modules (liva/test, etc.) don't generate use statements
    if crate::module::is_virtual_module(&import_decl.source) {
        return Ok(String::new());
    }

    // Parse the source path and resolve relative to current module
    let source_path = Path::new(&import_decl.source);

    // Remove .liva extension if present
    let module_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            crate::CompilerError::CodegenError(crate::error::SemanticErrorInfo::new(
                "E9001",
                &format!("Invalid module path: {}", import_decl.source),
                "",
            ))
        })?;

    // Convert relative path to Rust module path
    let rust_module_path = if import_decl.source.starts_with("./") {
        // Same directory: ./math.liva → crate::math
        format!("crate::{}", module_name)
    } else if import_decl.source.starts_with("../") {
        // Parent directory: ../utils/math.liva → crate::utils::math
        // For now, simplify to crate::module_name
        format!("crate::{}", module_name)
    } else {
        // Absolute or other: treat as crate::module_name
        format!("crate::{}", module_name)
    };

    if import_decl.is_wildcard {
        // Wildcard import: import * as alias from "..."
        if let Some(alias) = &import_decl.alias {
            // Only use 'as' if alias is different from module name
            if alias != module_name {
                Ok(format!("use {} as {};", rust_module_path, alias))
            } else {
                // If alias == module_name, just import the module itself
                Ok(format!("use {};", rust_module_path))
            }
        } else {
            Ok(format!("use {}::*;", rust_module_path))
        }
    } else if import_decl.imports.len() == 1 {
        // Single import
        let symbol = &import_decl.imports[0];
        // Don't convert if it starts with uppercase (it's a type)
        let rust_symbol = if symbol
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
            symbol.clone()
        } else {
            to_snake_case(symbol)
        };
        Ok(format!("use {}::{};", rust_module_path, rust_symbol))
    } else {
        // Multiple imports: use crate::math::{add, subtract};
        let rust_symbols: Vec<String> = import_decl
            .imports
            .iter()
            .map(|s| {
                // Don't convert if it starts with uppercase (it's a type)
                if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    s.clone()
                } else {
                    to_snake_case(s)
                }
            })
            .collect();
        let symbols = rust_symbols.join(", ");
        Ok(format!("use {}::{{{}}};", rust_module_path, symbols))
    }
}

/// Generate the entry point (main.rs) with mod declarations and main function
fn generate_entry_point(
    entry_module: &crate::module::Module,
    mod_declarations: &[String],
    ctx: &DesugarContext,
    all_modules: &[&crate::module::Module],
) -> Result<String> {
    let mut codegen = CodeGenerator::new(ctx.clone());

    // Pre-populate enum metadata from all imported modules so the entry module
    // can reference imported enums (e.g., Color.Red) correctly
    for module in all_modules {
        for item in &module.ast.items {
            if let TopLevel::Enum(enum_decl) = item {
                codegen.enum_names.insert(enum_decl.name.clone());
                let mut variants_map = std::collections::HashMap::new();
                let mut variants_type_map = std::collections::HashMap::new();
                for variant in &enum_decl.variants {
                    let field_names: Vec<String> =
                        variant.fields.iter().map(|f| f.name.clone()).collect();
                    let field_types: Vec<(String, TypeRef)> =
                        variant.fields.iter().map(|f| (f.name.clone(), f.type_ref.clone())).collect();
                    variants_map.insert(variant.name.clone(), field_names);
                    variants_type_map.insert(variant.name.clone(), field_types);

                    // Pre-populate optional fields per variant for Some() wrapping
                    let variant_optionals: Vec<bool> = variant.fields.iter()
                        .map(|f| matches!(&f.type_ref, TypeRef::Optional(_)))
                        .collect();
                    if variant_optionals.iter().any(|&o| o) {
                        let key = format!("{}::{}", enum_decl.name, variant.name);
                        codegen.enum_variant_optionals.insert(key, variant_optionals);
                    }
                }
                codegen.enum_variants
                    .insert(enum_decl.name.clone(), variants_map);
                codegen.enum_variant_field_types
                    .insert(enum_decl.name.clone(), variants_type_map);

                // SH-006 fix: Pre-populate boxed_enum_fields for recursive enum auto-boxing
                let mut boxed_fields_for_enum: std::collections::HashMap<
                    String,
                    std::collections::HashSet<String>,
                > = std::collections::HashMap::new();
                for variant in &enum_decl.variants {
                    for field in &variant.fields {
                        if CodeGenerator::is_recursive_field(&field.type_ref, &enum_decl.name) {
                            boxed_fields_for_enum
                                .entry(variant.name.clone())
                                .or_default()
                                .insert(field.name.clone());
                        }
                    }
                }
                if !boxed_fields_for_enum.is_empty() {
                    codegen.boxed_enum_fields
                        .insert(enum_decl.name.clone(), boxed_fields_for_enum);
                }
            }
            // B23 fix: Pre-populate fallible functions from imported modules
            // Without this, cross-file error binding generates (fn(), None) instead of match { Ok/Err }
            if let TopLevel::Function(func) = item {
                if func.contains_fail {
                    codegen.fallible_functions.insert(func.name.clone());
                }
                // B100 fix: Pre-populate array/string returning functions from imported modules
                if let Some(ret_type) = &func.return_type {
                    if let TypeRef::Array(elem) = ret_type {
                        let elem_type = match elem.as_ref() {
                            TypeRef::Simple(name) => name.clone(),
                            _ => String::new(),
                        };
                        codegen.array_returning_functions.insert(func.name.clone(), elem_type);
                    }
                    if matches!(ret_type, TypeRef::Simple(name) if name == "string") {
                        codegen.string_returning_functions.insert(func.name.clone());
                    }
                    // BUG-007: Track functions returning T? (Option<T>)
                    if matches!(ret_type, TypeRef::Optional(_)) {
                        codegen.optional_returning_functions.insert(func.name.clone());
                    }
                }
            }
            // B23 fix: Also pre-populate fallible methods from imported classes
            if let TopLevel::Class(class) = item {
                // BUG-003 fix: Pre-populate class_fields from all modules so
                // register_pattern_bindings recognizes imported class types
                let mut fields = std::collections::HashSet::new();
                let mut optional_fields = std::collections::HashSet::new();
                let mut array_field_types = std::collections::HashMap::new();
                for m2 in &class.members {
                    if let crate::ast::Member::Field(f) = m2 {
                        fields.insert(f.name.clone());
                        if f.is_optional || matches!(&f.type_ref, Some(TypeRef::Optional(_))) {
                            optional_fields.insert(f.name.clone());
                        }
                        // BUG-003 fix: Track array field element types for for-loop var typing
                        if let Some(TypeRef::Array(element_type)) = &f.type_ref {
                            if let TypeRef::Simple(type_name) = element_type.as_ref() {
                                array_field_types.insert(f.name.clone(), type_name.clone());
                            }
                        }
                        // SH: Track Map-typed fields from imported classes for deep member access
                        if matches!(&f.type_ref, Some(TypeRef::Map(_, _))) {
                            codegen.map_vars.insert(codegen.sanitize_name(&f.name));
                        }
                        // SH: Track Set-typed fields from imported classes
                        if matches!(&f.type_ref, Some(TypeRef::Set(_))) {
                            codegen.set_vars.insert(codegen.sanitize_name(&f.name));
                        }
                    }
                }
                codegen.class_fields.insert(class.name.clone(), fields);
                codegen.class_optional_fields.insert(class.name.clone(), optional_fields);
                if !array_field_types.is_empty() {
                    codegen.class_array_field_types.insert(class.name.clone(), array_field_types);
                }

                for member in &class.members {
                    if let crate::ast::Member::Method(method) = member {
                        if method.contains_fail {
                            codegen.fallible_methods.insert(method.name.clone());
                        }
                        // B100 fix: Pre-populate string/array returning methods from imported classes
                        if let Some(ret_type) = &method.return_type {
                            if matches!(ret_type, TypeRef::Simple(name) if name == "string") {
                                codegen.string_returning_methods.insert(method.name.clone());
                            }
                            if let TypeRef::Array(elem) = ret_type {
                                let elem_type = match elem.as_ref() {
                                    TypeRef::Simple(name) => name.clone(),
                                    _ => String::new(),
                                };
                                codegen.array_returning_methods.insert(method.name.clone(), elem_type);
                            }
                        }
                    }
                }
            }
        }
    }

    // Suppress common codegen warnings (must be at top of main.rs)
    codegen.writeln("#![allow(unused_parens, unused_mut)]");

    // Add mod declarations for all other modules
    for mod_decl in mod_declarations {
        codegen.writeln(mod_decl);
    }

    if !mod_declarations.is_empty() {
        codegen.output.push('\n'); // Blank line after mod declarations
    }

    // Generate use statements from entry module's imports
    // Also register module aliases for wildcard imports
    for import_decl in &entry_module.imports {
        if import_decl.is_wildcard && import_decl.alias.is_some() {
            // Wildcard import with alias like `import * as utils from "./utils.liva"`
            // Register the alias -> module_name mapping for code generation
            let source_path = std::path::Path::new(&import_decl.source);
            if let Some(module_name) = source_path.file_stem().and_then(|s| s.to_str()) {
                let alias = import_decl.alias.as_ref().unwrap();
                codegen
                    .module_aliases
                    .insert(alias.clone(), module_name.to_string());
            }
            // The module is already available via `mod math;`, skip the use statement
            continue;
        }

        let use_stmt = generate_use_statement(import_decl, &entry_module.path)?;
        codegen.output.push_str("#[allow(unused_imports)]\n");
        codegen.output.push_str(&use_stmt);
        codegen.output.push('\n');
    }

    if !entry_module.imports.is_empty() {
        codegen.output.push('\n');
    }

    // Generate the entry module using generate_program logic
    codegen.generate_program(&entry_module.ast)?;

    Ok(codegen.output.clone())
}

pub fn generate_with_ast(program: &Program, ctx: DesugarContext) -> Result<(String, String)> {
    let mut generator = CodeGenerator::new(ctx);

    // Suppress common codegen warnings (crate-level attribute for single-file projects)
    generator.writeln("#![allow(unused_parens, unused_mut)]");

    // First pass: collect fallible functions and array-returning functions
    for item in &program.items {
        if let TopLevel::Function(func) = item {
            if func.contains_fail {
                generator.fallible_functions.insert(func.name.clone());
            }
            // Track functions that return [T] (arrays)
            if let Some(ret_type) = &func.return_type {
                if let TypeRef::Array(elem) = ret_type {
                    let elem_type = match elem.as_ref() {
                        TypeRef::Simple(name) => name.clone(),
                        _ => String::new(),
                    };
                    generator
                        .array_returning_functions
                        .insert(func.name.clone(), elem_type);
                }
                // Track functions that return string
                if matches!(ret_type, TypeRef::Simple(name) if name == "string") {
                    generator
                        .string_returning_functions
                        .insert(func.name.clone());
                }
                // BUG-007: Track functions returning T? (Option<T>)
                if matches!(ret_type, TypeRef::Optional(_)) {
                    generator
                        .optional_returning_functions
                        .insert(func.name.clone());
                }
            }
        }
        // B19 fix: Scan class methods for fallible (contains_fail) 
        if let TopLevel::Class(class) = item {
            for member in &class.members {
                if let Member::Method(method) = member {
                    if method.contains_fail {
                        generator.fallible_methods.insert(method.name.clone());
                    }
                }
            }
        }
    }

    generator.generate_program(program)?;

    // Insert hoisted `use` statements from `rust { }` blocks at the top of the file
    if !generator.rust_block_uses.is_empty() {
        let mut hoisted = String::new();
        for use_stmt in &generator.rust_block_uses {
            hoisted.push_str(use_stmt);
            hoisted.push('\n');
        }
        hoisted.push('\n');
        // Insert after the #![allow(...)] line
        if let Some(pos) = generator.output.find('\n') {
            generator.output.insert_str(pos + 1, &hoisted);
        }
    }

    let cargo_toml = generate_cargo_toml(&generator.ctx)?;

    Ok((generator.output, cargo_toml))
}

pub fn generate_cargo_toml(ctx: &DesugarContext) -> Result<String> {
    let mut cargo_toml = String::from(
        "[package]\n\
         name = \"liva_project\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\n\
         [dependencies]\n",
    );

    // Helper: collect extra features a user wants for an internal crate
    let user_features_for = |crate_name: &str| -> Vec<String> {
        ctx.rust_crates
            .iter()
            .filter(|dep| dep.name == crate_name)
            .flat_map(|dep| dep.features.clone())
            .collect()
    };

    // Always add tokio since liva_rt uses it
    {
        let mut feats: Vec<String> = vec!["full".to_string()];
        feats.extend(user_features_for("tokio"));
        feats.dedup();
        let feats_str: Vec<String> = feats.iter().map(|f| format!("\"{}\"", f)).collect();
        writeln!(cargo_toml, "tokio = {{ version = \"1\", features = [{}] }}", feats_str.join(", ")).unwrap();
    }

    // Add serde and serde_json (serde needed for derive macros in Phase 2)
    {
        let mut feats: Vec<String> = vec!["derive".to_string()];
        feats.extend(user_features_for("serde"));
        feats.dedup();
        let feats_str: Vec<String> = feats.iter().map(|f| format!("\"{}\"", f)).collect();
        writeln!(cargo_toml, "serde = {{ version = \"1.0\", features = [{}] }}", feats_str.join(", ")).unwrap();
    }
    cargo_toml.push_str("serde_json = \"1.0\"\n");

    // Add reqwest for HTTP client
    {
        let mut feats: Vec<String> = vec!["json".to_string(), "rustls-tls".to_string()];
        feats.extend(user_features_for("reqwest"));
        feats.dedup();
        let feats_str: Vec<String> = feats.iter().map(|f| format!("\"{}\"", f)).collect();
        writeln!(cargo_toml, "reqwest = {{ version = \"0.11\", default-features = false, features = [{}] }}", feats_str.join(", ")).unwrap();
    }

    if ctx.has_parallel {
        cargo_toml.push_str("rayon = \"1.11\"\n");
    }

    if ctx.has_random {
        cargo_toml.push_str("rand = \"0.8\"\n");
        cargo_toml.push_str("uuid = { version = \"1\", features = [\"v4\"] }\n");
    }

    if ctx.has_logging || ctx.has_date {
        cargo_toml.push_str("chrono = \"0.4\"\n");
    }

    if ctx.has_regex {
        cargo_toml.push_str("regex = \"1\"\n");
    }

    if ctx.has_crypto {
        cargo_toml.push_str("sha2 = \"0.10\"\n");
        cargo_toml.push_str("md-5 = \"0.10\"\n");
        cargo_toml.push_str("base64 = \"0.22\"\n");
    }

    if ctx.has_server {
        cargo_toml.push_str("axum = \"0.8\"\n");
    }

    if ctx.has_db {
        cargo_toml.push_str("rusqlite = { version = \"0.32\", features = [\"bundled\"] }\n");
    }

    // Add user-specified crates (with version and features support)
    for dep in &ctx.rust_crates {
        let crate_name = &dep.name;
        // Skip internal crates that are already added above
        if crate_name == "tokio" || crate_name == "serde" || crate_name == "serde_json"
            || crate_name == "reqwest" || crate_name == "rayon" || crate_name == "rand"
            || (crate_name == "chrono" && (ctx.has_logging || ctx.has_date))
            || (crate_name == "regex" && ctx.has_regex)
            || ((crate_name == "sha2" || crate_name == "md-5" || crate_name == "base64") && ctx.has_crypto)
            || (crate_name == "uuid" && ctx.has_random)
            || (crate_name == "axum" && ctx.has_server)
            || (crate_name == "rusqlite" && ctx.has_db) {
            // For internal crates, only merge additional features
            if !dep.features.is_empty() {
                // Already handled: user can add features to internal crates
                // The merge will be done in the internal crate section above
            }
            continue;
        }
        if dep.features.is_empty() {
            if let Some(ver) = &dep.version {
                writeln!(cargo_toml, "{} = \"{}\"", crate_name, ver).unwrap();
            } else {
                writeln!(cargo_toml, "{} = \"*\"", crate_name).unwrap();
            }
        } else {
            let ver = dep.version.as_deref().unwrap_or("*");
            let feats: Vec<String> = dep.features.iter().map(|f| format!("\"{}\"", f)).collect();
            writeln!(cargo_toml, "{} = {{ version = \"{}\", features = [{}] }}", crate_name, ver, feats.join(", ")).unwrap();
        }
    }

    Ok(cargo_toml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        let gen = CodeGenerator::new(DesugarContext {
            rust_crates: vec![],
            has_async: false,
            has_parallel: false,
            has_random: false,
            has_rust_blocks: false,
            has_logging: false,
            has_config: false,
            has_regex: false,
            has_date: false,
            has_crypto: false,
            has_server: false,
            has_db: false,
            async_functions: std::collections::BTreeSet::new(),
            source_filename: String::new(),
        });

        assert_eq!(gen.to_snake_case("CamelCase"), "camel_case");
        assert_eq!(gen.to_snake_case("myFunction"), "my_function");
        assert_eq!(gen.to_snake_case("snake_case"), "snake_case");
    }
}
