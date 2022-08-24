import * as vscode from 'vscode';
import { Config } from './config';
import { Context } from './context';

const EXTENSION_NAME = "rust-deadlock-detector";

export async function activate(context: vscode.ExtensionContext) {
	console.log(`${EXTENSION_NAME} is activating`);
	const config = new Config(context);
	const serverPath = getServerPath(config);
	if(!serverPath) {
		return console.error("cannot find server path");
	}
	const ctx = await Context.create(config, context, serverPath, { kind: "Workspace Folder" });

	ctx.registerCommand("reload", (_) => async () => {
        void vscode.window.showInformationMessage("Reloading rust-deadlock-detector...");
        while (context.subscriptions.length > 0) {
            try {
                context.subscriptions.pop()!.dispose();
            } catch (err) {
                console.error("Dispose error:", err);
            }
        }
        await activate(context).catch(console.error);
    });
	
}



export function deactivate() {

}


function getServerPath(config: Config): string | null {
	return process.env.__DL_LSP_SERVER_DEBUG ?? config.serverPath;
}

