
use builtin;
use str;

set edit:completion:arg-completer[mdget] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'mdget'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'mdget'= {
            cand -H 'H'
            cand --home 'home'
            cand -l 'l'
            cand --log-level 'log-level'
            cand --log-file-enable 'log-file-enable'
            cand --log-file-path 'log-file-path'
            cand --log-file-level 'log-file-level'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand command1 'Run command1'
            cand command2 'Run command2'
            cand fetch 'Fetch URL and convert to markdown'
            cand completion 'Generate shell completion script'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mdget;command1'= {
            cand -H 'H'
            cand --home 'home'
            cand -l 'l'
            cand --log-level 'log-level'
            cand --log-file-enable 'log-file-enable'
            cand --log-file-path 'log-file-path'
            cand --log-file-level 'log-file-level'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'mdget;command2'= {
            cand -a 'An argument for command2'
            cand --arg 'An argument for command2'
            cand -H 'H'
            cand --home 'home'
            cand -l 'l'
            cand --log-level 'log-level'
            cand --log-file-enable 'log-file-enable'
            cand --log-file-path 'log-file-path'
            cand --log-file-level 'log-file-level'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'mdget;fetch'= {
            cand -o 'Output to file instead of stdout'
            cand --output 'Output to file instead of stdout'
            cand --max-body-words 'Hard cap the rendered body to this many words'
            cand --timeout 'Request timeout in seconds'
            cand --max-redirects 'Maximum redirects to follow'
            cand --user-agent 'User-Agent header'
            cand -H 'H'
            cand --home 'home'
            cand -l 'l'
            cand --log-level 'log-level'
            cand --log-file-enable 'log-file-enable'
            cand --log-file-path 'log-file-path'
            cand --log-file-level 'log-file-level'
            cand --compact 'Reduce body output to headings and short paragraph summaries'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'mdget;completion'= {
            cand --shell 'The shell to generate the completions for'
            cand -H 'H'
            cand --home 'home'
            cand -l 'l'
            cand --log-level 'log-level'
            cand --log-file-enable 'log-file-enable'
            cand --log-file-path 'log-file-path'
            cand --log-file-level 'log-file-level'
        }
        &'mdget;help'= {
            cand command1 'Run command1'
            cand command2 'Run command2'
            cand fetch 'Fetch URL and convert to markdown'
            cand completion 'Generate shell completion script'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mdget;help;command1'= {
        }
        &'mdget;help;command2'= {
        }
        &'mdget;help;fetch'= {
        }
        &'mdget;help;completion'= {
        }
        &'mdget;help;help'= {
        }
    ]
    $completions[$command]
}
