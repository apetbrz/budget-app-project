import express from 'express';
import db from '../database.js';
import session from '../session.js';

const router = express.Router();

router.get("/", (req, res) => {
    db.getAllUsers().then(obj => {
        console.table(obj);
        res.send(JSON.stringify(obj));
    }).catch(err => {
        res.send("server-error");
    })
})

router.post("/create/", (req, res) => {
    console.log(req.body);
    db.addUser(req.body.username, req.body.password)
    .then(msg => {
        console.log(msg);
        res.send(JSON.stringify({message: msg}));
    })
    .catch(err => {
        res.statusCode(409).send(JSON.stringify(err));
    });
})

router.post("/login/", (req, res) => {
    console.log(req.body);
    db.authenticate(req.body.username, req.body.password)
    .then(uuid => {

        //CODE HERE RUNS WHEN THE USER LOGIN IS ACCEPTED
        //TODO: webtoken
        //TODO: create websocket to process user interaction
        const socket = new session.Session(uuid);

        res.send(socket.url);


    }).catch(err => {
        console.error(err);
        res.send(err.message);
    });
})



export default router