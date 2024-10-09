document.onload(() => {
    if(!sessionStorage.getItem("token")){
        window.location.replace("https://budget.nos-web.dev/");
    }
})

