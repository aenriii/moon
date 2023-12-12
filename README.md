# `moon` - a simple cli for [moonlight](https://github.com/moonlight-mod/moonlight)

## usage

### `moon up`

This is the injection/reinjection command, supply
`-b|--branch [stable|ptb|canary|development]` to change the branch injected
from the default (stable).

### `moon down`

This is the uninject command, supply `-b|--branch [stable|ptb|canary|development]`
to change the branch injected from the default (stable).

### `moon dev`

This is the development/watch command, it'll detect whether the folder you selected is
a moonlight-mod git repo or a moonlight extension repo (based on [the template](https://github.com/moonlight-mod/sample-extension))

