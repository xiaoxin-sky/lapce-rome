use anyhow::Result;
use lapce_plugin::{
    psp_types::{
        lsp_types::{request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, Url},
        Request,
    },
    register_plugin, LapcePlugin, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![
        DocumentFilter {
            // lsp language id
            language: Some(String::from("javascript")),
            // glob pattern
            pattern: Some(String::from("**.js")),
            // like file:
            scheme: None,
        },
        DocumentFilter {
            // lsp language id
            language: Some(String::from("typescript")),
            // glob pattern
            pattern: Some(String::from("**.ts")),
            // like file:
            scheme: None,
        },
    ];

    let server_args = vec!["lsp-proxy".to_string()];
    let volt_uri = std::env::var("VOLT_URI")?;

    let server_path = params
        .initialization_options
        .as_ref()
        .and_then(|options| options.get("serverPath"))
        .and_then(|server_path| server_path.as_str())
        .and_then(|server_path| {
            if !server_path.is_empty() {
                Some(Url::parse(&format!("urn:{}", server_path)).unwrap())
            } else {
                Some(Url::parse(&volt_uri).unwrap().join("rome").unwrap())
            }
        });
    match server_path {
        Some(server_path) => {
            PLUGIN_RPC.start_lsp(
                server_path,
                server_args.clone(),
                document_selector.clone(),
                params.initialization_options,
            );
        }
        None => PLUGIN_RPC.stderr("server_path can't find"),
    }

    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                let _ = initialize(params);
            }
            _ => {}
        }
    }
}
