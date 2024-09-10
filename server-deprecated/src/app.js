import express from 'express';  //this API makes HTTP so much easier and nicer to use
import dotenv from 'dotenv/config'  //this lets me put settings in the server/.env file

//these are the route handlers i've written:
import indexRoutes from './routes/index.js';  //for the landing page(s)
import userAuthRoutes from './routes/users.js';   //for user account pages/info
import userRoutes from './routes/user.js'     //for user data

//this determines how much console printing the code does.
const DEV_LOGGING = process.env.DEV_LOGGING == "true";

//create the server app, and select port
const app = express();
const port = process.env.PORT;

//enable json messages
app.use(express.json());

//middleware to enable logging of each and every request
app.use((req, res, next) => {
  //if no logging, nothing happens
  if(DEV_LOGGING){
    //prints the time down to milliseconds, and the method + url
    let d = new Date();
    console.log(`
      request at ${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}:${String(d.getSeconds()).padStart(2,"0")}.${String(d.getMilliseconds()).padStart(3,"0")}: ${req.method} -> ${req.url}`);
    console.table(req.body);
  }
  //continue to route handler
  next()
})

//use userAuthRoutes for all /users/ URLs
app.use('/users', userAuthRoutes);

//use userRoutes for all /user/ URLs
app.use('/user', userRoutes);

//use indexRoutes for every other URL
app.use('/', indexRoutes);

//^ these work like: 

app.listen(port, () => {
  console.log(` 
Listening on port ${port}.
Connect to 'localhost:${port}' to see.
Press ctrl+c to exit.
`)
})