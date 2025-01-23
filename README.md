# A custom webserver built from scratch with Rust

This is a simple web server built from the ground up using Rust. 

Its main goal is to allow for file browsing the host's file system.

The server is designed to demonstrate basic networking concepts, HTTP request handling, and multi-threading in Rust.

## Features

- Serve the file system
- Basic HTTP request/response handling
- Multi-threaded requests handling, capped at 4 threads
- Custom routing and static file serving
- Built with Rust's `std::net`, `std::thread`, and `std::io` libraries

## Prerequisites

Before you can run the web server, you need to have the following:

- Rust installed (via rustup)
- A terminal with access to run commands

## Installation and execution

1. Clone the repo
2. Build abd run the project with ```cargo run --release```
3. Open `http://localhost:7878` on a browser

