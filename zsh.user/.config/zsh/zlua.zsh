ZLUA_SCRIPT="/home/pu/Source/z.lua/z.lua"
ZLUA_LUAEXE="/usr/sbin/lua"

_zlua() {
	local arg_mode=""
	local arg_type=""
	local arg_subdir=""
	local arg_inter=""
	local arg_strip=""
	if [ "$1" = "--add" ]; then
		shift
		_ZL_RANDOM="$RANDOM" "$ZLUA_LUAEXE" "$ZLUA_SCRIPT" --add "$@"
		return
	elif [ "$1" = "--complete" ]; then
		shift
		"$ZLUA_LUAEXE" "$ZLUA_SCRIPT" --complete "$@"
		return
	fi
	while [ "$1" ]; do
		case "$1" in
			-l) local arg_mode="-l" ;;
			-e) local arg_mode="-e" ;;
			-x) local arg_mode="-x" ;;
			-t) local arg_type="-t" ;;
			-r) local arg_type="-r" ;;
			-c) local arg_subdir="-c" ;;
			-s) local arg_strip="-s" ;;
			-i) local arg_inter="-i" ;;
			-I) local arg_inter="-I" ;;
			-h) local arg_mode="-h" ;;
			--help) local arg_mode="-h" ;;
			--purge) local arg_mode="--purge" ;;
			*) break ;;
		esac
		shift
	done
	if [ "$arg_mode" = "-h" ] || [ "$arg_mode" = "--purge" ]; then
		"$ZLUA_LUAEXE" "$ZLUA_SCRIPT" $arg_mode
	elif [ "$arg_mode" = "-l" ] || [ "$#" -eq 0 ]; then
		"$ZLUA_LUAEXE" "$ZLUA_SCRIPT" -l $arg_subdir $arg_type $arg_strip "$@"
	elif [ -n "$arg_mode" ]; then
		"$ZLUA_LUAEXE" "$ZLUA_SCRIPT" $arg_mode $arg_subdir $arg_type $arg_inter "$@"
	else
		local zdest=$("$ZLUA_LUAEXE" "$ZLUA_SCRIPT" --cd $arg_type $arg_subdir $arg_inter "$@")
		if [ -n "$zdest" ] && [ -d "$zdest" ]; then
			if [ -z "$_ZL_CD" ]; then
				builtin cd "$zdest"
			else
				$_ZL_CD "$zdest"
			fi
			if [ -n "$_ZL_ECHO" ]; then pwd; fi
		fi
	fi
}
# alias ${_ZL_CMD:-z}='_zlua 2>&1'
alias ${_ZL_CMD:-z}='_zlua'

_zlua_precmd() {
	(_zlua --add "${PWD:a}" &)
}
typeset -ga chpwd_functions
[ -n "${chpwd_functions[(r)_zlua_precmd]}" ] || {
	chpwd_functions[$(($#chpwd_functions+1))]=_zlua_precmd
}

_zlua_zsh_tab_completion() {
	# tab completion
	(( $+compstate )) && compstate[insert]=menu # no expand
	local -a tmp=(${(f)"$(_zlua --complete "${words/_zlua/z}")"})
	_describe "directory" tmp -U
}
if [ "${+functions[compdef]}" -ne 0 ]; then
	compdef _zlua_zsh_tab_completion _zlua 2> /dev/null
	compdef ${_ZL_CMD:-z}=_zlua
fi

zlua_fzf="fzf --nth 2.. --reverse --info=inline --tac  --height 35%  +s -e "
if command -v fzf >/dev/null 2>&1; then
	# To redraw line after fzf closes (printf '\e[5n')
	bindkey '\e[0n' kill-whole-line
	_zlua_zsh_fzf_complete() {
		local list=$(_zlua -l ${words[2,-1]})

		if [ -n "$list" ]; then
			local selected=$(print $list | ${=zlua_fzf} | sed 's/^[0-9,.]* *//')

			if [ -n "$selected" ]; then
				cd ${selected}
				printf '\e[5n'
			fi
		fi
	}

	if [ "${+functions[compdef]}" -ne 0 ]; then
		compdef _zlua_zsh_fzf_complete _zlua > /dev/null 2>&1
		compdef ${_ZL_CMD:-z}=_zlua
	fi
fi

export _ZL_MATCH_MODE=1
