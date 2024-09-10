import dotenv from "dotenv/config"
import db from './database.js';
import WebSocket from 'ws';

class Session{
    constructor(uuid){
        let wsUrl = "ws://localhost:"+process.env.PORT+"/user/?id="+uuid;
        
        this.uuid = uuid;
        this.socket = new WebSocket(wsUrl);
        

        this.socket.on("error", console.error);

        this.socket.on("open", () => {
            this.socket.send("hello world!");
        });

        this.socket.on("message", data => {
            console.table(data);
        })
    }
}


export default { Session }