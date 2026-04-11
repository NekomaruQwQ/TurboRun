#? [[plugins]]
#? name = "noop"
#? description = "Does nothing."
export def noop [command: closure]: nothing -> nothing {
    do $command
}

#? [[plugins]]
#? name = "env"
#? args = [{ name = "key" }, { name = "value" }]
#? description = "Runs the command with an environment variable"
export def env [command: closure, --key: string, --value: string]: nothing -> nothing {
    {} | insert $key $value | with-env $in $command
}

#? [[plugins]]
#? name = "loop"
#? description = "Restarts the command after an interval."
#?
#? [[plugins.args]]
#? name = "interval"
#? optional = true
#? description = "Interval in milliseconds between each execution of the command. Default is 1000ms."
export def loop [command: closure, --interval: string = "1000"]: nothing -> nothing {
    let interval = $"($interval)ms" | into duration
    loop {
        do $command
        sleep $interval
    }
}

#? [[plugins]]
#? name = "time"
#? args = [{ name = "unit", optional = true }]
#? description = "Measures the execution time of a command and prints it to stderr."
export def time [command: closure, --unit: string = "ms"]: nothing -> nothing {
    let start = date now
    do $command
    let end = date now
    let elapsed = $end - $start

    print --stderr $"finished in ($elapsed | format duration $unit)"
}
