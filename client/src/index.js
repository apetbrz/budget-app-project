var username = document.getElementById("username");

let submit = async() => {
    let name = username.value;
    
    let response = await fetch("/users", {
        method: "post",
        headers: {
            "Content-Type": "application/json; charset=UTF-8"
        },
        body: JSON.stringify({username: name})
    });

    console.log(response);
}