(function(){
var root=document.documentElement,button=document.getElementById("theme-switch");
if(!button)return;
var query=matchMedia("(prefers-color-scheme: dark)");
var isDark=function(){var chosen=root.dataset.theme;return chosen?chosen==="dark":query.matches};
var show=function(){button.setAttribute("aria-pressed",isDark()?"true":"false")};
button.addEventListener("click",function(){
var next=isDark()?"light":"dark";
root.dataset.theme=next;
try{localStorage.setItem("cuhlumn:theme",next)}catch(e){}
show();
});
query.addEventListener("change",show);
show();
})();
