## A TUI calculator written in Rust

Built on the [num_parser](https://docs.rs/num_parser/latest/num_parser/#) crate on the backend, and [ratatui](https://github.com/ratatui-org/ratatui) on the frontend.

### How to use

Moving a release build into `usr/local/bin` allows this to be called straight from the terminal. The included `builder` script contains the following which will build and alias it for you.

```zsh
dest="/usr/local/bin/calc"
cargo build --release
cp ./target/release/rust_calc $dest
```

Then either type `calc "sin(pi/3)"` or any similar query to get instant answers, or `calc` on its own to get the full TUI.

### Features

- Make use of most common operations and functions, +,-,/, \* as well as sin(), exp(),...
- Support for complex numbers
- Store and see previous queries and their answers
- Use arrow keys to scroll to previous queries or within the current one
- Define custom variables and functions, try "a = sin(pi/17)", "f(x) = exp(-2x)", "f(a)"
- Set a custom precision and radian or degree input for angle-based functions.

### Todo

- [ ] More testing
- [ ] Resizable panes and better design
- [ ] Streamline some parts of the code - history could be stored better
