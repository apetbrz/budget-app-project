var usernameText = document.getElementById("username");
var passwordText = document.getElementById("password");

let register = async() => {
    let name = usernameText.value;
    let pw = passwordText.value;

    let body = JSON.stringify({username: name, password: pw})

    console.log("SENDING THE REQUEST!!! BODYSIZE=" + body.length);
    
    let response = await fetch("/users/register", {
        method: "post",
        headers: {
            "Content-Type": "application/json; charset=UTF-8",
            "Content-Length": body.length
        },
        body: body
    });

    await handleLogin(response);
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

    await handleLogin(response);
}

let handleLogin = async (response) => {

    

    let response_body = await response.json();

    if(response_body.token) {
        sessionStorage.setItem("token", response_body.token);
        document.location.href = response.headers.get("Location");
    }

}