
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'mdget' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'mdget'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'mdget' {
            [CompletionResult]::new('--home', '--home', [CompletionResultType]::ParameterName, 'home')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'l')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'log-level')
            [CompletionResult]::new('--log-file-enable', '--log-file-enable', [CompletionResultType]::ParameterName, 'log-file-enable')
            [CompletionResult]::new('--log-file-path', '--log-file-path', [CompletionResultType]::ParameterName, 'log-file-path')
            [CompletionResult]::new('--log-file-level', '--log-file-level', [CompletionResultType]::ParameterName, 'log-file-level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('command1', 'command1', [CompletionResultType]::ParameterValue, 'Run command1')
            [CompletionResult]::new('command2', 'command2', [CompletionResultType]::ParameterValue, 'Run command2')
            [CompletionResult]::new('fetch', 'fetch', [CompletionResultType]::ParameterValue, 'Fetch URL and convert to markdown')
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Generate shell completion script')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'mdget;command1' {
            [CompletionResult]::new('--home', '--home', [CompletionResultType]::ParameterName, 'home')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'l')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'log-level')
            [CompletionResult]::new('--log-file-enable', '--log-file-enable', [CompletionResultType]::ParameterName, 'log-file-enable')
            [CompletionResult]::new('--log-file-path', '--log-file-path', [CompletionResultType]::ParameterName, 'log-file-path')
            [CompletionResult]::new('--log-file-level', '--log-file-level', [CompletionResultType]::ParameterName, 'log-file-level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'mdget;command2' {
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'An argument for command2')
            [CompletionResult]::new('--arg', '--arg', [CompletionResultType]::ParameterName, 'An argument for command2')
            [CompletionResult]::new('--home', '--home', [CompletionResultType]::ParameterName, 'home')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'l')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'log-level')
            [CompletionResult]::new('--log-file-enable', '--log-file-enable', [CompletionResultType]::ParameterName, 'log-file-enable')
            [CompletionResult]::new('--log-file-path', '--log-file-path', [CompletionResultType]::ParameterName, 'log-file-path')
            [CompletionResult]::new('--log-file-level', '--log-file-level', [CompletionResultType]::ParameterName, 'log-file-level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'mdget;fetch' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Output to file instead of stdout')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Output to file instead of stdout')
            [CompletionResult]::new('--max-body-words', '--max-body-words', [CompletionResultType]::ParameterName, 'Hard cap the rendered body to this many words')
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'Request timeout in seconds')
            [CompletionResult]::new('--max-redirects', '--max-redirects', [CompletionResultType]::ParameterName, 'Maximum redirects to follow')
            [CompletionResult]::new('--user-agent', '--user-agent', [CompletionResultType]::ParameterName, 'User-Agent header')
            [CompletionResult]::new('-H', '-H ', [CompletionResultType]::ParameterName, 'Repeatable header Name:Value')
            [CompletionResult]::new('--header', '--header', [CompletionResultType]::ParameterName, 'Repeatable header Name:Value')
            [CompletionResult]::new('--cookie', '--cookie', [CompletionResultType]::ParameterName, 'Repeatable cookie Name=Value')
            [CompletionResult]::new('--bearer', '--bearer', [CompletionResultType]::ParameterName, 'Bearer token authorization')
            [CompletionResult]::new('--home', '--home', [CompletionResultType]::ParameterName, 'home')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'l')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'log-level')
            [CompletionResult]::new('--log-file-enable', '--log-file-enable', [CompletionResultType]::ParameterName, 'log-file-enable')
            [CompletionResult]::new('--log-file-path', '--log-file-path', [CompletionResultType]::ParameterName, 'log-file-path')
            [CompletionResult]::new('--log-file-level', '--log-file-level', [CompletionResultType]::ParameterName, 'log-file-level')
            [CompletionResult]::new('--compact', '--compact', [CompletionResultType]::ParameterName, 'Reduce body output to headings and short paragraph summaries')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as structured JSON envelope')
            [CompletionResult]::new('--no-frontmatter', '--no-frontmatter', [CompletionResultType]::ParameterName, 'Output markdown body only without frontmatter')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'mdget;completion' {
            [CompletionResult]::new('--shell', '--shell', [CompletionResultType]::ParameterName, 'The shell to generate the completions for')
            [CompletionResult]::new('--home', '--home', [CompletionResultType]::ParameterName, 'home')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'l')
            [CompletionResult]::new('--log-level', '--log-level', [CompletionResultType]::ParameterName, 'log-level')
            [CompletionResult]::new('--log-file-enable', '--log-file-enable', [CompletionResultType]::ParameterName, 'log-file-enable')
            [CompletionResult]::new('--log-file-path', '--log-file-path', [CompletionResultType]::ParameterName, 'log-file-path')
            [CompletionResult]::new('--log-file-level', '--log-file-level', [CompletionResultType]::ParameterName, 'log-file-level')
            break
        }
        'mdget;help' {
            [CompletionResult]::new('command1', 'command1', [CompletionResultType]::ParameterValue, 'Run command1')
            [CompletionResult]::new('command2', 'command2', [CompletionResultType]::ParameterValue, 'Run command2')
            [CompletionResult]::new('fetch', 'fetch', [CompletionResultType]::ParameterValue, 'Fetch URL and convert to markdown')
            [CompletionResult]::new('completion', 'completion', [CompletionResultType]::ParameterValue, 'Generate shell completion script')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'mdget;help;command1' {
            break
        }
        'mdget;help;command2' {
            break
        }
        'mdget;help;fetch' {
            break
        }
        'mdget;help;completion' {
            break
        }
        'mdget;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
