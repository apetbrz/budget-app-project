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
    let text = document.getElementById("data");
    text.textContent = JSON.stringify(user);

}

let addNewExpense = async () => {
    let name = document.getElementById("commandtarget").value;
    let amount = document.getElementById("commanddollarvalue").value;

    let response = await fetch("/user", {
        method: "post",
        headers: {
            "Authorization": sessionStorage.getItem("token")
        },
        body: JSON.stringify({
            command: "new",
            label: name,
            amount: amount
        })
    });

    if(response.status != 200){
        alert("bad command!");
    }
    else{
        let data = await response.json();

        document.getElementById("data").textContent = JSON.stringify(data);
    }
}

let getPaid = async () => {
    let name = document.getElementById("commandtarget").value;
    let amount = document.getElementById("commanddollarvalue").value;

    let body = {
        command: "getpaid"
    }
    if(amount) {
        body.amount = amount;
    }

    let response = await fetch("/user", {
        method: "post",
        headers: {
            "Authorization": sessionStorage.getItem("token")
        },
        body: JSON.stringify(body)
    });

    if(response.status != 200){
        alert("bad command!");
    }
    else{
        let data = await response.json();

        document.getElementById("data").textContent = JSON.stringify(data);
    }
}

let setIncome = async () => {
    let name = document.getElementById("commandtarget").value;
    let amount = document.getElementById("commanddollarvalue").value;

    let body = {
        command: "setincome",
        amount: amount
    }

    let response = await fetch("/user", {
        method: "post",
        headers: {
            "Authorization": sessionStorage.getItem("token")
        },
        body: JSON.stringify(body)
    });

    if(response.status != 200){
        alert("bad command!");
    }
    else{
        let data = await response.json();

        document.getElementById("data").textContent = JSON.stringify(data);
    }
}