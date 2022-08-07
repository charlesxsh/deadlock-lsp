
import * as vscode from 'vscode';

export class Config {
    readonly rootSection = "rust-deadlock-dectector";
    constructor(ctx: vscode.ExtensionContext) {
       
    }
    private get<T>(path: string): T {
        return this.cfg.get<T>(path)!;
    }

    get serverPath() {
        return this.get<null | string>("serverPath");
    }

    get dyldLibPath() {
        return this.get<null | string>("dyldLibPath");
    }

    get luckbud() {
        return this.get<null | string>("luckbud");
    }
    
    private get cfg(): vscode.WorkspaceConfiguration {
        return vscode.workspace.getConfiguration(this.rootSection);
    }
}