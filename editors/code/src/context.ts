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
        const client = createClient(serverPath, {
            // for mac os
            "DYLD_LIBRARY_PATH": config.dyldLibPath,
            // for linux
            "LD_LIBRARY_PATH": config.dyldLibPath,
            "RUST_LOG":"lsp_server=debug",
            __DL_RUSTC:config.luckbud
        });
        const ctx = new Context(config, extCtx, client, serverPath);

	    client.start();
        return ctx;
    }
}