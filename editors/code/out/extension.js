"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = require("vscode");
const config_1 = require("./config");
const context_1 = require("./context");
const collection = vscode.languages.createDiagnosticCollection('result');
const EXTENSION_NAME = "rust-deadlock-detector";
// create a decorator type
const lifetimeLineDecorationType = vscode.window.createTextEditorDecorationType({
    cursor: 'crosshair',
    // use a themable color. See package.json for the declaration and default values.
    backgroundColor: { id: 'vrlifetime.lifetimeLineBackground' },
    overviewRulerColor: "#ce0f0f38",
    overviewRulerLane: vscode.OverviewRulerLane.Left,
});
function updateDecorations() {
    const editor = vscode.window.activeTextEditor;
    const workspaceFolder = vscode.workspace.workspaceFolders;
    if (!workspaceFolder || !editor) {
        return;
    }
    let select = editor.selection;
    let filename = editor.document.uri.path;
    let ranges = [];
    // for (let key in lifetimeObj){
    // 	let regEx = new RegExp(key + "$");
    // 	let match = regEx.exec(filename);
    // 	if (match) {
    // 		// @ts-ignore
    // 		ranges = lifetimeObj[key];
    // 		break;
    // 	}
    // }
    const lifetimeLines = [];
    if (ranges) {
        for (let i in ranges) {
            const decoration = { range: ranges[i], hoverMessage: `lifetime for` };
            lifetimeLines.push(decoration);
        }
    }
    editor.setDecorations(lifetimeLineDecorationType, lifetimeLines);
}
async function activate(context) {
    console.log(`${EXTENSION_NAME} is activating`);
    const config = new config_1.Config(context);
    const serverPath = getServerPath(config);
    if (!serverPath) {
        return console.error("cannot find server path");
    }
    const ctx = await context_1.Context.create(config, context, serverPath, { kind: "Workspace Folder" });
    // vscode.window.onDidChangeTextEditorSelection(event => {
    // 	const activeEditor = vscode.window.activeTextEditor;
    // 	if (activeEditor == event.textEditor) {
    // 		if (event.selections[0].isEmpty) {return;}
    // 		const selectedText = activeEditor.document.getText(event.selections[0]);
    // 		ctx.client.outputChannel.appendLine(`selected ${selectedText}`);
    // 		// ctx.client.sendRequest()
    // 		// updateLifetimeObj(false);
    // 		// triggerUpdateDecorations();
    // 	}
    // }, null, context.subscriptions);
    // // context.subscriptions.push(collection);
    // vscode.workspace.onDidSaveTextDocument(event => {
    // 	if (!vscode.workspace.workspaceFolders) {
    // 		return;
    // 	}
    // 	const workspaceDir = vscode.workspace.workspaceFolders[0].uri.fsPath;
    // 	const result = runkDetector(workspaceDir, "double-lock");
    // 	const results = parseDetectorOutput(result);
    // 	const diagCtx = getDiagnoisContext(results);
    // 	console.log(diagCtx);
    // 	updateDiagnostics(collection, diagCtx);
    // }, null, context.subscriptions);
}
exports.activate = activate;
function getDiagnoisContext(detectorOutput) {
    let diagnosticObj = Object();
    for (let i in detectorOutput) {
        let key_elem = detectorOutput[i]["secondLock"];
        let related_elem = detectorOutput[i]["firstLock"];
        if (key_elem["fname"] in diagnosticObj) {
            if (key_elem["pos"] in diagnosticObj[key_elem["fname"]]) {
                diagnosticObj[key_elem["fname"]][key_elem["pos"]].push({
                    msg: key_elem["msg"],
                    related: [related_elem]
                });
            }
            else {
                diagnosticObj[key_elem["fname"]][key_elem["pos"]] = [{
                        msg: key_elem["msg"],
                        related: [related_elem]
                    }];
            }
        }
        else {
            diagnosticObj[key_elem["fname"]] = Object();
            diagnosticObj[key_elem["fname"]][key_elem["pos"]] = [{
                    msg: key_elem["msg"],
                    related: [related_elem]
                }];
        }
    }
    return diagnosticObj;
}
function updateDiagnostics(collection, diagnosticObj) {
    console.log("updateDiagnostics");
    let editor = vscode.window.activeTextEditor;
    if (!editor || !vscode.workspace.workspaceFolders) {
        return;
    }
    console.log("updateDiagnostics updating");
    let diagnosticArray = [];
    let document = editor.document;
    let dirPath = vscode.workspace.workspaceFolders[0].uri.path;
    for (let filename in diagnosticObj) {
        let filePath = dirPath + '/' + filename;
        let fileUri = vscode.Uri.file(filePath);
        let fileDiagnostic = diagnosticObj[filename];
        let entryArray = [];
        for (let range in fileDiagnostic) {
            for (let i in fileDiagnostic[range]) {
                let relatedInformations = [];
                for (let j in fileDiagnostic[range][i]["related"]) {
                    let relatedInfo = fileDiagnostic[range][i]["related"][j];
                    let relatedUri = vscode.Uri.file(dirPath + "/" + relatedInfo["fname"]);
                    let posRange = parsePositionRange(relatedInfo["pos"]);
                    relatedInformations.push(new vscode.DiagnosticRelatedInformation(new vscode.Location(relatedUri, posRange), relatedInfo["msg"]));
                }
                entryArray.push({
                    code: '',
                    message: fileDiagnostic[range][i]["msg"],
                    range: parsePositionRange(range),
                    severity: vscode.DiagnosticSeverity.Warning,
                    source: `${EXTENSION_NAME}`,
                    relatedInformation: relatedInformations
                });
            }
        }
        diagnosticArray.push([
            fileUri, entryArray
        ]);
    }
    console.log("updateDiagnostics", diagnosticArray);
    if (document && document.uri.path.search(dirPath) != -1) {
        if (diagnosticArray.length != 0) {
            // @ts-ignore
            collection.set(diagnosticArray);
        }
        else {
            collection.clear();
        }
    }
    else {
        collection.clear();
    }
}
async function tryActivate(context) {
}
function deactivate() {
}
exports.deactivate = deactivate;
function getServerPath(config) {
    return process.env.__DL_LSP_SERVER_DEBUG ?? config.serverPath;
}
function parsePosition(s) {
    let result = s.split(":");
    return new vscode.Position(Number(result[0]) - 1, Number(result[1]) - 1);
}
function parsePositionRange(s) {
    let tmp = s.split(": ");
    return new vscode.Range(parsePosition(tmp[0]), parsePosition(tmp[1]));
}
//# sourceMappingURL=extension.js.map