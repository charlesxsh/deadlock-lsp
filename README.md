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

### Install deadlock-lsp and lauch the plugin

(a) We first need to download and build deadlock-lsp. 
```
$ git clone https://github.com/charlesxsh/deadlock-lsp.git
$ cd deadlock-lsp
$ cargo build
$ cd editors/code
$ npm i
```

(b) We then add a directory named ".vscode" under the root directory of the project we will edit and 
create a file named "setting.json" under the directory. 
The content of "setting.json" is shown as follows:

```
{
    "rust-deadlock-dectector": {
        "serverPath": "${PATH_TO_THE_EXECUTABLE_OF_DEADLOCK-LSP}",
        "dyldLibPath": "${PATH_TO_RUSTUP}/toolchains/nightly-2022-06-14-x86_64-apple-darwin/lib",
        "luckbud": "${PATH_TO_THE_EXECUTABLE_OF_LOCKBUD}""
    }
}

```
serverPath is the path of the compiled language server. 

The value of dyldLibPath can be figured out by running command "cargo rustc -Zunstable-options --print  sysroot"
under the root directory of lockbud and appending "/lib" to the command output. 

luckbud is the path of the compiled lockbud. 

(c) In the end, we use VScode to open the folder of deadlock-lsp, click the "Run and Debug" button on the left, and click 
the "Start Debugging" button to "Run Extension". A new VSCode window is poped out. The plugin is enabled in the window
and we can use it to develop the project with ".vscode/setting.json". 

## Demonstration

We record a demonstration video to showcase the installation and the usage. 
The video is relased on [youtube](https://www.youtube.com/watch?v=FidSnF_I2uE).

## License
This Project is licensed under BSD-3-Clause-Clear license.