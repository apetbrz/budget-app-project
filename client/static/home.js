window.onload = async () => {
    if(!sessionStorage.getItem("token")){
        window.location.replace("https://budget.nos-web.dev/");
    }
    let user = await fetch("/user", {
        method: "get",
        headers: {
            "Authorization": sessionStorage.getItem("token")
        }
    }).then((res) => res.json());

    console.log(user);
    let text = document.createElement("h");
    text.textContent = JSON.stringify(user);
    document.getElementById("body").append(text)
}
