# curls
*The Curly Language Server*

Curls is an implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) for Curly, written in Rust. It uses [the Curly frontend](https://github.com/curly-lang/curly-lang/tree/master/src/frontend) to power advanced language features.

Curls is still under active development.

## Installation
### Vim
Install [coc.nvim](https://github.com/neoclide/coc.nvim), do `:CocConfig`, and add the following to your config:
```json
{
    "languageserver": {
        "curly" : {
            "command": "~/curls/target/debug/curls",
            "filetypes": ["curly"]
        }
    }
}
```

### VSCode
Install the [Curly extension for VSCode](https://github.com/curly-lang/curly-vscode)

## Goals
- [ ] Semantic token highlighting
- [ ] Autocomplete
- [x] Diagnostics
- [ ] Go to definition/usages
- [ ] Code folding
- [ ] Hover and type information
