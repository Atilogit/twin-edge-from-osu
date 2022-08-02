var def = document.getElementsByClassName("default")[0];
def.classList.remove("default");

mo_tab(def);
mc_tab(def);

function mo_tab(e) {
	def.classList.remove("hover");
	e.classList.add("hover");
}

function me_tab(e) {
	e.classList.remove("hover");
	def.classList.add("hover");
}

function mc_tab(e) {
	var tabs = document.querySelectorAll('[tab]');
	for(var i = 0; i<tabs.length; i++) {
		tabs[i].hidden = true;
	}
	var target = document.querySelector('[tab="' + e.getAttribute("tab-target") + '"]');
	target.hidden = false;
	def = e;
}

var els = document.getElementsByClassName("button-tab");
for(var i = 0; i < els.length; i++){
	els[i].onmouseover = function() { mo_tab(this) };
	els[i].onmouseout = function() { me_tab(this) };
	els[i].onclick = function() { mc_tab(this) };
}