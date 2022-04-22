"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Context = void 0;
const client_1 = require("./client");
class Context {
    constructor(config, extCtx, client, serverPath) {
        this.config = config;
        this.extCtx = extCtx;
        this.client = client;
        this.serverPath = serverPath;
    }
    static async create(config, extCtx, serverPath, workspace) {
        const client = (0, client_1.createClient)(serverPath, {});
        const ctx = new Context(config, extCtx, client, serverPath);
        client.start();
        return ctx;
    }
}
exports.Context = Context;
//# sourceMappingURL=context.js.map