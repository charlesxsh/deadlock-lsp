import {
	CloseAction,
	ErrorAction,
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	Trace,
	TransportKind
  } from 'vscode-languageclient/node';
import * as vscode from 'vscode';


export function createClient(serverPath: string, extraEnv: Record<string, any>): LanguageClient {
	const newEnv = Object.assign({}, process.env);
    Object.assign(newEnv, extraEnv);
	const run: Executable = {
		command: serverPath,
		options: {env:newEnv}
	};
	const serverOptions: ServerOptions = {
        run,
        debug: run,
    };
	const traceOutputChannel = vscode.window.createOutputChannel(
        'Rust Deadlock Detector Language Server Trace',
    );
	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'rust' }],
		traceOutputChannel,
		diagnosticCollectionName: "rust-deadlock-detector",
		errorHandler: {
			error: (err) => {
				console.error("lsp client", err);
				return{
					action: ErrorAction.Continue
				};
			},
			closed: () => ({
				action: CloseAction.Restart
			})
		}

	};
	const client = new LanguageClient(
		"rust-deadlock",
		"Rust Deadlock Detector Language Server",
		serverOptions,
		clientOptions
	);	
	
	return client;
}