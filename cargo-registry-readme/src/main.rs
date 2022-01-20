use anyhow::{bail, Result};
use cargo_registry_markdown::text_to_html;
use clap::{App, AppSettings, Arg};
use console::{style, Color, Emoji, Term};
use hotwatch::{Event, Hotwatch};
use rustc_serialize::base64::{Config, Newline, Standard, ToBase64};
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use tiny_http::{Header, ReadWrite, Request, Response, Server, StatusCode};

//let the OS pick an available port number
const ADDRESS: &str = "127.0.0.1:0";

/// Turns a Sec-WebSocket-Key into a Sec-WebSocket-Accept.
fn convert_key(input: &str) -> String {
    use sha1::Sha1;

    let mut input = input.to_string().into_bytes();
    let mut bytes = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
        .to_string()
        .into_bytes();
    input.append(&mut bytes);

    let mut sha1 = Sha1::new();
    sha1.update(&input);

    sha1.digest().bytes().to_base64(Config {
        char_set: Standard,
        pad: true,
        line_length: None,
        newline: Newline::LF,
    })
}

// a threadsafe sharable websocket stream
struct WsStream(Option<Box<dyn ReadWrite + Send>>);

fn spawn_websockets(request: Request, stream: Arc<Mutex<WsStream>>) -> Result<()> {
    // we are handling this websocket connection in a new task
    let child = spawn(move || {
        // checking the "Upgrade" header to check that it is a websocket
        request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Upgrade"))
            .and_then(|hdr| {
                if hdr.value == "websocket" {
                    Some(hdr)
                } else {
                    None
                }
            });

        // getting the value of Sec-WebSocket-Key
        let key = match request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Sec-WebSocket-Key"))
            .map(|h| h.value.clone())
        {
            None => {
                let response = Response::new_empty(StatusCode(400));
                request.respond(response).expect("Responded");
                return;
            }
            Some(k) => k,
        };

        // building the "101 Switching Protocols" response
        let response = Response::new_empty(StatusCode(101))
            .with_header("Upgrade: websocket".parse::<Header>().unwrap())
            .with_header("Connection: Upgrade".parse::<Header>().unwrap())
            .with_header(
                "Sec-WebSocket-Protocol: hot_reload"
                    .parse::<Header>()
                    .unwrap(),
            )
            .with_header(
                format!("Sec-WebSocket-Accept: {}", convert_key(key.as_str()))
                    .parse::<Header>()
                    .unwrap(),
            );

        //
        let mut guard = stream.lock().unwrap();
        guard.0 = Some(request.upgrade("websocket", response));
    });
    child.join().unwrap();
    Ok(())
}

fn main() -> Result<()> {
    let mut modification_count = 0;
    let running = Arc::new(AtomicBool::new(true));

    let args = App::new("cargo-markdown")
        .version("0.1")
        .bin_name("cargo")
        .about("Rapid readme development for crates.io")
        .version("v0.1.0")
        .setting(AppSettings::PropagateVersion)
        .setting(AppSettings::TrailingVarArg)
        .subcommand(
            App::new("markdown")
                .override_help(
                    format!(
                        "{} v0.1.0

    Preview your crate readme file within your default browser in a near-pixel-perfect 
    hot reloading mockup of the crates.io crate info page for rapid readme development 
    before publishing to staging.crates.io or crates.io

{}
    cargo markdown [OPTIONS] <README> 

{}
    {}    📄 The path to your readme file

{}
    {}❔ Print help information
    {}🚢 The port used by the preview server [default: 8080]
    {}🏨 The hostname used by the hot-reload server [default: 127.0.0.1]
    {}🌐 Automagically open your browser on startup [default: false]
    {}🎬 Print version information

{} ⭐ Star     {}
                🐦 Follow   {}
",
                        style("cargo-markdown").fg(Color::Color256(2)),
                        style("USAGE:").fg(Color::Color256(3)),
                        style("ARGS:").fg(Color::Color256(3)),
                        style("<README>").fg(Color::Color256(2)),
                        style("OPTIONS:").fg(Color::Color256(3)),
                        style("-h, --help           ").fg(Color::Color256(2)),
                        style("-p, --port <PORT>    ").fg(Color::Color256(2)),
                        style("--host <HOSTNAME>    ").fg(Color::Color256(2)),
                        style("-o, --open <open>    ").fg(Color::Color256(2)),
                        style("-V, --version        ").fg(Color::Color256(2)),
                        style("LIKE THIS TOOL?").fg(Color::Color256(3)),
                        style("https://github.com/rrrodzilla/cargo-registry-preview")
                            .fg(Color::Color256(4))
                            .underlined(),
                        style("https://twitter.com/rrrodzilla")
                            .fg(Color::Color256(4))
                            .underlined(),
                    )
                    .as_str(),
                )
                .arg(Arg::new("README").required(true))
                .arg(Arg::new("HOSTNAME").default_value("127.0.0.1").long("host"))
                .arg(
                    Arg::new("PORT")
                        .default_value("8080")
                        .short('p')
                        .long("port"),
                )
                .arg(Arg::new("open").takes_value(false).short('o').long("open")),
        )
        .get_matches();

    //get arguments
    let readme = args
        .subcommand_matches("markdown")
        .unwrap()
        .value_of("README")
        .unwrap();
    let hostname = args
        .subcommand_matches("markdown")
        .unwrap()
        .value_of("HOSTNAME")
        .unwrap();
    let port: usize = args
        .subcommand_matches("markdown")
        .unwrap()
        .value_of_t("PORT")
        .unwrap();
    let open = args
        .subcommand_matches("markdown")
        .unwrap()
        .is_present("open");

    //set up the console
    let mut term = Term::stdout();
    term.set_title("cargo-markdown");
    term.hide_cursor()?;
    term.clear_screen()?;
    term.write_line(
        format!(
            "{} by {} (Ctrl-C to quit)\n",
            &style(format!("{} cargo-markdown", Emoji("🦀", ""),)).fg(Color::Color256(166)),
            &style("@rrrodzilla").fg(Color::Color256(166))
        )
        .as_str(),
    )?;

    //verify the readme file exists
    match fs::read_to_string(&readme) {
        Ok(_) => {
            //let user know we're watching their file
            term.write_line(&format!(
                "{} {} {}",
                Emoji("🟢", ""),
                style("found readme at").fg(Color::Color256(2)),
                style(readme).fg(Color::Color256(2))
            ))?;
        }
        Err(_) => {
            term.show_cursor().unwrap();
            bail!(
                "⚫ {} {}",
                style("readme not found at").fg(Color::Color256(214)),
                readme
            )
        }
    };

    //threadsafe container for websocket stream
    let ws_stream = Arc::new(Mutex::new(WsStream(None)));

    //start watching readme
    let mut hotwatch = Hotwatch::new()?;
    let ws_stream_cloned = ws_stream.clone();
    let mut term_c = term.clone();

    hotwatch.watch(readme, move |event: Event| {
        if let Event::NoticeWrite(_path) = event {
            modification_count += 1;
            //send a hot_reload message to the browser so it can reload
            let mut guard = ws_stream_cloned.lock().unwrap();
            let stream = guard.0.as_mut();
            if stream.is_none() {
                return;
            };
            let stream: &mut Box<dyn ReadWrite + Send> = guard.0.as_mut().unwrap();
            let data = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f];
            stream.write(&data).ok();
            stream.flush().ok();

            //write to the console
            term_c.move_cursor_to(1, 7).ok();
            term_c.clear_line().ok();
            term_c
                .write(
                    format!(
                        "{} {}",
                        Emoji("💚", ""),
                        style(format!("readme updates => {}", modification_count))
                            .fg(Color::Color256(2))
                    )
                    .as_bytes(),
                )
                .ok();
        }
    })?;

    //let user know we're watching their file
    term.write_line(&format!(
        "{} {}...",
        Emoji("🟢", ""),
        style("awaiting updates").fg(Color::Color256(2))
    ))?;

    //set up a tcp listener depending on whether a port was provided
    let listener = match port {
        p if p > 0 => TcpListener::bind(format!("127.0.0.1:{}", port))?,
        _ => TcpListener::bind(ADDRESS)?,
    };

    let host = listener.local_addr()?.ip().to_string();
    let port = listener.local_addr()?.port();

    let server = Server::from_listener(listener, None)
        .ok()
        .map(|server| {
            term.write_line(&format!(
                "{} {}{}",
                Emoji("🟢", ""),
                format!(
                    "{}",
                    style(format!(" {}crates.io (mockup) ", Emoji("📦", "")))
                        .bg(Color::Color256(22))
                        .fg(Color::White),
                ),
                format!(
                    " {}",
                    &style(&format!("http://{}:{}", host, port))
                        .fg(Color::Color256(2))
                        .underlined()
                        .to_string(),
                )
            ))
            .unwrap();

            server
        })
        .unwrap();

    //sping up the server
    let server = Arc::new(server);

    //open the default browser if the user chose to do so
    if open {
        term.write_line(&format!(
            "{} {}...",
            Emoji("🟢", ""),
            style("automagically opening browser").fg(Color::Color256(2))
        ))
        .unwrap();
        opener::open_browser(format!("http://{}:{}", host, port))?;
    } else {
        //otherwise write that they did not choose to do so
        term.write_line(&format!(
            "{} {}",
            Emoji("⚫", ""),
            style("automagic browser opener").fg(Color::Color256(244))
        ))
        .unwrap();
    }

    //handle ctrl-c
    let s = server.clone();
    let t = term.clone();
    ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
        t.clear_screen().unwrap();
        t.clone().show_cursor().unwrap();
        t.move_cursor_to(0, 0).unwrap();
        t.write_line(
            &style(format!("{} nice work rustacean!", Emoji("🦀", ""),))
                .fg(Color::Color256(166))
                .to_string(),
        )
        .unwrap();
        s.unblock();
        drop(s.clone());
    })
    .expect("Error setting Ctrl-C handler");

    //only do this while we're running (no ctrl-c detected)
    for request in server.incoming_requests() {
        //if the request is a web socket request, upgrade the connection
        //in a separate thread and get back the stream for further processing
        if let "/ws" = request.url() {
            spawn_websockets(request, ws_stream.clone())?;

            //let the user know we're ready to hot reload
            term.move_cursor_to(1, 6)?;
            term.clear_line()?;
            term.write_all(
                format!(
                    "{} {}",
                    Emoji("🔥", ""),
                    style("hot reload standing by...").fg(Color::Color256(166))
                )
                .as_bytes(),
            )?;

            continue;
        };

        //if we have a different hostname, we're probably using ngrok so use port 80 for web
        //sockets
        let mut port = port;
        if !hostname.eq("127.0.0.1") {
            port = 80
        }

        //handle the various requests
        match request.url() {
            //respond with the html page and inject the readme html
            "/" => request.respond(
                Response::from_string(
                    include_str!("../assets/crates/src/index.html")
                        .replace(
                            "{}",
                            &text_to_html(&fs::read_to_string(readme)?, "readme", None, None),
                        )
                        .replace("{hot_reload_port}", &port.to_string()),
                )
                .with_header("Content-Type: text/html".parse::<Header>().unwrap()),
            )?,
            //respond with the minified highlights js file
            "/ws.js" => request.respond(
                Response::from_string(
                    include_str!("../assets/crates/src/ws.js")
                        .replace("{hot_reload_port}", &port.to_string())
                        .replace("{hot_reload_host}", hostname),
                )
                .with_header("Content-Type: text/javascript".parse::<Header>().unwrap()),
            )?,
            //respond with the minified highlights js file
            "/highlight.min.js" => request.respond(
                Response::from_data(
                    include_bytes!("../assets/crates/src/highlight.min.js").as_ref(),
                )
                .with_header("Content-Type: text/javascript".parse::<Header>().unwrap()),
            )?,
            //respond with the minified github css file
            //sourced from https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.4.0/styles/github.min.css
            "/github.min.css" => request.respond(
                Response::from_data(include_bytes!("../assets/crates/src/github.min.css").as_ref())
                    .with_header("Content-Type: text/css".parse::<Header>().unwrap()),
            )?,
            //respond with the minified css file
            "/output.css" => request.respond(
                Response::from_data(include_bytes!("../assets/crates/out/output.css").as_ref())
                    .with_header("Content-Type: text/css".parse::<Header>().unwrap()),
            )?,
            //respond with Ferris B. Rustacean image file
            //sourced from Rustacean.net
            "/rustacean-flat-happy.png" => request.respond(
                Response::from_data(
                    include_bytes!("../assets/crates/rustacean-flat-happy.png").as_ref(),
                )
                .with_header("Content-Type: image/png".parse::<Header>().unwrap()),
            )?,
            //respond with the downloads image file
            //sourced from crates.io - rand crate (most downloaded all time)
            "/downloads.png" => request.respond(
                Response::from_data(include_bytes!("../assets/crates/download.png").as_ref())
                    .with_header("Content-Type: image/png".parse::<Header>().unwrap()),
            )?,
            //respond with the crates.io favicon
            "/favicon.ico" => request.respond(
                Response::from_data(include_bytes!("../assets/crates/src/favicon.ico").as_ref())
                    .with_header("Content-Type: image/x-icon".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraSans-Bold.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../assets/crates/src/FiraSans-Bold.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraSans-Italic.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../assets/crates/src/FiraSans-Italic.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraSans-Regular.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../assets/crates/src/FiraSans-Regular.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with a required font
            "/woff/FiraMono-Regular.woff" => request.respond(
                Response::from_data(
                    include_bytes!("../assets/crates/src/FiraMono-Regular.woff").as_ref(),
                )
                .with_header("Content-Type: font/woff".parse::<Header>().unwrap()),
            )?,
            //respond with the logo file
            //sourced from crates.io
            "/logo.png" => request.respond(
                Response::from_data(include_bytes!("../assets/crates/logo.png").as_ref())
                    .with_header("Content-Type: image/png".parse::<Header>().unwrap()),
            )?,
            //respond with the fira typography css file
            "/fira.css" => request.respond(
                Response::from_string(include_str!("../assets/crates/src/fira.css"))
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
