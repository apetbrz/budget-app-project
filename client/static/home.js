document.onload(() => {
    if(!sessionStorage.getItem("token")){
        window.location.replace("https://budget.nos-web.dev/");
    }
})
/*
let response = await fetch("/user/{some user token lol}", {
    method: "get",
}).then((res) => res.json());

let user =  {
    username: "something",
    current_balance: 0,
    expected_income: 0,
    expected_expenses: { rent: 100 },
    current_expenses: { rent: 0 },
    savings: 0,
}

let body = {
    command: "add_expense",
    name: "rent",
    value: 100
}

let response = await fetch("/app", {
    method: "post",
    headers: {
        "Content-Type": "application/json; charset=UTF-8",
        "Content-Length": body.length
    },
    body: body
})
*/