(function(){
var bar=document.querySelector("div[data-announcement]");
var button=bar&&bar.querySelector(".announcement-close");
if(!button)return;
button.addEventListener("click",function(){
try{localStorage.setItem("announcement:"+bar.getAttribute("data-announcement"),"1")}catch(e){}
document.documentElement.setAttribute("data-announcement-dismissed","");
});
})();
