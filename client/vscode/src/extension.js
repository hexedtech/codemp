const vscode = require("vscode");
const codemp = require("./codemp.node");

var CLIENT = null
var CONTROLLER
var OP_CACHE = new Set()

async function activate(context) {
	context.subscriptions.push(
		vscode.commands.registerCommand("codemp.connect", connect),
		vscode.commands.registerCommand("codemp.share", share),
		vscode.commands.registerCommand("codemp.join", join),
	)
}

async function connect() {
	let host = await vscode.window.showInputBox({prompt: "server host (default to http://fantabos.co:50051)"})
	if (host === undefined) return  // user cancelled with ESC
	if (host.length == 0) host = "http://fantabos.co:50051"
	CLIENT = await codemp.connect(host)
	vscode.window.showInformationMessage(`Connected to codemp @[${host}]`);
}

async function share() {
	if (CLIENT === null) {
		vscode.window.showErrorMessage("No connected client");
	}

	let path = await vscode.window.showInputBox({prompt: "buffer uri (default to file path)"})
	if (path === undefined) return  // user cancelled with ESC
	if (path.length == 0) path = doc.uri.toString()

	let doc = vscode.window.activeTextEditor.document;

	try {
		if (!await CLIENT.create(path, doc.getText())) {
			vscode.window.showErrorMessage("Could not share buffer");
		}

		await _attach(path)

		vscode.window.showInformationMessage(`Shared document on buffer "${path}"`);
	} catch (err) {
		vscode.window.showErrorMessage("Error sharing: " + err)
	}
}

async function join() {
	if (CLIENT === null) {
		vscode.window.showErrorMessage("No connected client");
	}

	let path = await vscode.window.showInputBox({prompt: "buffer uri"})

	try {
		let controller = await _attach(path)

		vscode.window.showInformationMessage(`Joined buffer "${path}"`);

		let editor = vscode.window.activeTextEditor

		let range = new vscode.Range(
			editor.document.positionAt(0),
			editor.document.positionAt(editor.document.getText().length)
		)
		let content = controller.content()
		OP_CACHE.add((range, content))
		editor.edit(editBuilder => editBuilder.replace(range, content))
	} catch (err) {
		vscode.window.showErrorMessage("error joining " + err)
	}
}

async function _attach(path) {
	let doc = vscode.window.activeTextEditor.document;
	CONTROLLER = await CLIENT.attach(path)
	vscode.workspace.onDidChangeTextDocument(async (e) => {
		if (e.document != doc) return
		for (let change of e.contentChanges) {
			if (OP_CACHE.has((change.range, change.text))) {
				OP_CACHE.delete((change.range, change.text))
				continue
			}
			try {
				await CONTROLLER.apply(change.rangeOffset, change.text, change.rangeOffset + change.rangeLength)
			} catch (err) {
				vscode.window.showErrorMessage("failed sending change: " + err)
			}
		}
	})
	let editor = vscode.window.activeTextEditor
	CONTROLLER.set_callback((start, end) => {
		// TODO only change affected document range
		let content = CONTROLLER.content()
		let range = new vscode.Range(
			editor.document.positionAt(0),
			editor.document.positionAt(editor.document.getText().length)
		)
		try {
			OP_CACHE.add((range, content))
			editor.edit(editBuilder => editBuilder.replace(range, content))
		} catch (err) {
			vscode.window.showErrorMessage("could not set buffer: " + err)
		}
	})
	return CONTROLLER
}

module.exports = {
	activate,
}
