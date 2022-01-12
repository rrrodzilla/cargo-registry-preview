<h1 align="center">cargo-markdown</h1>

![image](https://github.com/rrrodzilla/cargo-registry-preview/raw/main/assets/cargo_markdown_logo.png)

---

<h3 align="center">Create great readme files for your crates.io page fast!</h3>

A cargo subcommand to preview your crate's readme file within your default browser in a near-pixel-perfect hot reloading mockup of the crates.io website.

## ⚡Quick Start


Make sure you have Rust and Cargo installed, then install with:
```bash
cargo install cargo-markdown
```
From your project directory, run:
```bash
# your readme file must already exist at the path you specify
cargo markdown ./readme.md 
```
<img class="align-center" src="https://github.com/rrrodzilla/cargo-registry-preview/raw/main/assets/demo.gif" alt="An animated image showing the output of cargo-markdown when run from a terminal" align="center" />

Open your browser to the address given and you'll see your readme file in a mockup of the crates.io website.

Now you can edit your readme file in the editor of your choice. Each time you save the file, the preview web page will hot reload immediately.


## 🎯 Features

- Same markdown rendering code and styles as the official crates.io website
- Same responsive breakpoints so you can test the layout in a mobile format
- Ultra fast hot reloading when your readme file is updated
- Test from a [live mobile device](#-testing-on-mobile) using [ngrok](https://ngrok.com)
- Automatically open your default browser by passing the `--open` flag
- Keeps track of how many readme updates you've made each session

## ⚙️ Usage

```bash
USAGE:
    cargo markdown [OPTIONS] <README> [PORT]

ARGS:
    <README>    📄 The path to your readme file
    <PORT>      🚢 The port used by the preview server [default: 8080]

<OPTIONS>
    -h, --help           ❔ Print help information
    -p, --port <PORT>    🚢 The port used by the preview server [default: 8080]
    --host <HOSTNAME>    🏨 The hostname used by the hot-reload server [default: 127.0.0.1]
    -o, --open <open>    🌐 Automagically open your browser on startup [default: false]
    -V, --version        🎬 Print version information
```

### Opening your default browser

Simply pass the `--open` flag to have *cargo-markdown* automatically open your default browser on startup.

Don't pass `--open` if you don't want your browser to automatically open.  You can browse to the displayed url on your own.  This is great for when you already have a tab open to your preview website but you've closed down *cargo-markdown*.  When you re-open *cargo-markdown* any tabs you have open will automatically reconnect within 5 seconds.


## 🧪 Testing on mobile

You can use a tool like [ngrok](https://ngrok.com/) to open a public url to your local preview site and then connect to it on your mobile device (or any other device with internet access).

Once you have ngrok installed on your system, run it like so:
```bash
./ngrok http 8080
```

ngrok will show you a public url to your localhost:8080 address.  ![image](https://github.com/rrrodzilla/cargo-registry-preview/raw/main/assets/ngrok_ui.png)

Now simply run *cargo-markdown* passing in the host from ngrok.
```bash
cargo markdown readme.md --host 62cf2a5454d8.ngrok.io

```

You can now use another machine or a mobile device to access your crates.io mockup on another device complete with hot reloading by browsing to: http://62cf2a5454d8.ngrok.io (or whatever address ngrok gave you)

> **note:** hot reload over ngrock might be slower since the site is reloading over the internet and via a proxy instead of directly from localhost

---

## ❤️ Like this tool?

> ⭐ Star     https://github.com/rrrodzilla/cargo-registry-preview

> 🐦 Follow   https://twitter.com/rrrodzilla



