export default class Router {
    constructor() {
        this._onPopState = this._onPopState.bind(this);
    }

    static create(...args) {
        const router = new this(...args);

        router.onNavigate(location.pathname);
        router._listen();

        return router;
    }

    async navigate(path) {
        await this.onNavigate(path);
        history.pushState(null, document.title, path)
    }

    _listen() {
        window.addEventListener("popstate", this._onPopState);
    }

    async _onPopState() {
        await router._onNavigate(location.pathname);
    }

    async onNavigate(path) {
        throw new Error(`Implement onNavigate(${path}) in subclass`);
    }
}