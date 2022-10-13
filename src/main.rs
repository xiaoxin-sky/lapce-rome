use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::Result;
use flate2::read::GzDecoder;
use lapce_plugin::{
    psp_types::{
        lsp_types::{request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, Url},
        Request,
    },
    register_plugin, Http, LapcePlugin, PLUGIN_RPC,
};
use serde::Deserialize;
use serde_json::Value;
use tar_wasi::Archive;

#[derive(Default)]
struct State {}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    PLUGIN_RPC.stderr("启动lapce-rome");
    // download_lsp_server()?;

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

    let server_args = vec!["start".to_string()];
    let volt_uri = std::env::var("VOLT_URI")?;
    let server_path = Url::parse(&volt_uri).unwrap().join("rome").unwrap();

    PLUGIN_RPC.stderr(&format!("server_path：{}", server_path));
    PLUGIN_RPC.start_lsp(
        server_path,
        server_args.clone(),
        document_selector.clone(),
        params.initialization_options,
    );
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

#[derive(Deserialize, Debug)]
struct VoltConfig {
    version: String,
}

fn download_lsp_server() -> Result<bool> {
    let volt_path = Path::new("volt.toml");

    let volt_str = fs::read_to_string(volt_path)?;

    let volt_toml: VoltConfig = toml::from_str(&volt_str)?;

    let arch = match std::env::var("VOLT_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("aarch64") => "arm64",
        _ => panic!("unknow arch"),
    };
    let os = match std::env::var("VOLT_OS").as_deref() {
        Ok("linux") => "linux",
        Ok("macos") => "darwin",
        Ok("windows") => "win32",
        _ => panic!("unknow os"),
    };

    let lsp_server_name = format!("rome");

    let lsp_server_binary_name = format!("{}-{}-{}", lsp_server_name, os, arch);
    // let lapce_volar_gz_path_name = format!("{}.tar.gz", &lsp_server_binary_name.clone());
    // let lapce_volar_gz_path = Path::new(&lapce_volar_gz_path_name);

    if !Path::new(&lsp_server_binary_name).exists() && !Path::new(&lsp_server_name).exists()
    {
        let volt_download_url = format!(
            "https://github.com/rome/tools/releases/download/cli%2Fv{}/{}",
            &volt_toml.version, &lsp_server_binary_name,
        );
        PLUGIN_RPC.stderr(&format!("下载地址_{}", volt_download_url));

        let mut resp = Http::get(&volt_download_url)?;
        let body = resp.body_read_all()?;
        fs::write(&lsp_server_binary_name, body)?;
    } else {
        PLUGIN_RPC.stderr("已存在,跳过下载");
    }

    fs::rename(lsp_server_binary_name, lsp_server_name)?;
    // let tar_gz = File::open(&lapce_volar_gz_path_name);
    // match tar_gz {
    //     Ok(tar) => {
    //         let tar = GzDecoder::new(tar);
    //         let mut archive = Archive::new(tar);

    //         let res = archive.unpack(".");

    //         PLUGIN_RPC.stderr(&format!("{:#?}", res));
    //         fs::rename(lsp_server_binary_name, lsp_server_name)?;
    //     }
    //     Err(err) => {
    //         PLUGIN_RPC.stderr(&format!("{}", err));
    //     }
    // }

    Ok(true)
}
