use tower_lsp::{LspService, Server};
use crate::server::Backend;

mod server;
mod document;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.contains(&"--lsp".to_string()) {
        run_lsp_server();
        return;
    }

    #[cfg(feature = "gui")]
    {
        if let Err(e) = finix::gui::run_app() {
            eprintln!("Failed to run GUI: {:?}", e);
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "gui"))]
    {
        eprintln!("=========================================================");
        eprintln!("          Finix Language Engine & Compiler");
        eprintln!("=========================================================");
        eprintln!("To launch the retro Turbo Finix C++ IDE Playground, run:");
        eprintln!("  cargo run --features gui");
        eprintln!("");
        eprintln!("To launch the Language Server Protocol (LSP) server, run:");
        eprintln!("  cargo run -- --lsp");
        eprintln!("=========================================================");
    }
}

#[tokio::main]
async fn run_lsp_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new)
        .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}