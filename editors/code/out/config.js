"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Config = void 0;
const vscode = require("vscode");
class Config {
    constructor(ctx) {
        this.rootSection = "rust-deadlock-dectector";
    }
    get(path) {
        return this.cfg.get(path);
    }
    get serverPath() {
        return this.get("server.path") ?? this.get("serverPath");
    }
    get cfg() {
        return vscode.workspace.getConfiguration(this.rootSection);
    }
}
exports.Config = Config;
//# sourceMappingURL=config.js.map