var usernameText = document.getElementById("username");
var passwordText = document.getElementById("password");

window.onload = async () => {
    if(localStorage.getItem("token")){
        await fetch("/user", {
            method: "get",
            headers: {
                "Authorization": localStorage.getItem("token") 
            }
        })
        .then((res) => {
            if(res.status == 200) {
                window.location.replace("https://budget.nos-web.dev/home");
            }
            else{
                localStorage.removeItem("token");
            }
        })
    }
}

let register = async() => {
    let name = usernameText.value;
    let pw = passwordText.value;

    let body = JSON.stringify({username: name, password: pw})

    let response = await fetch("/users/register", {
        method: "post",
        headers: {
            "Content-Type": "application/json; charset=UTF-8",
            "Content-Length": body.length
        },
        body: body
    });
    
    await response.json()
    .then((resbody) => {
        handleLogin(response, resbody);
    })
    .catch((why) => {
        alert("invalid credentials!");
    });
}

let login = async() => {
    let name = usernameText.value;
    let pw = passwordText.value;

    let body = JSON.stringify({username: name, password: pw})
    
    let response = await fetch("/users/login", {
        method: "post",
        headers: {
            "Content-Type": "application/json; charset=UTF-8",
            "Content-Length": body.length
        },
        body: body
    });

    await response.json()
    .then((resbody) => {
        handleLogin(response, resbody);
    })
    .catch((why) => {
        alert("invalid credentials!");
    });
}

let handleLogin = async (response, resbody) => {

    if(resbody.token) {
        localStorage.setItem("token", resbody.token);
        document.location.href = response.headers.get("Location");
    }

}