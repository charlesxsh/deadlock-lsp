# deadlock-lsp
Visualize potential deadlocks for Rust. 

This VSCode plugin can visualize the programmer-selected critical section and highlight potential deadlocks in the critical section or in the whole program. [Lockbud](https://github.com/BurtonQin/lockbud) provides the required deadlock detection functionality, and thus it is required by this project.  

## Install

The installation takes two steps, one is to install lockbud, and the other is to add deadlock-lsp as a plugin of your VSCode editor. 

### Install lockbud
Currently, lockbud supports rustc 1.63.0-nightly (1f34da9ec 2022-06-14)
```
$ git clone https://github.com/BurtonQin/lockbud.git
$ cd lockbud
$ rustup component add rust-src
$ rustup component add rustc-dev
$ rustup component add llvm-tools-preview
$ cargo install --path .
```

### Install deadlock-lsp
