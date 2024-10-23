# 511 Group Project

### by Arthur Petroff and Tyler Brown

---

This is a personal budgeting web app to allow users to track their earnings, expenses, and savings.

---

### Setup

1. Install Rust, from `https://rustup.rs/`

2. Run once by running `cargo run` within the `server/` folder, to generate preset environment variables. Restart.

3. Open the `server/.env` file and add a `SECRET` value.

### Running

 - Run with `cargo run` in the `server/` folder.

---

### Notes

Custom multithreaded implementation, avoiding async/await, to familiarize with concurrent threading code. Highly in-development.