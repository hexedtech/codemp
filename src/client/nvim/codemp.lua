local BINARY = "/home/alemi/projects/codemp/target/debug/client-nvim --debug"

if vim.g.codemp_jobid == nil then
	vim.g.codemp_jobid = vim.fn.jobstart(
		BINARY,
		{
			rpc = true,
			on_stderr = function(_, data, _) print(vim.fn.join(data, "\n")) end,
		}
	)
end

local M = {}
M.create = function(path, content) return vim.rpcrequest(vim.g.codemp_jobid, "create", path, content) end
M.insert = function(path, txt, pos) return vim.rpcrequest(vim.g.codemp_jobid, "insert", path, txt, pos) end
M.delete = function(path, pos, count) return vim.rpcrequest(vim.g.codemp_jobid, "delete", path, pos, count) end
M.sync   = function(path) return vim.rpcrequest(vim.g.codemp_jobid, "sync", path) end
M.dump   = function() return vim.rpcrequest(vim.g.codemp_jobid, "dump") end
M.attach = function(path) return vim.rpcrequest(vim.g.codemp_jobid, "attach", path) end

local function cursor_offset()
	local cursor = vim.api.nvim_win_get_cursor(0)
	return vim.fn.line2byte(cursor[1]) + cursor[2] - 1
end

local function hook_callbacks(path)
	vim.api.nvim_create_autocmd(
		{ "InsertCharPre" },
		{ callback = function() M.insert(path, vim.v.char, cursor_offset()) end }
	)
	vim.keymap.set('i', '<BS>', function()
		local off = cursor_offset()
		pcall(M.delete, path, off, 1)
		return '<BS>'
	end, {expr = true})
	vim.keymap.set('i', '<Del>', function()
		pcall(M.cancel, path, cursor_offset(), 1)
		return '<Del>'
	end, {expr = true})
	vim.keymap.set('i', '<CR>', function()
		pcall(M.insertpath, "\n", cursor_offset())
		return '<CR>'
	end, {expr = true})
end

vim.api.nvim_create_user_command(
	'Share',
	function(args)
		local path = args.fargs[1]
		local bufnr = vim.api.nvim_get_current_buf()
		local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
		M.create(path, vim.fn.join(lines, "\n"))
		hook_callbacks(path)
		M.attach(path)
	end,
	{nargs=1}
)

vim.api.nvim_create_user_command(
	'Join',
	function(args)
		local path = args.fargs[1]
		M.sync(path)
		hook_callbacks(path)
		M.attach(path)
	end,
	{nargs=1}
)

return M
