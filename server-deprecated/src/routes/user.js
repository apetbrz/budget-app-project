import express from 'express';
import db from '../database.js';
import session from '../session.js';

const router = express.Router();

router.get("/", (req, res) => {
    const {id} = req.query.id;
    console.log("user/?id="+id+" reached");

    res.send(JSON.stringify({uuid: id}));
})

export default router