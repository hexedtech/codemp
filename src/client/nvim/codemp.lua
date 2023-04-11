local BINARY = "/home/alemi/projects/codemp/target/debug/client-nvim --debug"

if vim.g.codemp_jobid == nil then
	vim.g.codemp_jobid = vim.fn.jobstart(BINARY, { rpc = true })
end

local M = {}
M.create = function(path, content) return vim.rpcrequest(vim.g.codemp_jobid, "create", path, content) end
M.insert = function(path, txt, pos) return vim.rpcrequest(vim.g.codemp_jobid, "insert", path, txt, pos) end
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
	return vim.rpcrequest(vim.g.codemp_jobid, "attach", path)
end

return M
