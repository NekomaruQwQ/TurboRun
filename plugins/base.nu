#? [[plugins]]
#? name = "time"
#? description = "Measures the execution time of a command and prints it to stderr."

export def noop [command: closure]: nothing -> nothing {
    do $command
}

#? [[plugins]]
#? name = "time"
#? description = "Measures the execution time of a command and prints it to stderr."

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
