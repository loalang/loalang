import Router from "./Router.js";

export default class DocsRouter extends Router {
    constructor(docs) {
        super();

        this._docs = docs;

    }

    async onNavigate(path) {
        if (this._root != null) {
            this._root.unmount();
        }
        this._root = new Main(this);
        this._root.mount();
    }
}

class Main {
    constructor(router) {
        this._router = router;
        this._element = document.createElement("button");

        this._element.textContent ' Clie'
    }

    unmount() {
        document.body.removeChild(this._element);
    }

    mount() {
        document.body.appendChild(this._element);
    }
}