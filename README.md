# poed - a very lightweight poem editor

poed is an incredibly simple text editor aimed at writing poetry. Simplicity is the keyword here: the only controls are 
Escape to exit, Ctrl+S to save and arrow keys to navigate. poed always centers the content and takes up the entire terminal
window; I recommend using it in a fullscreen terminal.

## Usage

poed can be invoked in two ways: with an argument or on its own. With an argument it takes a file path; without one, it starts a new empty buffer.
When saving an empty buffer, the file path will be taken from the first line of the buffer. Upon saving for the first time, that first line will be
removed and the remaining buffer content is what will actually be saved.

## License

MIT.
