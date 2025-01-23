# A custom webserver built from scratch with Rust

This server allows for file browsing the host's file system.

It is a simple web server built from the ground up using Rust. 

The server is designed to demonstrate basic networking concepts, HTTP request handling, and multi-threading in Rust.



## Supported paths

- `/fs`: Shows all the files in a given host's directory
![Screenshot from 2025-01-23 19-20-19](https://github.com/user-attachments/assets/06722674-47ca-42d1-8000-d35907dd428e)
- `/sleep`: Serves a basic static html page after a 5 seconds delay. Useful for demonstrating the multithreaded behavior of the server
- `/`: Welcome page which links to other pages

## Features

- Serve the file system
- Basic HTTP request/response handling
- Multi-threaded requests handling, capped at 4 threads
- Custom routing and static file serving
- Built with Rust's `std::net`, `std::thread`, and `std::io` libraries

## Prerequisites

Before you can run the web server, you need to have the following:

- Rust installed
- A terminal 

## Installation and execution

1. Clone the repo
2. Build and run the project with ```cargo run --release```
3. Open `http://localhost:7878` on a browser

