local BINARY = vim.g.codemp_binary or "/home/alemi/projects/codemp/target/debug/client-nvim"

local M = {}
M.jobid = nil
M.create = function(path, content) return vim.rpcrequest(M.jobid, "create", path, content) end
M.insert = function(path, txt, pos) return vim.rpcrequest(M.jobid, "insert", path, txt, pos) end
M.cursor = function(path, row, col) return vim.rpcrequest(M.jobid, "cursor", path, row, col) end
M.delete = function(path, pos, count) return vim.rpcrequest(M.jobid, "delete", path, pos, count) end
M.attach = function(path) return vim.rpcrequest(M.jobid, "attach", path) end
M.listen = function(path) return vim.rpcrequest(M.jobid, "listen", path) end
M.detach = function(path) return vim.rpcrequest(M.jobid, "detach", path) end

local function cursor_offset()
	local cursor = vim.api.nvim_win_get_cursor(0)
	return vim.fn.line2byte(cursor[1]) + cursor[2] - 1
end

local codemp_autocmds = vim.api.nvim_create_augroup("CodempAuGroup", { clear = true })

local function hook_callbacks(path, buffer)
	vim.api.nvim_create_autocmd(
		{ "InsertCharPre" },
		{
			callback = function(_) M.insert(path, vim.v.char, cursor_offset()) end,
			buffer = buffer,
			group = codemp_autocmds,
		}
	)
	vim.api.nvim_create_autocmd(
		{ "CursorMoved", "CursorMovedI" },
		{
			callback = function(_)
				local cursor = vim.api.nvim_win_get_cursor(0)
				M.cursor(path, cursor[1], cursor[2])
			end,
			buffer = buffer,
			group = codemp_autocmds,
		}
	)
	vim.keymap.set('i', '<BS>', function() M.delete(path, cursor_offset(), 1) return '<BS>' end, {expr = true, buffer = buffer})
	vim.keymap.set('i', '<Del>', function() M.delete(path, cursor_offset() + 1, 1) return '<Del>' end, {expr = true, buffer = buffer})
	vim.keymap.set('i', '<CR>', function() M.insert(path, "\n", cursor_offset()) return '<CR>'end, {expr = true, buffer = buffer})
end

local function unhook_callbacks(buffer)
	vim.api.nvim_clear_autocmds({ group = codemp_autocmds, buffer = buffer })
	vim.keymap.del('i', '<BS>',  { buffer = buffer })
	vim.keymap.del('i', '<Del>', { buffer = buffer })
	vim.keymap.del('i', '<CR>',  { buffer = buffer })
end

vim.api.nvim_create_user_command('Connect',
	function(args)
		if M.jobid ~= nil then
			print("already connected, disconnect first")
			return
		end
		local bin_args = { BINARY }
		if #args.args > 0 then
			table.insert(bin_args, "--host")
			table.insert(bin_args, args.args[1])
		end
		if args.bang then
			table.insert(bin_args, "--debug")
		end
		M.jobid = vim.fn.jobstart(
			bin_args,
			{
				rpc = true,
				on_stderr = function(_, data, _)
					for _, line in pairs(data) do
						print(line)
					end
					-- print(vim.fn.join(data, "\n"))
				end,
				stderr_buffered = false,
			}
		)
		if M.jobid <= 0 then
			print("[!] could not start codemp client")
		end
	end,
{ nargs='?', bang=true })

vim.api.nvim_create_user_command('Stop',
	function(_)
		vim.fn.jobstop(M.jobid)
		M.jobid = nil
	end,
{ bang=true })

vim.api.nvim_create_user_command('Share',
	function(args)
		if M.jobid <= 0 then
			print("[!] connect to codemp server first")
			return
		end
		local path = args.fargs[1]
		local bufnr = vim.api.nvim_get_current_buf()
		local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
		M.create(path, vim.fn.join(lines, "\n"))
		hook_callbacks(path, bufnr)
		M.attach(path)
		M.listen(path)
	end,
{ nargs=1 })

vim.api.nvim_create_user_command('Join',
	function(args)
		if M.jobid <= 0 then
			print("[!] connect to codemp server first")
			return
		end
		local path = args.fargs[1]
		local bufnr = vim.api.nvim_get_current_buf()
		hook_callbacks(path, bufnr)
		M.attach(path)
		M.listen(path)
	end,
{ nargs=1 })

vim.api.nvim_create_user_command('Detach',
	function(args)
		local bufnr = vim.api.nvim_get_current_buf()
		unhook_callbacks(bufnr)
		M.detach(args.fargs[1])
	end,
{ nargs=1 })

return M
