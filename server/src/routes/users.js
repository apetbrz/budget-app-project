import express from 'express';
import db from '../data/database.js';
const router = express.Router();

var users = [
    new db.User("test"),
    new db.User("test2")
]

router.get("/", (req, res) => {
    res.send(users);
})

router.post("/", (req, res) => {
    console.log(req.body);
    res.send("ty");
    db.addUser(req.body.username);
    users.push(new db.User(req.body.username))
})

export default router