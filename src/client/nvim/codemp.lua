local BINARY = "/home/alemi/source/codemp/target/debug/client-nvim --debug"

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
M.dump   = function() return vim.rpcrequest(vim.g.codemp_jobid, "dump") end
M.attach = function(path)
	vim.api.nvim_create_autocmd(
		{ "InsertCharPre" },
		{
			callback = function()
				local cursor = vim.api.nvim_win_get_cursor(0)
				M.insert(path, vim.v.char, cursor[2])
			end,
		}
	)
	vim.keymap.set('i', '<BS>', function()
		local cursor = vim.api.nvim_win_get_cursor(0)
		M.delete(path, cursor[2], 1)
		return '<BS>'
	end, {expr = true})
	vim.keymap.set('i', '<CR>', function()
		local cursor = vim.api.nvim_win_get_cursor(0)
		M.insert(path, "\n", cursor[2])
		return '<CR>'
	end, {expr = true})
	return vim.rpcrequest(vim.g.codemp_jobid, "attach", path)
end

return M
