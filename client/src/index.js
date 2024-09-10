var usernameText = document.getElementById("username");
var passwordText = document.getElementById("password")

let register = async() => {
    let name = usernameText.value;
    let pw = passwordText.value;
    
    let response = await fetch("/users/create", {
        method: "post",
        headers: {
            "Content-Type": "application/json; charset=UTF-8"
        },
        body: JSON.stringify({username: name, password: pw})
    }).then((res) => res.json());

    console.table(response);
}

let login = async() => {
    let name = usernameText.value;
    let pw = passwordText.value;
    
    let response = await fetch("/users/login", {
        method: "post",
        headers: {
            "Content-Type": "application/json; charset=UTF-8"
        },
        body: JSON.stringify({username: name, password: pw})
    }).then((res) => res.json());

    console.table(response);
}