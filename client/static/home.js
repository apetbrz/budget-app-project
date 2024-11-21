if(!localStorage.getItem("token")){
    window.location.replace("https://budget.nos-web.dev/");
}

const moneyFormat = new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
});


window.onload = async () => {
    await fetch("/user", {
        method: "get",
        headers: {
            "Authorization": localStorage.getItem("token")
        }
    }).then((res) => {
        if(res.status != 200) {
            localStorage.removeItem("token");
            window.location.replace("https://budget.nos-web.dev/");
        }
        else{
            return res.json();
        }
    }).then((user) => {
        console.log(user);
        updateData(user);
    });

}

let sendCommand = async (body) => {
    let bodyJson = JSON.stringify(body);

    console.log("sending command: " + bodyJson);

    return await fetch("/user", {
        method: "post",
        headers: {
            "Authorization": localStorage.getItem("token"),
            "Content-Type": "application/json",
            "Content-Length": bodyJson.length
        },
        body: bodyJson
    });
}

let updateData = (newdata) => {
    console.log("updating data:")
    console.table(newdata);

    let text = document.getElementById("data");
    let username = document.getElementById("username");
    let income = document.getElementById("income");
    let balance = document.getElementById("balance");
    let savings = document.getElementById("savings");

    let expectedExpenses = document.getElementById("expectedExpenses");
    let currentExpenses = document.getElementById("currentExpenses");
    
    username.textContent = newdata.username;
    income.textContent = "income: " + moneyFormat.format(newdata.expected_income/100);
    balance.textContent = "balance: " + moneyFormat.format(newdata.current_balance/100);
    savings.textContent = "savings: " + moneyFormat.format(newdata.savings/100);
    
    expectedExpenses.textContent = '';
    for(el in newdata.expected_expenses){
        let data = document.createElement("li");
        data.textContent = el + ": " + moneyFormat.format(newdata.expected_expenses[el]/100);
        expectedExpenses.appendChild(data);
    }
    currentExpenses.textContent = '';
    for(el in newdata.current_expenses){
        let data = document.createElement("li");
        data.textContent = el + ": " + moneyFormat.format(newdata.current_expenses[el]/100);
        currentExpenses.appendChild(data);
    }
}

let addNewExpense = async () => {
    let name = document.getElementById("commandtarget").value;
    let amount = document.getElementById("commanddollarvalue").value;

    let body = {
        command: "new",
        label: name,
        amount: amount
    };

    let response = await sendCommand(body)

    if(response.status != 200){
        alert("bad command!");
    }
    else{
        let data = await response.json();

        updateData(data);
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

    let response = await sendCommand(body);

    if(response.status != 200){
        alert("bad command!");
    }
    else{
        let data = await response.json();
        console.log(data);

        updateData(data);
    }
}

let setIncome = async () => {
    let name = document.getElementById("commandtarget").value;
    let amount = document.getElementById("commanddollarvalue").value;

    let body = {
        command: "setincome",
        amount: amount
    };


    let response = await sendCommand(body);

    if(response.status != 200){
        alert("bad command!");
    }
    else{
        let data = await response.json();
        console.log(data);

        updateData(data)
    }
}

let logout = async () => {
    await fetch("/users/logout", {
        method: "post",
        headers: {
            "Authorization": localStorage.getItem("token")
        }
    });
    localStorage.removeItem("token");
    document.location.href = "http://budget.nos-web.dev";
}