(function () {
  var fetchButton = document.getElementById("picker-fetch");
  var urlInput = document.getElementById("picker-url");
  var selectorInput = document.getElementById("picker-selector");
  var frame = document.getElementById("picker-frame");

  function cssEscape(value) {
    if (window.CSS && CSS.escape) return CSS.escape(value);
    return value.replace(/[^a-zA-Z0-9_-]/g, "\\$&");
  }

  function selectorFor(element) {
    if (element.id) return "#" + cssEscape(element.id);
    var parts = [];
    while (element && element.nodeType === 1 && element.tagName.toLowerCase() !== "html") {
      var part = element.tagName.toLowerCase();
      if (element.classList.length) {
        part += "." + Array.from(element.classList).slice(0, 2).map(cssEscape).join(".");
      }
      var parent = element.parentElement;
      if (parent) {
        var sameTag = Array.from(parent.children).filter(function (child) {
          return child.tagName === element.tagName;
        });
        if (sameTag.length > 1) part += ":nth-of-type(" + (sameTag.indexOf(element) + 1) + ")";
      }
      parts.unshift(part);
      if (parts.length >= 4) break;
      element = parent;
    }
    return parts.join(" > ");
  }

  function installPicker(doc) {
    if (!doc || doc.__feedsmithPickerInstalled) return;
    doc.__feedsmithPickerInstalled = true;
    var style = doc.createElement("style");
    style.textContent = ".__feedsmith_hover{outline:2px solid #0f766e!important;cursor:pointer!important}";
    (doc.head || doc.documentElement).appendChild(style);
    var current;
    doc.addEventListener("pointerover", function (event) {
      if (current) current.classList.remove("__feedsmith_hover");
      current = event.target;
      if (current && current.classList) current.classList.add("__feedsmith_hover");
    });
    doc.addEventListener("click", function (event) {
      event.preventDefault();
      event.stopPropagation();
      selectorInput.value = selectorFor(event.target);
    }, true);
  }

  fetchButton.addEventListener("click", async function () {
    fetchButton.disabled = true;
    try {
      var body = new URLSearchParams({ url: urlInput.value }).toString();
      var response = await fetch("/api/fetch-preview-html", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded;charset=UTF-8" },
        body: body
      });
      if (!response.ok) throw new Error(await response.text());
      var data = await response.json();
      frame.addEventListener("load", function onLoad() {
        frame.removeEventListener("load", onLoad);
        try {
          installPicker(frame.contentDocument);
        } catch (error) {
          selectorInput.value = "Picker setup failed: " + error.message;
        }
      });
      frame.srcdoc = data.html;
    } finally {
      fetchButton.disabled = false;
    }
  });
})();
