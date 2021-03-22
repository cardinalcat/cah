var isopen = false;
var socket = new WebSocket('ws://10.0.0.206:3012');
var username = "";
var gamecode = "0";
var submited = false;
var isjudge = false;
var judgecards = [username];
function startup(){
    socket = new WebSocket('ws://10.0.0.206:3012');
}
//causes problems if someone joins after card submited
//var message = "";
//var last_message = "";
function changeimage(){
    let images = ["cah1.jpeg", "cah2.jpeg", "cah3.png"];
    let i = Math.round(Math.random() * 1000) % 3;
    document.body.style.backgroundImage = "url(images/" + images[i] + ")";
}
socket.addEventListener('open', function (event) {
    isopen = true;
    //message = event.data;
    changeimage();
});
socket.addEventListener('message', function (event) {
    console.log('Message from server', event.data);
    let content = JSON.parse(event.data);
    /// automatically done
    /*if(content.task == "StartGame"){
        gamecode = content.data;
        let CreateUser = {
            gameid: gamecode,
            username: username,
            kind: "Admin",
            task: "CreateUser",
            data: username,
        }
        socket.send(JSON.stringify(CreateUser));
        window.alert("game id: " + gamecode);
    }*/
    if(content.task == "SendMessage"){
        // do something with the message area
    }
    if(content.task == "CreateUser"){
        let drawcard = {
            gameid: gamecode,
            username: username,
            kind: "Game",
            task: "DrawWhite",
            data: "",
        }
        if(!isjudge){
            let count = 0;
            while(count < 7){
                socket.send(JSON.stringify(drawcard));
                console.log("sending object: " + JSON.stringify(drawcard));
                count = count + 1;             
            }
        }
    }
    if(content.task == "DrawWhite"){
        let card1 = document.getElementById("content1");
        let card2 = document.getElementById("content2");
        let card3 = document.getElementById("content3");
        let card4 = document.getElementById("content4");
        let card5 = document.getElementById("content5");
        let card6 = document.getElementById("content6");
        let card7 = document.getElementById("content7");
        let cards = [card1, card2, card3, card4, card5, card6, card7];
        submited = false;
        let foundcard = false;
        cards.forEach(card => {
            if(card.innerHTML == "" && !foundcard){
                card.innerHTML = content.data;
                foundcard = true;
            }
        });
    }
    if(content.task == "ChangeJudge"){
        document.getElementById("content1").innerHTML = "";
        document.getElementById("content2").innerHTML = "";
        document.getElementById("content3").innerHTML = "";
        document.getElementById("content4").innerHTML = "";
        document.getElementById("content5").innerHTML = "";
        document.getElementById("content6").innerHTML = "";
        document.getElementById("content7").innerHTML = "";
        isjudge = true;
    }
    if(content.task == "DrawBlack"){
        document.getElementById("card0").innerHTML = content.data;
    }
    if(content.task == "CreateUserError"){
        
        document.getElementById("startup").style.display = "block";
        document.getElementById("game").style.display = "none";
        show_error(content.data);
    }
    if(content.task == "SubmitCard"){
        let card1 = document.getElementById("content1");
        let card2 = document.getElementById("content2");
        let card3 = document.getElementById("content3");
        let card4 = document.getElementById("content4");
        let card5 = document.getElementById("content5");
        let card6 = document.getElementById("content6");
        let card7 = document.getElementById("content7");
        let cards = [card1, card2, card3, card4, card5, card6, card7];
        let foundcard = false;
        cards.forEach(card => {
            if(card.innerHTML == "" && !foundcard){
                card.innerHTML = content.data;
                judgecards.push(content.username);
                foundcard = true;
            }
        });
    }
});
function show_error(err){
    //document.getElementById("topbar").style.display = "block";
    ///document.getElementById("errmessage").innerHTML = err;
    console.log(err);
}
//only use this as a way of recv data when absolutely nessisary the event listener
//shoud be used whenever possible
/*function recv(){
    while(last_message == message){
        setTimeout(function (){}, 1000);
    } 
}*/
function show_score(){
    document.getElementById("cards").style.display = "none";

}
function show_board(){
    document.getElementById("cards").style.display = "block";
}
function submit(param){
    switch(param){
        case 1: { var card = document.getElementById("content1"); break;}
        case 2: { var card = document.getElementById("content2"); break; }
        case 3: { var card = document.getElementById("content3"); ""; break; }
        case 4: { var card = document.getElementById("content4"); break; }
        case 5: { var card = document.getElementById("content5"); break; }
        case 6: { var card = document.getElementById("content6"); break; }
        case 7: { var card = document.getElementById("content7"); break; }
    }
    let drawcard = {
        gameid: gamecode,
        username: username,
        kind: "Game",
        task: "SubmitCard",
        data: card.innerHTML,
    }
    let selectuser = {
        gameid: gamecode,
        username: judgecards[param],
        kind: "Game",
        task: "SelectWinner",
        data: card.innerHTML,
    }
    if(isopen && !submited && !isjudge){
        socket.send(JSON.stringify(drawcard));
        card.innerHTML = "";
        submited = true;
    }else if(isjudge && isopen){
        socket.send(JSON.stringify(selectuser));
        console.log("sent message: " + JSON.stringify(selectuser));
        isjudge = false;
    }else if(!submited && isjudge){
        show_error("unable to connect to server trying again");
        startup();
    }
}

function newgame(){             
    document.getElementById("firstscreen").style.display = "none";
    document.getElementById("newgamemenu").style.display = "block";
    
}
function startgame(){
    username = document.getElementById("useridnew").value;
    
    let startgame = {
        gameid: gamecode,
        username: "",
        kind: "Admin",
        task: "StartGame",
        data: "",
    }

    if(isopen){
        socket.send(JSON.stringify(startgame));
        document.getElementById("startup").style.display = "none";
        document.getElementById("game").style.display = "block";
        isjudge = true;
    }else{
        show_error("unable to connect to server trying again");
        startup();
    }
}
function joingame(){
    document.getElementById("firstscreen").style.display = "none";
    document.getElementById("joingamemenu").style.display = "block";
}
function join(){
    console.log("join called");
    if(isopen){
        document.getElementById("startup").style.display = "none";
        document.getElementById("game").style.display = "block";
        username = document.getElementById("userid").value;
        gamecode = document.getElementById("gameid").value;
        let CreateUser = {
            gameid: gamecode,
            username: username,
            kind: "Admin",
            task: "CreateUser",
            data: username,
        }
        console.log("sending data: " + JSON.stringify(CreateUser));
        socket.send(JSON.stringify(CreateUser));
    }else{
        show_error("unable to connect to server trying again");
        startup();
    }
}
function hiderror(){
    document.getElementById("topbar").style.display = "none";
}
var rules = false;
function gamerules(){
    if(!rules){
        document.getElementById("rules").style.display = "block";
        rules = true;
    }else{
        document.getElementById("rules").style.display = "none";
        rules = false;
    }
}