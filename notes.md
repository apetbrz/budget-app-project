# dev notes

### 9/4/24:

Today I began work on the codebase. I've dabbled in server code a little bit before, but I'm by no means very experienced. I'm still relearning as I go. To begin, I'm only writing a 'CRUD' API in JavaScript. CRUD refers to when you can create/read/update/delete user data in a database. It's a way to say that I'm only writing the code to register/login/access/edit/delete/etc. user accounts. The barebones CRUD API will be what I will benchmark to compare performance, once I get the Pi. If other languages are significantly faster than JS they may be much nicer to write in.

### 9/9/24: 

Today I continued some work on the barebones CRUD API in JS. Learning JavaScript has been tricky, with all its async functionality and promises and whatnot. I'm particularly having troubles sending messages to the client, it seems like I'm sending Promise objects instead of messages??? I'm not sure. I've gotten registration working, where you can send your username/password to the server and it'll hash your password and store it in the database, but I'm struggling to authenticate when you want to login. I'm using Bcrypt for encryption, it makes it very easy. The hashing is done server-side. For extra security, I plan on getting an HTTPS signature done later on, but for now, passwords are sent as plain text.

After some work, I finally got past the issues I was having today. Promises are annoying. But once you figure them out, it makes sense. I'm a big fan of JS' `console.table()`, it prints objects as a nice neat table in the console. Super useful.

### 9/10/24:

Realizing that just programming with no solid plan makes for very chaotic and inconsistent code. Today, I'm writing up some standards for how I plan on structuring data, including function return values and how messages are communicated across the entire stack. JSDoc is handy for this, it's like Typescript-lite, using IDE hover/autofill features to ensure consistency.

While JavaScript is tremendously helpful for learning how backend server code operates, it unfortunately has one issue: it is single-threaded. While possible to hack together a way to create other threads, it's limited to my physical CPU core count, as JS really can only be single-threaded per instance. So, while my CRUD API is not complete in JS, I'm going to stop work on it. Limiting how many threads I can use at a time is too debilitating for what I intended to be a highly multithreaded web app. Therefor, today marks me beginning work on a backend API written in Rust.

Rust, unlike JS, is compiled down to machine code, and has *incredible* support for multithreading. I look forward to seeing how well it works!