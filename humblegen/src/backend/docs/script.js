// Implements elements that can be folded using another element as handle, used to hide details of
// endpoints and user defined types
;!function() {
    var foldables = Array.from(document.getElementsByClassName("foldable"));
    foldables.forEach(foldable => {
        foldable.getElementsByClassName("foldable-handle")[0].addEventListener("click", function(event) {
            foldable.classList.toggle("fold-open");
        })
    });

    function unfoldAll() {
        foldables.forEach(foldable => { foldable.classList.add("fold-open"); event.preventDefault(); });
    }

    function foldAll() {
        foldables.forEach(foldable => { foldable.classList.remove("fold-open"); event.preventDefault(); });
    }

    Array.from(document.getElementsByClassName("unfoldAll")).forEach(b => b.addEventListener("click", unfoldAll));
    Array.from(document.getElementsByClassName("foldAll")).forEach(b => b.addEventListener("click", foldAll));
}();

// Implements a tabs, a horizontal navigation switching between multiple elements below, used to switch between
// languages in code snippets
;!function() {
    // initialize on page load, show first tab by default
    var tabbedEls = Array.from(document.getElementsByClassName("tabs"));
    tabbedEls.forEach(tabbed => {
        var defaultActive = tabbed.getElementsByClassName("tabs-linked-group-selector")[0].dataset.tabGroup;

        Array.from(document.getElementsByClassName("tabs-linked-group--"+defaultActive)).forEach(b => b.classList.add("tab-active"));
        Array.from(document.getElementsByClassName("tabs-linked-group-selector--"+defaultActive)).forEach(b => b.classList.add("tab-active"));
    });

    tabbedEls.forEach(tabbed => {
        Array.from(tabbed.getElementsByClassName("tabs-linked-group-selector")).forEach(tab => {
            tab.addEventListener("click", function(event) {
                 var linked = tab.dataset.tabGroup;
                 Array.from(document.getElementsByClassName("tab-active")).forEach(b => b.classList.remove("tab-active"));
                 Array.from(document.getElementsByClassName("tabs-linked-group--"+linked)).forEach(b => b.classList.add("tab-active"));
                 Array.from(document.getElementsByClassName("tabs-linked-group-selector--"+linked)).forEach(b => b.classList.add("tab-active"));
                 //tabItem.classList.toggle("fold-open");
                 event.preventDefault();
            })
        })
    })
}();
