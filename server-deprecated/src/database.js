import sqlite3 from 'sqlite3';      //using sqlite3 for database
import {v4 as uuidv4} from 'uuid';  //generates unique user ids
import bcrypt from 'bcrypt';        //encryption/hashing for passwords

//format for authentication table
const authTableName = "auth"
const authTableCreation = authTableName + "(name text UNIQUE NOT NULL, passwordhash text NOT NULL, uuid text NOT NULL)"

//how many times salt is added in password encryption
const saltRounds = 10;

//database is stored in server/db/
let dbPath = import.meta.dirname + "\\..\\db\\users.db"

//create/connect to the database file
let usersDatabase = new sqlite3.Database(dbPath, (err) => {
    console.log("looking for database in " + dbPath);

    if(err){
        return console.table(err);
    }

    console.log("sqlite database connected")

    //FIXME: THIS CLEARS THE DATABASE ON START UP, FOR DEVELOPMENT:
    usersDatabase.run("DROP TABLE IF EXISTS " + authTableName)
    //DONT FORGET ABOUT THIS!!!!

    //create auth table
    usersDatabase.run("CREATE TABLE IF NOT EXISTS " + authTableCreation)
})

//User class: defines how a User is stored in memory
class User {
    constructor(username, passwordHashed) {
        this.username = username;
        this.id = uuidv4();
        this.passwordhash = passwordHashed;
    }
}

let addUser = async(name, pw) => {
    console.log("addUser called: " + name);

    let hashedPassword = await bcrypt.genSalt(saltRounds).then(salt => {
        return bcrypt.hash(pw, salt);
    }).then(hash => {
        return hash;
    }).catch(err => {
        console.error(err.message);
        return null;
    });

    let user = new User(name, hashedPassword);

    return new Promise((res, rej) => {
        usersDatabase.run("INSERT INTO " + authTableName + " VALUES(?, ?, ?)", [user.username, user.passwordhash, user.id], (err) => {
            console.log("db query callback called: " + err);
            if(err) rej(err.message);
            else res("success");
        })
    })
}

//returns a promise, which resolves to the entire auth table row for the user
//if error, uuid is null, if valid, err is null
let authenticateAndGetAuthRow = (name, pw) => {
    let query = "SELECT * FROM " + authTableName + " WHERE name = ?" 

    return new Promise((res, rej) => {
        usersDatabase.get(query, [name], (err, row) => {
            if(err){
                console.error(err.message);
                rej(err.message);
            }
            else{
                bcrypt.compare(pw, row.passwordhash, (err, valid) => {
                    if(err) rej(err.message);
                    else if(valid){
                        res(row);
                    }
                    else{
                        rej("invalid-credentials");
                    }
                })
            }
        })
    })
}

//authenticates and returns UUID
let authenticate = async(name, pw) => {
    let row = await authenticateAndGetAuthRow(name, pw);

    console.table(row);

    return row.uuid;
}

//FIXME: displaying all is unsafe, only for dev purposes!
let getAllUsers = () => {
    let query = "SELECT * FROM " + authTableName;
    
    return new Promise((res,rej) => {
        usersDatabase.all(query, [], (err, rows) => {
            if(err){
                console.error(err.message);
                rej(err.message);
            }
            else res(rows);
        })
    })
}

export default { User, addUser, authenticate, authenticateAndGetAuthRow, getAllUsers }