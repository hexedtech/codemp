const vscode = require("vscode");

module.exports = {
	activate,
	deactivate,
};

function activate(context) {
	// This must match the command property in the package.json
	const commandID = "codemp.connect";
	let disposable = vscode.commands.registerCommand(commandID, sayHello);
	context.subscriptions.push(disposable);
}

function connect() {
	vscode.window.showInformationMessage("Connecting to CodeMP!");
}

function deactivate() {}
