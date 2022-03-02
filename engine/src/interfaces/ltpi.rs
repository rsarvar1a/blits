
use crate::config::*;
use crate::mcts::mcts::MCTS;

use lits::{Game, Tetromino};

use utils::error::*;
use utils::log;
use utils::notate::Notate;

///
/// Runs the main loop and interfaces with a controller program.
///
pub struct LTPInterface 
{
    config: Config,
    mcts: MCTS,
    state: Game
}

impl LTPInterface
{
    ///
    /// Halts this engine.
    ///
    pub fn halt (& mut self)
    {
        self.mcts.threadpool().set_stop_requirement(true);
    }

    ///
    /// Creates a new LTP interface.
    ///
    pub fn new (config: & Config) -> Result<LTPInterface>
    {
        let mcts = MCTS::new(config.clone())?;
        Ok(LTPInterface { config: config.clone(), mcts, state: Game::new() })
    }

    ///
    /// Runs the main loop.
    ///
    pub fn run_loop (& mut self) 
    {
        let mut cmdline = String::new();
        loop 
        {
            cmdline.clear();
            std::io::stdin().read_line(& mut cmdline).ok().unwrap();
            let args : Vec<& str> = cmdline.split_whitespace().collect();
            let cmd  : & str = args.first().unwrap_or(& "");

            match cmd 
            {
                "" => continue,

                "initialize" => 
                {
                    log::info!("LTP startup");
                }
                
                "shutdown"   => 
                {
                    self.halt();
                    break;
                }

                "setup-position" => 
                {
                    match Game::parse(& args[1])
                    {
                        Ok(new_game) => { self.state = new_game },
                        Err(e) => log::error!("{}", e)
                    };
                },
                
                "new-game" => 
                {
                    self.state = Game::new();
                },

                "play-move" => 
                {
                    match Tetromino::parse(& args[1])
                    {
                        Ok(tetromino) => 
                        {
                            match self.state.apply(& tetromino)
                            {
                                Ok(()) => {},
                                Err(e) => log::error!("{}", e)
                            }
                        },
                        Err(e) => log::error!("{}", e)
                    };
                },

                "undo-move" => 
                {
                    match self.state.undo()
                    {
                        Ok(()) => {},
                        Err(e) => log::error!("{}", e)
                    };
                },

                "cancel-search" => 
                {
                    self.mcts.stop_early();
                },

                "gen-move" => 
                {
                    self.mcts.search(self.state.get_board(), true);
                },

                "show-board" => 
                {
                    log::info!("{}\n{}", self.state.get_board().notate(), self.state.get_board());
                },

                _ => 
                {
                    log::error!("Unknown command '{}'.", cmd)
                }
            };
        }
    }
}
