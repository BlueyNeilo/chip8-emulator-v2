use constants::*;
use router::Router;

pub trait CommandEmulator {
    fn get_commands(&mut self) -> &mut Router<Command>;

    fn process_inbound_command(&mut self, &Command);
    
    fn process_inbound_commands(&mut self) {
        self.get_commands()
            .consume_all_inbound()
            .iter()
            .for_each(|c| self.process_inbound_command(c))
    }
    
    fn emulate_cycle(&mut self);
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
