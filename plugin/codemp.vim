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

let s:bin = "/home/alemi/projects/codemp/target/debug/codemp-client"

function! codemp#init()
	let result = s:StartJob()

	if 0 == result
		echoerr "codeMP: cannot start rpc process"
	elseif -1 == result
		echoerr "codeMP: rpc process is not executable"
	else
		let s:jobid = result
		call s:ConfigureJob(result)
	endif
endfunction

function! s:StartJob()
	if 0 == s:jobid
		let id = jobstart([s:bin], { 'rpc': v:true, 'on_stderr': function('s:OnStderr') })
		return id
	else
		return 0
	endif
endfunction

function! s:StopJob()
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

function! s:ConfigureJob(jobid)
	augroup codeMp
		" clear all previous autocommands
		autocmd!

		autocmd VimLeavePre * :call s:StopJob()

		autocmd InsertEnter * :call s:NotifyInsertEnter()
		autocmd InsertLeave * :call s:NotifyInsertLeave()

	augroup END
endfunction

function! s:NotifyInsertEnter()
	let [ bufnum, lnum, column, off ] = getpos('.')
	call rpcnotify(s:jobid, 'insert-enter', v:insertmode, lnum, column)
endfunction

function! s:NotifyInsertLeave()
endfunction

function! codemp#ping()
	call rpcnotify(s:jobid, "ping")
endfunction

function! codemp#test()
	call rpcnotify(s:jobid, "rpc")
endfunction

function! s:OnStderr(id, data, event) dict
	echom 'codemp: stderr: ' . join(a:data, "\n")
endfunction
