local BINARY = vim.g.codemp_binary or "./codemp-client-nvim"

local M = {}

M.jobid   = nil
M.create  = function(path, content) return vim.rpcrequest(M.jobid, "create", path, content) end
M.insert  = function(path, txt, pos) return vim.rpcrequest(M.jobid, "insert", path, txt, pos) end
M.delete  = function(path, pos, count) return vim.rpcrequest(M.jobid, "delete", path, pos, count) end
M.replace = function(path, txt) return vim.rpcrequest(M.jobid, "replace", path, txt) end
M.cursor  = function(path, cur) return vim.rpcrequest(M.jobid, "cursor", path, cur[1][1], cur[1][2], cur[2][1], cur[2][2]) end
M.attach  = function(path) return vim.rpcrequest(M.jobid, "attach", path) end
M.listen  = function(path) return vim.rpcrequest(M.jobid, "listen", path) end
M.detach  = function(path) return vim.rpcrequest(M.jobid, "detach", path) end

local function cursor_offset()
	local cursor = vim.api.nvim_win_get_cursor(0)
	return vim.fn.line2byte(cursor[1]) + cursor[2] - 1
end

local codemp_autocmds = vim.api.nvim_create_augroup("CodempAuGroup", { clear = true })

local function get_cursor_range()
	local mode = vim.fn.mode()
	if mode == "" or mode == "s" or mode == "Vs" or mode == "V" or mode == "vs" or mode == "v" then
		local start = vim.fn.getpos("'<")
		local finish = vim.fn.getpos("'>")
		return {
			{ start[2], start[3] },
			{ finish[2], finish[3] }
		}
	else
		local cursor = vim.api.nvim_win_get_cursor(0)
		return {
			{ cursor[1], cursor[2] },
			{ cursor[1], cursor[2] + 1 },
		}
	end
end

local function hook_callbacks(path, buffer)
	vim.api.nvim_create_autocmd(
		{ "InsertCharPre" },
		{
			callback = function(_)
				pcall(M.insert, path, vim.v.char, cursor_offset()) -- TODO log errors
			end,
			buffer = buffer,
			group = codemp_autocmds,
		}
	)
	vim.api.nvim_create_autocmd(
		{ "CursorMoved", "CompleteDone", "InsertEnter", "InsertLeave" },
		{
			callback = function(args)
				local lines = vim.api.nvim_buf_get_lines(args.buf, 0, -1, false)
				pcall(M.replace, path, vim.fn.join(lines, "\n")) -- TODO log errors
				pcall(M.cursor, path, get_cursor_range()) -- TODO log errors
			end,
			buffer = buffer,
			group = codemp_autocmds,
		}
	)
	local last_line = 0
	vim.api.nvim_create_autocmd(
		{ "CursorMovedI" },
		{
			callback = function(args)
				local cursor = get_cursor_range()
				pcall(M.cursor, path, cursor) -- TODO log errors
				if cursor[1][1] == last_line then
					return
				end
				last_line = cursor[1][1]
				local lines = vim.api.nvim_buf_get_lines(args.buf, 0, -1, false)
				pcall(M.replace, path, vim.fn.join(lines, "\n")) -- TODO log errors
			end,
			buffer = buffer,
			group = codemp_autocmds,
		}
	)
	vim.keymap.set('i', '<BS>', function() pcall(M.delete, path, cursor_offset(), 1) return '<BS>' end, {expr = true, buffer = buffer}) -- TODO log errors
	vim.keymap.set('i', '<Del>', function() pcall(M.delete, path, cursor_offset() + 1, 1) return '<Del>' end, {expr = true, buffer = buffer}) -- TODO log errors
end

local function unhook_callbacks(buffer)
	vim.api.nvim_clear_autocmds({ group = codemp_autocmds, buffer = buffer })
	vim.keymap.del('i', '<BS>',  { buffer = buffer })
	vim.keymap.del('i', '<Del>', { buffer = buffer })
end

local function auto_address(addr)
	if not string.find(addr, "://") then
		addr = string.format("http://%s", addr)
	end
	if not string.find(addr, ":", 7) then -- skip first 7 chars because 'https://'
		addr = string.format("%s:50051", addr)
	end
	return addr
end

vim.api.nvim_create_user_command('Connect',
	function(args)
		if M.jobid ~= nil and M.jobid > 0 then
			print("already connected, disconnect first")
			return
		end
		local bin_args = { BINARY }
		if #args.fargs > 0 then
			table.insert(bin_args, "--host")
			table.insert(bin_args, auto_address(args.fargs[1]))
		end
		if vim.g.codemp_remote_debug then
			table.insert(bin_args, "--remote-debug")
			table.insert(bin_args, vim.g.codemp_remote_debug)
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
				env = { RUST_BACKTRACE = 1 }
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
		if M.jobid == nil or M.jobid <= 0 then
			print("[!] connect to codemp server first")
			return
		end
		local path = args.fargs[1]
		local bufnr = vim.api.nvim_get_current_buf()
		local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
		vim.opt.fileformat = "unix"
		M.create(path, vim.fn.join(lines, "\n"))
		hook_callbacks(path, bufnr)
		M.attach(path)
		M.listen(path)
	end,
{ nargs=1 })

vim.api.nvim_create_user_command('Join',
	function(args)
		if M.jobid == nil or M.jobid <= 0 then
			print("[!] connect to codemp server first")
			return
		end
		local path = args.fargs[1]
		local bufnr = vim.api.nvim_get_current_buf()
		vim.opt.fileformat = "unix"
		hook_callbacks(path, bufnr)
		M.attach(path)
		M.listen(path)
	end,
{ nargs=1 })

vim.api.nvim_create_user_command('Detach',
	function(args)
		local bufnr = vim.api.nvim_get_current_buf()
		if M.detach(args.fargs[1]) then
			unhook_callbacks(bufnr)
			print("[/] detached from buffer")
		else
			print("[!] error detaching from buffer")
		end
	end,
{ nargs=1 })

return M
