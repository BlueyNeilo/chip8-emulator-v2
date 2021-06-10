use constants::*;

pub trait CommandEmulator {
    fn get_commands(&mut self) -> &mut CommandRouter;

    fn process_inbound_command(&mut self, &Command);
    
    fn process_inbound_commands(&mut self) {
        self.get_commands()
            .consume_all_inbound()
            .iter()
            .for_each(|c| self.process_inbound_command(c))
    }
    
    fn emulate_cycle(&mut self);
}

pub struct CommandRouter {
    inbound_queue: Queue<Command>,
    outbound_queue: Queue<Command>
}

impl CommandRouter {
    pub fn new() -> Self {
        CommandRouter {
            inbound_queue: Queue::<Command>::new(),
            outbound_queue: Queue::<Command>::new(),
        }
    }

    pub fn send_inbound(&mut self, command: Command) {
        self.inbound_queue.push(command)
    }

    pub fn send_outbound(&mut self, command: Command) {
        self.outbound_queue.push(command)
    }

    pub fn consume_all_inbound(&mut self) -> Vec<Command> {
        self.inbound_queue.remove_all()
    }

    pub fn consume_all_outbound(&mut self) -> Vec<Command> {
        self.outbound_queue.remove_all()
    }
}

pub struct Queue<T> {
    queue: Vec<T>
}

impl <T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            queue: Vec::new()
        }
    }

    pub fn remove_all(&mut self) -> Vec<T> {
        self.queue.drain(..)
            .collect()
    }

    pub fn push(&mut self, sent: T) {
        self.queue.push(sent)
    }
}

pub enum Command {
    Memory(MemoryCommand),
    Audio(AudioCommand),
    Display(DisplayCommand),
    Key(KeyCommand),
    GameState(GameCommand),
}

pub enum MemoryCommand {
    SendRAM([u8; RAM_BYTES])
}

pub enum AudioCommand {
    Play,
    Pause
}

pub enum DisplayCommand {
    SendPixels([bool; N]),
    SendDraw,
    SendClearDisplay
}

pub enum KeyCommand {
    KeyDownUp(usize, bool)
}

pub enum GameCommand {
    Exit
}
