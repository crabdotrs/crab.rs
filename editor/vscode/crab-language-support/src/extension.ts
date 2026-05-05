import * as vscode from 'vscode';
import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
	const config = vscode.workspace.getConfiguration('crab');
	const serverPath = config.get<string>('languageServerPath') || 'crab-lsp';
	const enableLogging = config.get<boolean>('enableLogging') || false;

	const serverOptions: ServerOptions = {
		command: serverPath,
		args: enableLogging ? ['--log'] : [],
		transport: TransportKind.stdio,
		options: {
			env: {
				RUST_LOG: enableLogging ? 'debug' : 'warn'
			}
		}
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{ scheme: 'file', language: 'crab' },
			{ scheme: 'untitled', language: 'crab' }
		],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher('**/.clientrc')
		}
	};

	client = new LanguageClient(
		'crabLanguageServer',
		'Crab Language Server',
		serverOptions,
		clientOptions
	);

	client.start();

	const restartCommand = vscode.commands.registerCommand('crab.restartLanguageServer', async () => {
		if (client) {
			await client.stop();
			client.start();
			vscode.window.showInformationMessage('Crab Language Server restarted');
		}
	});

	context.subscriptions.push(restartCommand);
}

export function deactivate(): Thenable<void> | undefined {
	if (client) {
		return client.stop();
	}
	return undefined;
}
