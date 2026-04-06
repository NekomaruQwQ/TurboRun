#[doc("Measures the execution time of a command and prints it to stderr.")]
#[var("unit")]

let __start = date now
do {{{command}}}
let __end = date now
let __elapsed = $__end - $__start
let __unit = "{{unit}}"
print --stderr $"finished in ($__elapsed | format duration $__unit)"
