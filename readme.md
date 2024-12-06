# 511 Group Project

### by Arthur Petroff and Tyler Brown

---

This is a personal budgeting web app to allow users to track their earnings, expenses, and savings. It consists of a custom multithreaded HTTP server implementation, serving raw HTML/CSS/JS, and performing app functionality entirely server-side.

---

### Usage

These instructions are subject to change, as the program is made more portable and standalone. For the time being, `cargo run` is recommanded for building + executing the current in-development version.

1. Install Rust, from `https://rustup.rs/`

2. In a shell, navigate into `server/` and run `cargo run` to build and execute.

Note: The generated .env file contains a not-very-secure secret string, please replace it, should security matter to you. Re-run `cargo build` after changing any .env variables. The server defaults to port 3000.

---


