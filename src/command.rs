use constants::*;

pub trait CommandInterpreter {
    fn read_commands(&mut self);
}

pub struct CommandInterface {
    pub input_stack: CommandStack,
    pub output_stack: CommandStack
}

impl CommandInterface {
    pub fn new() -> Self {
        CommandInterface {
            input_stack: CommandStack::new(),
            output_stack: CommandStack::new(),
        }
    }
}

pub struct CommandStack {
    stack: Vec<Command>
}

impl CommandStack {
    pub fn new() -> Self {
        CommandStack {
            stack: Vec::new()
        }
    }

    pub fn pop_all(&mut self) -> Vec<Command> {
        self.stack.drain(..)
            .collect()
    }

    pub fn push(&mut self, sent: Command) {
        self.stack.push(sent)
    }
}

pub enum Command {
    Memory(MemoryCommand),
    Audio(AudioCommand),
    Display(DisplayCommand),
    Key(KeystrokeCommand),
    Chip8(Chip8Command)
}

pub enum MemoryCommand {
    SendMemory([u8; N])
}

pub enum AudioCommand {
    Play,
    Pause
}

pub enum DisplayCommand {
    SendPixels([bool; N])
}

pub enum KeystrokeCommand {
    GetKeys,
    SendKeys([bool; 0x10])
}

pub enum Chip8Command {
    SendDraw,
    SendClearDisplay
}