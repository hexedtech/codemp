const vscode = require("vscode");
const codemp = require("./codemp.node");

var CLIENT = null
var CONTROLLER
var CURSOR
var DECORATION = null
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

function _order_tuples(a, b) {
	if (a[0] < b[0]) return (a, b)
	if (a[0] > b[0]) return (b, a)
	if (a[1] < b[1]) return (a, b)
	return (b, a)
}

async function _attach(path) {
	let editor = vscode.window.activeTextEditor
	let doc = editor.document;

	CURSOR = await CLIENT.listen()
	CURSOR.callback((usr, path, start, end) => {
		try {
			if (DECORATION != null) {
				DECORATION.dispose()
				DECORATION = null
			}
			const range_start = new vscode.Position(start[0] - 1, start[1]);
			const range_end = new vscode.Position(start[0] - 1, start[1] + 1);
			const decorationRange = new vscode.Range(range_start, range_end);
			DECORATION = vscode.window.createTextEditorDecorationType(
				{backgroundColor: 'red', color: 'white'}
			)
			editor.setDecorations(DECORATION, [decorationRange])
		} catch (err) {
			vscode.window.showErrorMessage("fuck! " + err)
		}
	})
	vscode.window.onDidChangeTextEditorSelection(async (e) => {
		let buf = e.textEditor.document.uri.toString()
		let selection = e.selections[0] // TODO there may be more than one cursor!!
		let anchor = [selection.anchor.line+1, selection.anchor.character]
		let position = [selection.active.line+1, selection.active.character]
		// (anchor, position) = _order_tuples(anchor, position)
		await CURSOR.send(buf, anchor, position)
	})

	CONTROLLER = await CLIENT.attach(path)
	CONTROLLER.callback((start, end) => {
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
	return CONTROLLER
}

module.exports = {
	activate,
}
