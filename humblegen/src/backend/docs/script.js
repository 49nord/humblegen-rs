;!function() {
    var endpoints = Array.from(document.getElementsByClassName("endpoint"));
    endpoints.forEach(endpoint => {
        endpoint.getElementsByClassName("endpoint--method-and-route")[0].addEventListener("click", function(event) {
            endpoint.classList.toggle("fold-open");
        })
    });

    function unfoldAll() {
        endpoints.forEach(endpoint => { endpoint.classList.add("fold-open"); event.preventDefault(); });
    }

    function foldAll() {
        endpoints.forEach(endpoint => { endpoint.classList.remove("fold-open"); event.preventDefault(); });
    }

    Array.from(document.getElementsByClassName("unfoldAll")).forEach(b => b.addEventListener("click", unfoldAll));
    Array.from(document.getElementsByClassName("foldAll")).forEach(b => b.addEventListener("click", foldAll));
}();
