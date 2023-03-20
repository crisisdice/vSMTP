// Rhai modules in the `rhai-fs` package.
mod pkg {
    include!("src/api.rs");
}

fn main() {
    if let Ok(docs_path) = std::env::var("DOCS_DIR") {
        let mut engine = rhai::Engine::new();

        engine.register_static_module("mysql", rhai::exported_module!(pkg::mysql_api).into());

        let docs = rhai_autodocs::options()
            .format_sections_with(rhai_autodocs::SectionFormat::Tabs)
            .include_standard_packages(false)
            .order_functions_with(rhai_autodocs::FunctionOrder::ByIndex)
            .generate(&engine)
            .expect("failed to generate documentation");

        write_docs(&docs_path, &docs);
    }
}

fn write_docs(path: &str, docs: &rhai_autodocs::ModuleDocumentation) {
    std::fs::write(
        std::path::PathBuf::from_iter([path, &format!("fn::{}.md", &docs.name)]),
        &docs.documentation,
    )
    .expect("failed to write documentation");

    for doc in &docs.sub_modules {
        write_docs(path, doc);
    }
}
