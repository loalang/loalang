import DocsRouter from "./DocsRouter.js";

fetch("/docs.json")
    .then(response => response.json())
    .then(DocsRouter.create.bind(DocsRouter));


