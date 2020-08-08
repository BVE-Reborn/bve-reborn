csv-unexpected-end-of-row = Not Enough Arguments
csv-float-parsing-error = Float parsing error "{$error}" in csv column {$column}
csv-int-parsing-error = Integer parsing error "{$error}" in csv column {$column}
csv-bool-parsing-error = Boolean parsing error "{$error}" in csv column {$column}

kvp-unknown-section = Unknown Section: "{$section}"
kvp-unknown-field = Unknown Field: "{$field}"
kvp-too-many-fields = Too Many Values: Value {$number} of {$total}
kvp-invalid-value = Invalid Value: "{$value}"

mesh-warning-useless-instruction = Instruction "{$name}" has no effect
mesh-error-utf8 = UTF-8 error on column {$column}
mesh-error-out-of-bounds = Index {$idx} is out of bounds
mesh-error-unknown-instruction = Unrecognized instruction {$name}
mesh-error-unknown-csv = Unknown error in csv-like parsing

route-preprocessing-malformed-directive = The syntax for preprocessing directive "{$directive}"" is incorrect
route-preprocessing-include-file-not-found = File "{$file}" included is not found
route-preprocessing-include-unreadable = File "{$file}" included cannot be opened due to "{$reason}"
route-preprocessing-random-include-none = Empty include directive
route-preprocessing-random-invalid-weight = List of include weights includes an invalid (weight < 0) weight: {$weights}
route-preprocessing-random-all-zero = List of include weights has all weights set at zero
route-preprocessing-invalid-argument = Invalid argument "{$arg}" provided to the {$directive} directive

route-parse-failure = Route command "{$command}" has invalid syntax

route-command-creation-missing-namespace = Command "{$command}": Executed without a namespace
route-command-creation-missing-index = Command "{$command}": Index #{$idx} is missing
route-command-creation-invalid-index = Command "{$command}": Index #{$idx} is present but invalid
route-command-creation-missing-argument = Command "{$command}": Argument #{$idx} is missing
route-command-creation-invalid-argument = Command "{$command}": Argument #{$idx} is present but invalid
route-command-creation-missing-suffix = Command "{$command}": Suffix is missing
route-command-creation-invalid-suffix = Command "{$command}": Suffix is present but invalid
route-command-creation-unknown-command = Unknown command "{$namespace}.{$name}"
route-command-creation-unknown-command-suffix = Unknown command "{$namespace}.{$name}.{$suffix}"
