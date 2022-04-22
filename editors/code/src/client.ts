import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
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
		traceOutputChannel
	};
	const client = new LanguageClient(
		"rust-deadlock-server",
		"Rust Deadlock Detector Language Server",
		serverOptions,
		clientOptions
	);
	
	client.registerProposedFeatures();
	return client;
}