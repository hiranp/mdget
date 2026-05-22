# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_mdget_global_optspecs
	string join \n home= l/log-level= log-file-enable= log-file-path= log-file-level= h/help V/version
end

function __fish_mdget_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_mdget_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_mdget_using_subcommand
	set -l cmd (__fish_mdget_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c mdget -n "__fish_mdget_needs_command" -l home -r -F
complete -c mdget -n "__fish_mdget_needs_command" -s l -l log-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_needs_command" -l log-file-enable -r -f -a "true\t''
false\t''"
complete -c mdget -n "__fish_mdget_needs_command" -l log-file-path -r -F
complete -c mdget -n "__fish_mdget_needs_command" -l log-file-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_needs_command" -s h -l help -d 'Print help'
complete -c mdget -n "__fish_mdget_needs_command" -s V -l version -d 'Print version'
complete -c mdget -n "__fish_mdget_needs_command" -f -a "command1" -d 'Run command1'
complete -c mdget -n "__fish_mdget_needs_command" -f -a "command2" -d 'Run command2'
complete -c mdget -n "__fish_mdget_needs_command" -f -a "fetch" -d 'Fetch URL and convert to markdown'
complete -c mdget -n "__fish_mdget_needs_command" -f -a "completion" -d 'Generate shell completion script'
complete -c mdget -n "__fish_mdget_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mdget -n "__fish_mdget_using_subcommand command1" -l home -r -F
complete -c mdget -n "__fish_mdget_using_subcommand command1" -s l -l log-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand command1" -l log-file-enable -r -f -a "true\t''
false\t''"
complete -c mdget -n "__fish_mdget_using_subcommand command1" -l log-file-path -r -F
complete -c mdget -n "__fish_mdget_using_subcommand command1" -l log-file-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand command1" -s h -l help -d 'Print help'
complete -c mdget -n "__fish_mdget_using_subcommand command2" -s a -l arg -d 'An argument for command2' -r
complete -c mdget -n "__fish_mdget_using_subcommand command2" -l home -r -F
complete -c mdget -n "__fish_mdget_using_subcommand command2" -s l -l log-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand command2" -l log-file-enable -r -f -a "true\t''
false\t''"
complete -c mdget -n "__fish_mdget_using_subcommand command2" -l log-file-path -r -F
complete -c mdget -n "__fish_mdget_using_subcommand command2" -l log-file-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand command2" -s h -l help -d 'Print help'
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -s o -l output -d 'Output to file instead of stdout' -r -F
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l max-body-words -d 'Hard cap the rendered body to this many words' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l timeout -d 'Request timeout in seconds' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l max-redirects -d 'Maximum redirects to follow' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l user-agent -d 'User-Agent header' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -s H -l header -d 'Repeatable header Name:Value' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l cookie -d 'Repeatable cookie Name=Value' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l bearer -d 'Bearer token authorization' -r
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l home -r -F
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -s l -l log-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l log-file-enable -r -f -a "true\t''
false\t''"
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l log-file-path -r -F
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l log-file-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l compact -d 'Reduce body output to headings and short paragraph summaries'
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l json -d 'Output as structured JSON envelope'
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -l no-frontmatter -d 'Output markdown body only without frontmatter'
complete -c mdget -n "__fish_mdget_using_subcommand fetch" -s h -l help -d 'Print help'
complete -c mdget -n "__fish_mdget_using_subcommand completion" -l shell -d 'The shell to generate the completions for' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c mdget -n "__fish_mdget_using_subcommand completion" -l home -r -F
complete -c mdget -n "__fish_mdget_using_subcommand completion" -s l -l log-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand completion" -l log-file-enable -r -f -a "true\t''
false\t''"
complete -c mdget -n "__fish_mdget_using_subcommand completion" -l log-file-path -r -F
complete -c mdget -n "__fish_mdget_using_subcommand completion" -l log-file-level -r -f -a "trace\t''
debug\t''
info\t''
warn\t''
error\t''"
complete -c mdget -n "__fish_mdget_using_subcommand help; and not __fish_seen_subcommand_from command1 command2 fetch completion help" -f -a "command1" -d 'Run command1'
complete -c mdget -n "__fish_mdget_using_subcommand help; and not __fish_seen_subcommand_from command1 command2 fetch completion help" -f -a "command2" -d 'Run command2'
complete -c mdget -n "__fish_mdget_using_subcommand help; and not __fish_seen_subcommand_from command1 command2 fetch completion help" -f -a "fetch" -d 'Fetch URL and convert to markdown'
complete -c mdget -n "__fish_mdget_using_subcommand help; and not __fish_seen_subcommand_from command1 command2 fetch completion help" -f -a "completion" -d 'Generate shell completion script'
complete -c mdget -n "__fish_mdget_using_subcommand help; and not __fish_seen_subcommand_from command1 command2 fetch completion help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
