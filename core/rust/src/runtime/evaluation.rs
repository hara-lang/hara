impl Runtime {
    /// Returns the canonical schema value loaded from HALC for a named schema Var.
    pub fn halc_schema(&self, qualified_var: &str) -> Option<&Form> {
        self.halc_schema_definitions.get(qualified_var)
    }

    /// Returns the canonical schema annotation loaded from HALC for a function Var.
    pub fn halc_function_schema(&self, qualified_var: &str) -> Option<&Form> {
        self.halc_function_schemas.get(qualified_var)
    }

    /// Returns the normalized compiler type for a named schema Var.
    pub fn halc_schema_type(&self, qualified_var: &str) -> Option<&kernel::SchemaType> {
        self.halc_schema_types.get(qualified_var)
    }

    /// Returns a conservative body-derived function signature, when the
    /// compiler could prove one independently of the declared contract.
    pub fn halc_inferred_function_type(&self, qualified_var: &str) -> Option<&kernel::SchemaType> {
        self.halc_inferred_function_types.get(qualified_var)
    }

    /// Returns a function's normalized annotation, resolving one named edge.
    pub fn halc_function_type(&self, qualified_var: &str) -> Option<&kernel::SchemaType> {
        let schema = self.halc_function_types.get(qualified_var)?;
        match schema {
            kernel::SchemaType::Reference(name) => {
                self.halc_schema_types.get(name).or(Some(schema))
            }
            _ => Some(schema),
        }
    }

    /// Evaluates native Hara source and returns its runtime value without a
    /// display round trip. Embedding hosts use this to inspect declarative
    /// values containing Vars, functions, bytes, and persistent collections.
    #[cfg(any(not(target_arch = "wasm32"), target_os = "wasi"))]
    pub fn eval_native_value(&mut self, source: &str) -> Result<core::Value, String> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(handler) = self.native_host_handler.clone() {
            return core::with_host_calls(handler, || self.eval_value_mode(source, false));
        }
        self.eval_value_mode(source, false)
    }

    /// Evaluates once through the existing evaluator and returns a portable
    /// bounded Evaluation Journal.
    #[cfg(feature = "evaluation-journal")]
    pub fn eval_native_journal(&mut self, source: &str) -> journal::Journal {
        let journal_id = journal::JournalId(self.next_journal_id);
        self.next_journal_id += 1;
        let (_, journal) = core::with_evaluation_journal(
            journal_id,
            journal::JournalLimits::default(),
            || {
                self.refresh_qualified_bindings();
                let forms = kernel::parse_forms(source)?;
                let result = self.eval_forms(forms, true)?;
                self.save_namespace();
                self.refresh_qualified_bindings();
                Ok(result)
            },
            |value, collector| {
                collector.preview_value(core::portable_type_name(value), value.display())
            },
        );
        journal
    }

    #[cfg(feature = "evaluation-journal")]
    #[deprecated(note = "use eval_native_journal")]
    pub fn eval_native_trace(&mut self, source: &str) -> Result<journal::Journal, String> {
        let journal = self.eval_native_journal(source);
        match journal.status {
            journal::JournalStatus::Error => Err(journal.error.clone().unwrap_or_default()),
            _ => Ok(journal),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn add_extension_root(&mut self, root: impl Into<std::path::PathBuf>) {
        self.extension_roots.push(root.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn install_discovered_extension(&mut self, namespace: &str) -> Result<(), String> {
        if self.wasm_extensions.contains_key(namespace) {
            return Ok(());
        }
        let package =
            native_extension::ExtensionPackage::discover(namespace, &self.extension_roots)?
                .ok_or_else(|| format!("extension/not-found: {namespace}"))?;
        if package.manifest.provider == "hta" {
            let target = package
                .manifest
                .targets
                .get("node")
                .ok_or_else(|| format!("extension/target-unsupported: node for {namespace}"))?;
            if target.runtime != "process" {
                return Err(format!(
                    "extension/target-unsupported: node for {namespace}"
                ));
            }
            let module = package.resolve(&target.module)?;
            let provider = process_extension::ProcessExtensionProvider::new(module);
            return self.install_wasm_extension(
                &package.source,
                &package.descriptor.display().to_string(),
                provider,
            );
        }
        if package.manifest.provider != "wasm" {
            return Err(format!(
                "extension/provider-unsupported: {} for {namespace}",
                package.manifest.provider
            ));
        }
        let bytes = package.module_bytes()?;
        let provider = wasmtime_provider::WasmtimeExtensionProvider::compile(&bytes)?;
        self.install_wasm_extension(
            &package.source,
            &package.descriptor.display().to_string(),
            provider,
        )
    }

    pub fn install_wasm_extension<P: extension::WasmExtensionProvider + 'static>(
        &mut self,
        manifest_source: &str,
        origin: &str,
        provider: P,
    ) -> Result<(), String> {
        let manifest = extension::ExtensionManifest::parse(manifest_source, origin)?;
        let namespace = manifest.namespace.clone();
        if self.wasm_extensions.contains_key(&namespace)
            || self.extensions.contains(&namespace)
            || self.resources.contains_key(&namespace)
        {
            return Err(format!(
                "extension/ambiguous: namespace already registered: {namespace}"
            ));
        }
        let extension = extension::WasmExtension::new(manifest, provider)?;
        self.wasm_extensions.insert(namespace, extension);
        Ok(())
    }

    pub fn cancel_wasm_extension(&self, name: &str, request: u64) -> Result<(), String> {
        self.wasm_extensions
            .get(name)
            .ok_or_else(|| format!("extension/not-found: {name}"))?
            .cancel(request)
    }

    /// Invokes an installed WASM extension without routing the call through
    /// source text. Service hosts use this binary-safe boundary for HTA0
    /// arguments and results.
    pub fn invoke_wasm_extension(
        &mut self,
        namespace: &str,
        export: &str,
        arguments: &[extension::Value],
    ) -> Result<extension::Value, String> {
        let binding = self
            .wasm_extensions
            .get_mut(namespace)
            .ok_or_else(|| format!("extension/not-found: {namespace}"))?
            .require()?
            .into_iter()
            .find(|binding| binding.name == export)
            .ok_or_else(|| format!("extension/export-missing: {namespace}/{export}"))?;
        binding.invoke(arguments)
    }

    fn namespace_source(&self) -> Rc<dyn Fn(&str) -> Option<core::NamespaceResource>> {
        let resources = self.resources.clone();
        #[cfg(feature = "bytecode-vm")]
        let bytecode_resources = self.bytecode_resources.clone();
        Rc::new(move |name: &str| {
            #[cfg(feature = "bytecode-vm")]
            if let Some((namespace_form, artifact)) = bytecode_resources.get(name) {
                return Some(core::NamespaceResource::Bytecode {
                    namespace_form: namespace_form.clone(),
                    artifact: artifact.clone(),
                });
            }
            resources
                .get(name)
                .cloned()
                .map(core::NamespaceResource::Source)
        })
    }

    fn load_wasm_extension_namespace(&mut self, name: &str) -> Result<String, String> {
        let bindings = self
            .wasm_extensions
            .get_mut(name)
            .ok_or_else(|| format!("extension/not-found: {name}"))?
            .require()?;
        let namespace = self.namespace_registry.find_or_create(name);
        for binding in bindings {
            let arity = binding.specification.arguments.len();
            let function_name = format!("{name}/{}", binding.name);
            let binding_name = binding.name.clone();
            namespace.intern(
                &binding_name,
                core::native_function(&function_name, arity, move |arguments| {
                    binding.invoke(&arguments)
                }),
            );
        }
        self.refresh_qualified_bindings();
        Ok(":loaded".into())
    }
}


