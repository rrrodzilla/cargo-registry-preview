use anyhow::{bail, Result};
use cargo_registry_readme_lib::preview;
use clap::Parser;
use std::fs;
use std::net::TcpListener;
use tiny_http::{Header, Response, Server};

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
    println!("🦀 Server started!");
    println!("📦 Preview at http://{}:{}", host, port);
    println!("🔥 Hot reload enabled");

    //open the default browser if the user chooses
    if args.open {
        println!("🌐 Opening browser");
        opener::open_browser(format!("http://{}:{}", host, port))?;
    } else {
        println!("❌ Opening browser (nope)");
    }

    for request in server.incoming_requests() {
        //handle the various requests
        match request.url() {
            //respond with the html page and inject the readme html
            "/" => request.respond(
                Response::from_string(
                    include_str!("../../wireframes/crates/src/index.html")
                        .replace("{}", &preview(&readme)),
                )
                .with_header("Content-Type: text/html".parse::<Header>().unwrap()),
            )?,
            //respond with the minified highlights js file
            "/highlight.min.js" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/highlight.min.js").as_ref(),
                )
                .with_header("Content-Type: text/javascript".parse::<Header>().unwrap()),
            )?,
            //respond with the minified github css file
            //sourced from https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.4.0/styles/github.min.css
            "/github.min.css" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/github.min.css").as_ref(),
                )
                .with_header("Content-Type: text/css".parse::<Header>().unwrap()),
            )?,
            //respond with the minified css file
            "/output.css" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/dist/output.css").as_ref(),
                )
                .with_header("Content-Type: text/css".parse::<Header>().unwrap()),
            )?,
            //respond with Ferris B. Rustacean image file
            //sourced from Rustacean.net
            "/rustacean-flat-happy.png" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/rustacean-flat-happy.png").as_ref(),
                )
                .with_header("Content-Type: image/png".parse::<Header>().unwrap()),
            )?,
            //respond with the downloads image file
            //sourced from crates.io - rand crate (most downloaded all time)
            "/downloads.png" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/download.png").as_ref(),
                )
                .with_header("Content-Type: image/png".parse::<Header>().unwrap()),
            )?,
            //respond with the crates.io favicon
            "/favicon.ico" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/favicon.ico").as_ref(),
                )
                .with_header("Content-Type: image/x-icon".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraSans-Bold.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/FiraSans-Bold.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraSans-Italic.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/FiraSans-Italic.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraSans-Regular.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/FiraSans-Regular.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraMono-Regular.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../../wireframes/crates/src/FiraMono-Regular.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with the logo file
            //sourced from crates.io
            "/logo.png" => request.respond(
                Response::from_data(include_bytes!("../../wireframes/crates/logo.png").as_ref())
                    .with_header("Content-Type: image/png".parse::<Header>().unwrap()),
            )?,
            //respond with the fira typography css file
            "/fira.css" => request.respond(
                Response::from_string(include_str!("../../wireframes/crates/src/fira.css"))
                    .with_header("Content-Type: text/css".parse::<Header>().unwrap()),
            )?,
            //else respond with a 308 to the index page
            _ => request.respond(
                Response::empty(308).with_header("Location: /".parse::<Header>().unwrap()),
            )?,
        };
    }

    Ok(())
}
