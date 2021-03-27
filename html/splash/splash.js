var modal = document.getElementById("Rules");
var btn = document.getElementById("open-rules");
var span = document.getElementsByClassName("close)[0];

btn.onClick = function() {
    alert("Button clicked");
    modal.style.display = "block";
}

window.onclick = function(event) {
    if(event.target == modal) {
        modal.style.display = "none";
    }
}
