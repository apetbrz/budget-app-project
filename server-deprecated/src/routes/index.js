import express from 'express';
import path from 'path';
const router = express.Router();


router.get('/', (req, res) => {
    res.send('Hello World!')
})

router.get('/home', (req, res) => {
    let filepath = path.join(import.meta.dirname, "../../../client/src/index.html")
    res.sendFile(filepath, (err) => {
        if(err) console.error(err.message);
    })
})

router.get('/file/:file', (req, res) => {
    const {file} = req.params;
    while(file.startsWith("../")){
        file = file.substring(3)
    }
    let filepath = path.join(import.meta.dirname, "../../../client/src/", file);
    res.sendFile(filepath, (err) => {
        if(err) console.error(err.message);
    })
})

export default router