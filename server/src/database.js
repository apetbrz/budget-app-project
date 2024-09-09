import sqlite3 from 'sqlite3';
import {v4 as uuidv4} from 'uuid';

const authTableCreation = "auth(name text NOT NULL, uuid text NOT NULL)"
const authTable = "auth"

let dbPath = import.meta.dirname + "\\..\\db\\users.db"

let usersDatabase = new sqlite3.Database(dbPath, (err) => {
    console.log(dbPath);
    if(err){
        return console.table(err);
    }
    console.log("sqlite database connected")

    //FIXME: THIS CLEARS THE DATABASE ON START UP, FOR DEVELOPMENT:
    usersDatabase.run("DROP TABLE IF EXISTS " + authTable)
    //DONT FORGET ABOUT THIS!!!!

    usersDatabase.run("CREATE TABLE IF NOT EXISTS " + authTableCreation)
})

class User {
    constructor(username) {
        this.username = username;
        this.id = uuidv4();
    }
}

let addUser = (name) => {
    console.log("addUser called: " + name);

    let user = new User(name);

    usersDatabase.run("INSERT INTO " + authTable + " VALUES(?, ?)", user.username, user.id)
    
}

export default { User, addUser }