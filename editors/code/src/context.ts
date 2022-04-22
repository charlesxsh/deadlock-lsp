import { Config } from "./config";
import * as vscode from 'vscode';
import { createClient } from "./client";
import { LanguageClient } from "vscode-languageclient/node";

export type Workspace =
    {
        kind: 'Workspace Folder';
    }
    | {
        kind: 'Detached Files';
        files: vscode.TextDocument[];
    };

export class Context {
    private constructor(
        readonly config: Config,
        private readonly extCtx: vscode.ExtensionContext,
        readonly client: LanguageClient,
        readonly serverPath: string,
    ){

    }

    static async create(
        config: Config,
        extCtx: vscode.ExtensionContext,
        serverPath: string,
        workspace: Workspace,
    ): Promise<Context> {
        const client = createClient(serverPath, {});
        const ctx = new Context(config, extCtx, client, serverPath);

	    client.start();
        return ctx;
    }
}