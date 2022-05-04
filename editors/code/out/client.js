"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createClient = void 0;
const node_1 = require("vscode-languageclient/node");
const vscode = require("vscode");
function createClient(serverPath, extraEnv) {
    const newEnv = Object.assign({}, process.env);
    Object.assign(newEnv, extraEnv);
    const run = {
        command: serverPath,
        options: { env: newEnv }
    };
    const serverOptions = {
        run,
        debug: run,
    };
    const traceOutputChannel = vscode.window.createOutputChannel('Rust Deadlock Detector Language Server Trace');
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'rust' }],
        traceOutputChannel,
        diagnosticCollectionName: "rust-deadlock-detector",
        errorHandler: {
            error: (err) => {
                console.error("lsp client", err);
                return {
                    action: node_1.ErrorAction.Continue
                };
            },
            closed: () => ({
                action: node_1.CloseAction.Restart
            })
        }
    };
    const client = new node_1.LanguageClient("rust-deadlock", "Rust Deadlock Detector Language Server", serverOptions, clientOptions);
    return client;
}
exports.createClient = createClient;
//# sourceMappingURL=client.js.map