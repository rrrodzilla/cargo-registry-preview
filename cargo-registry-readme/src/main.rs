use anyhow::{bail, Result};
use cargo_registry_readme_lib::preview;
use clap::Parser;
use std::fs;
use std::io::Cursor;
use std::net::TcpListener;
use tiny_http::{Header, Response, Server};

const CRATES_PATH: &str = "/crates";
const LIBRS_PATH: &str = "/librs";

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// 📄 Readme file (e.g. - "./readme.md")
    #[clap()]
    readme: String,
    /// 🌐 Automatically open the preview in your default browser
    #[clap(short, long)]
    open: bool,
}

fn default_response() -> Response<Cursor<Vec<u8>>> {
    let preview_html = "<html><head><link rel='stylesheet' href='https://code.cdn.mozilla.net/fonts/fira.css'></head><body>Nothing here</body></html>";
    let mut response = Response::from_string(preview_html);
    response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
    response
}

fn librs_preview_response(readme: &str) -> Response<Cursor<Vec<u8>>> {
    let preview_html = format!("<html><head><link rel='stylesheet' href='https://code.cdn.mozilla.net/fonts/fira.css'></head><body>{}</body></html>", preview(readme));
    let mut response = Response::from_string(preview_html);
    response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
    response
}

fn crates_preview_response(readme: &str) -> Response<Cursor<Vec<u8>>> {
    let preview_html = format!("<html><head><link rel='stylesheet' href='https://code.cdn.mozilla.net/fonts/fira.css'></head><body>{}</body></html>", preview(readme));
    let mut response = Response::from_string(preview_html);
    response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
    response
}

fn main() -> Result<()> {
    let args = Args::parse();

    //verify the readme file exists
    let readme = match fs::read_to_string(&args.readme) {
        Ok(s) => s,
        Err(_) => bail!("🛑 Couldn't 🔍find a readme 📄 at {}", args.readme),
    };

    //let the OS pick an available port number
    const ADDRESS: &str = "127.0.0.1:0";

    let listener = TcpListener::bind(ADDRESS)?;
    let host = listener.local_addr()?.ip().to_string();
    let port = listener.local_addr()?.port();

    let server = Server::from_listener(listener, None).unwrap();
    println!("Preview readme 🦀🔎:");
    println!("  📦crates.io: http://{}:{}{}", host, port, CRATES_PATH);
    println!("  📦lib.rs: http://{}:{}{}", host, port, LIBRS_PATH);

    //open the default browser if the user chooses
    if args.open {
        opener::open_browser(format!("http://{}:{}{}", host, port, CRATES_PATH))?;
        opener::open_browser(format!("http://{}:{}{}", host, port, LIBRS_PATH))?;
    }

    for request in server.incoming_requests() {
        //      println!(
        //          "received request! method: {:?}, url: {:?}, headers: {:?}",
        //          request.method(),
        //          request.url(),
        //          request.headers()
        //      );
        let response = match request.url() {
            CRATES_PATH => crates_preview_response(&readme),
            LIBRS_PATH => librs_preview_response(&readme),
            _ => default_response(),
        };
        request.respond(response)?;
    }

    Ok(())
}
