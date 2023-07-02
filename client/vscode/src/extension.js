const vscode = require("vscode");
const codemp = require("./codemp.node");

var CLIENT = null

async function activate(context) {
	// This must match the command property in the package.json
	const commandID = "codemp.connect";
	let disposable = vscode.commands.registerCommand(commandID, connect);
	context.subscriptions.push(disposable);
}

async function connect() {
	CLIENT = await codemp.connect("http://fantabos.co:50051")
	vscode.window.showInformationMessage("Connected to CodeMP!");
}

module.exports = {
	activate,
}
