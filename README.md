![onlang logo](static/logos/OnLang-transparent.png)

[![Crates.io](https://img.shields.io/crates/v/onlang?style=flat-square)](https://crates.io/crates/onlang)

# You may have questions

ONLang - Object Notation Language (js`ON`)
VSCode extension - [OnLang](https://marketplace.visualstudio.com/items?itemName=artegoser.onlang)

## 1. God, what the f\*\*\*\* is this

ONLang is an experimental, esoteric programming language, that allows you to use (**json, json5 or yaml**) for **PROGRAMMING**.

## 2. What is it for

For writing simple scripts.

## 3. How to write in this language

Variants of the "Hello World!"

on json5

```json5
[
  "Hello world!",
  ["Hello", " world!" ]
  {println:"Hello world!"},
  {println:["Hello world!"]},
]
```

on yaml

```yaml
main:
  - Hello world!
  - - Hello
    - " world!"
  - println: Hello world!
  - println:
      - Hello world!
```

Check the [Documentation](doc/main.md)

## Installation

`cargo install onlang`  
or  
Get binary from [releases](https://github.com/artegoser/ONLang/releases) and add the executable file to the PATH variable

## Using

`on examples/example.json5`

or

1. Clone this repo
2. `cargo run --quiet --release -- examples/example.json5`

If you want to help create a pull request

## License

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fartegoser%2FONLang.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fartegoser%2FONLang?ref=badge_large)
