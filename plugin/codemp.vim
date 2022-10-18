" Copyright 2017 Justin Charette
"
" Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
" http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
" <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
" option. This file may not be copied, modified, or distributed
" except according to those terms.

if ! exists('s:jobid')
	let s:jobid = 0
endif

" TODO I know I know...
let s:bin = "/home/alemi/projects/codemp/target/debug/client-neovim"

function codemp#init()
	let result = s:StartJob()

	if 0 == result
		echoerr "codeMP: cannot start rpc process"
	elseif -1 == result
		echoerr "codeMP: rpc process is not executable"
	else
		let s:jobid = result
		let g:codemp_jobid = result
		call s:ConfigureJob(result)
	endif
endfunction

function s:StartJob()
	if 0 == s:jobid
		let id = jobstart([s:bin], { 'rpc': v:true, 'on_stderr': function('s:OnStderr') })
		return id
	else
		return 0
	endif
endfunction

function s:StopJob()
	if 0 < s:jobid
		augroup codeMp
			autocmd!		" clear all previous autocommands
		augroup END

		call rpcnotify(s:jobid, 'quit')
		let result = jobwait(s:jobid, 500)

		if -1 == result
			" kill the job
			call jobstop(s:jobid)
		endif

		" reset job id back to zero
		let s:jobid = 0
	endif
endfunction

function s:ConfigureJob(jobid)
	augroup codeMp
		" clear all previous autocommands
		autocmd!

		autocmd VimLeavePre * :call s:StopJob()

		autocmd CursorMoved * :call codemp#cursor()
	augroup END
endfunction

function s:NotifyInsertEnter()
	" let [ bufnum, lnum, column, off ] = getpos('.')
	call rpcnotify(s:jobid, 'insert', 1)
endfunction

function s:NotifyInsertLeave()
	call rpcnotify(s:jobid, 'insert', -1)
endfunction

function codemp#create(k)
	let l:sid = rpcrequest(s:jobid, "create", a:k)
	echo l:sid
endfunction

function codemp#join(k)
	let l:ret = rpcrequest(s:jobid, "join", a:k)
	echo l:ret
endfunction

function codemp#startcursor(k)
	call rpcrequest(s:jobid, "cursor-start", a:k)
endfunction

function codemp#cursor()
	let l:position = getpos('.')
	call rpcnotify(s:jobid, "cursor", 0, l:position[1], l:position[2])
endfunction

function s:OnStderr(id, data, event) dict
	let g:msg = 'codemp: stderr: ' . join(a:data, "\n")
	echo g:msg
endfunction
