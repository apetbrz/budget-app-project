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

### 9/12/24:

Researching into Rust server ecosystems has led me to countless libraries or packages or whatever else, to handle most of the hard work for me. Unfortunately for those, I wanted to really get down to the nitty gritty processing of HTTP requests/responses. I'm gonna be using bare-minimum dependencies, only the ones that speed up menial tasks (like actually parsing from strings to objects (for HTTP requests/responses, json, etc.)) or do things I can't do manually (like encryption).

### 9/17/24:

Last week I bought the hardware for the project, and spent the weekend getting it set up through Cloudflare. It worked well! I got a domain name, linked it all up, and now anyone can connect to a server running on my Raspberry Pi through a URL (currently `budget.nos-web.dev`)! I've also got simple enough HTTP server code written in Rust, I can successfully take in requests and create responses. I still need to handle things like file access and connecting to a database, but that's what I'm planning on getting to further this week. Going forward, I'm planning on writing many helper functions to make development much easier, for setting up things like routes and middleware. I'm enjoying writing all this from scratch, instead of just using some pre-made package for server code.

### 9/18/24:

I began working on a system to route users, from the path in their HTTP request. I wanted to make it modular, so setting up new pages could be done easily. It's based on a HashMap, mapping from (String, String) tuples to function pointers. lol. I'm still iterating on better ways to store the key, but having the values in the map be function pointers makes it really easy to write code to handle different routes (URLs)!

### 9/19/24

The routing system is the most complicated part by far, so far. I'm working on cleaning it up and making it more structured and organized using different function calls instead of passing everything through one single central huge big annoying weird "String, String tuple to Function Pointer" map. Splitting things up lets me have different files/folders for the different "areas" of the webserver, like user-auth handling routes and file accessing routes.

### 9/23/24

Continued work on the routing system. Implementing a tree-like structure instead of flat maps. It mostly works, but I'm still polishing the structure so that functions can take in both the path itself and any potential http POST request body data as parameters. I'll finish this up soon. Afterwards, it's on to setting up the database!!!! I'll be honest, using a framework would've probably been easier (and more performant, because async/await structure is more efficient than just raw multithreading with infinite threads), but I wanted to manually make parallelized code, instead of letting some framework black box do everything for me.

### 10/1/24

Oops, forgot to update for a while! To be honest, I've been tinkering with the Pi some more, getting used to developing in the Linux terminal (with like, a bunch of plugins) so keeping things organized and up to date has been lower in my mind's priority. The routing system is mostly feature complete, it works how it should, and is easy enough to develop on top of, meaning I can mostly set that section behind me. I've still got to organize folders and stuff, and make sure everything is adequately commented, but that'll come. I've also began database integration, already having account creation (+ uuid generation) and user authentication (+ token generation) functional! I still have a ways to go to make it complete, like password security rules (server AND clientside!), streamlined registration -> login redirection, account information and settings (and tools like deletion), but functionality is a big step!

All that is just for ensuring the CRUD API side of things. As for the server itself, I've already made it multithreaded! Since database interactions are blocking, and I don't want the server to hang (and I'm *not* using async/await), all user authentication is handled in a separate thread. When the server receives a login/registration request (after parsing and routing), it shoots the request body and TCP stream straight to the auth thread, which handles everything else (JSON parsing, database insertions/queries). This is also how actual logged-in user interaction will be handled, every budget app related request will include the logged-in user's jsonwebtoken (a neat little encrypted key, given at login time, with an expiration time), which will allow the main server thread to shoot the request itself to be handled by that user's thread (also created at login time). Doing things this way, the main HTTP handling shouldn't ever hang if any one user demands some huge expensive operation.

I'm really excited to get this all working, and I'm especially looking forward to doing my best to take lots and lots of metrics to display in a T(erminal)UI (if I have the time)! I'm quickly approaching the point where I'll need a - at least basic skeleton - frontend, to enable further testing. My one tiny little pile of register/login javascript can only do so much.

### 10/9/24

I've spent some time separately making the budget app. I made it separate with a little command line interface to make it easier to develop, so all I have to do is port it over to the server backend. I've also started work on creating threads for user connections. I've got little diagrams written in a notebook for how the architecture will roughly look, where the main request parsing is single threaded, but requests are passed to other threads for things like user authentication (in one thread) and all logged-in user commands (through another thread as an "owner" of several individual-user-specific threads). I'm confident that this is a horribly unoptimized approach and I'd get much better performance throwing it all into a generic async/await system, but that sorta turns threading into this magic black box of performance wizardry, as opposed to doing it myself.

So far I've gotten to the point where logging in generates a token and stores it in the client's session storage, then redirects to /home. My minimal testing frontend is starting to not be enough anymore, and I'm going to need to get the front end layout designed up and built, at least to the point of functionality. I'm still working on hooking up user tokens to individual threads, they still don't spawn upon token creation yet.

I may need to look into performance soon. Maybe I'm being wasteful with database operations, I'm not sure yet. What I do know is that one register request took 600ms on the authentication thread, which can definitely be noticeable. Fortunately, most other HTTP requests so far fit comfortably under 1ms, with the worst being no greater than 2ms. I think that's great for now.

### 10/16/24

I've got the threads hooked up to account login/registration!!! It's so nice to see it work, where when you log in it spawns a new thread that loads your budget data from the database into memory, and can send it to you when needed! This is really coming together!

### 10/22/24

I put some timers all over the place, to log how long each thread takes. I've still got to figure out how to time the full incoming->outgoing message latency, but that'll come soon. I need to finish hooking up user commands to use budgeting functions on the loaded user data. After that, I think I can call the project "functional" lol.