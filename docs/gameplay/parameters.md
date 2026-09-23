# Parameters and inputs

## Parameters

The parameter is text, converted when the input is called. A plain value becomes the first of these that fits: an
integer, a float, `true` or `false`, three numbers as a `Vector3`, else a string.

A JSON array like `[10, "$activator"]` spreads into several arguments. Each value is then converted to the type the
method declares. When the method takes a node before any other argument, a text parameter is resolved as a target, so
`!player` or a targetname can be passed straight in.

| Placeholder | Becomes |
| --- | --- |
| `$activator` | The activator |
| `$self`, `$caller` | The receiving entity |
| `$position` | The receiving entity's global position |
| `$caller_name` | The receiving entity's targetname, else its node name |

Placeholders are worked out on the **receiving** entity, not the one that fired. Inside an array only an element that
is exactly the placeholder is replaced.

## No parameter

With the parameter left empty, the activator goes into the first argument typed as an object, or untyped and named
`activator`, wherever it sits. That is why `take_damage(amount, source)` gets it in `source`. Other values the output
carried fill the remaining arguments in order, so `changed(value)` wired to `set_value` passes the value on.

This forwards `false` too: `switched(false)` wired to a counter's `add` adds 0. Set an explicit parameter when you want
a fixed value.

## How an input is found

1. A method of that name, then its PascalCase and camelCase spellings, so a C# `AddScore` answers to `add_score`.
2. A property of that name, set to the parameter or the value the output carried. The input `speed` with parameter `4`
   changes a door's speed.
3. One of the built-in inputs below.

| Input | Effect |
| --- | --- |
| `kill` | Frees the node |
| `show`, `enable` | Makes it visible and processing |
| `hide`, `disable` | Makes it invisible and stops processing |
| `toggle` | Flips visibility |

The entity's own methods come first, which is why `disable` on a trigger switches the trigger off instead of hiding it.
For `show` and `hide` only a method in the entity's script counts, Godot's own `Node3D.show()` is skipped so the
built-in version can stop processing too. Anything else prints `[GT I/O] <node> has no input '<name>'`.
