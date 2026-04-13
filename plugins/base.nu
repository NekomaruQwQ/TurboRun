#? [[plugins]]
#? name = "noop"
#? description = "Does nothing."
export def noop [command: closure]: nothing -> nothing {
    do $command
}

#? [[plugins]]
#? name = "env"
#? description = "Runs the command with an environment variable"
#?
#? [[plugins.args]]
#? name = "key"
#?
#? [[plugins.args]]
#? name = "value"
export def env [command: closure, --key: string, --value: string]: nothing -> nothing {
    {} | insert $key $value | with-env $in $command
}

#? [[plugins]]
#? name = "repeat"
#? description = "Repeats the command after an interval."
#?
#? [[plugins.args]]
#? name = "interval"
#? optional = true
#? description = "Interval in milliseconds between each execution of the command. Default is 1000ms."
export def repeat [command: closure, --interval: string = "1000"]: nothing -> nothing {
    let interval = $"($interval)ms" | into duration
    loop {
        do $command
        sleep $interval
    }
}

#? [[plugins]]
#? name = "time"
#? description = "Measures the execution time of a command and prints it to stderr."
#?
#? [[plugins.args]]
#? name = "unit"
#? optional = true
export def time [command: closure, --unit: string = "ms"]: nothing -> nothing {
    let start = date now
    do $command
    let end = date now
    let elapsed = $end - $start

    print --stderr $"finished in ($elapsed | format duration $unit)"
}
