
#![allow(mutable_transmutes)]

use crate::config::*;
use crate::neural::network::Network;

use lits::*;

use super::searcher::*;
use super::threadpool::*;

use utils::error::*;
use utils::log;
use utils::notate::Notate;

///
/// The manager for an MCTS search.
///
#[derive(Debug)]
pub struct MCTS 
{
    threadpool: ThreadPool,
    policy: Network,
    config: MCTSConfig
}

impl MCTS 
{
    ///
    /// Gets the currently-set best move from the threadpool;
    /// please make sure that this actually exists before calling 
    /// this method.
    ///
    pub fn best_move (& self) -> Tetromino 
    {
        self.threadpool.best_move.into()
    }

    ///
    /// Returns this manager's configuration.
    ///
    pub fn config (& self) -> MCTSConfig 
    {
        self.config.clone()
    }

    ///
    /// Creates a new MCTS manager.
    ///
    pub fn new (config: Config) -> Result<MCTS>
    {
        let mctsconfig = config.mcts;
        let policy = match config.neural.use_best 
        {
            true  => Network::from_best(& config.neural)?,
            false => Network::from_template(& config.neural)?
        };
        let threadpool = ThreadPool::new(& config);

        let mut mcts = MCTS { config: mctsconfig, policy, threadpool };

        mcts.threadpool.set_num_threads(mctsconfig.num_threads, & mcts.policy);

        Ok(mcts)
    }

    ///
    /// Returns the policy handle, but highly unsafely.
    ///
    pub fn policy (& mut self) -> & mut Network 
    {
        & mut self.policy
    }

    ///
    /// Remembers a state-result pair.
    ///
    pub fn remember (& mut self, board: & Board, outcome: & Outcome)
    {
        self.policy.remember(board, outcome);
    }

    ///
    /// Starts a search on this threadpool, with the given starting position,
    /// optimizing for the given player.
    ///
    pub fn search (& mut self, position: & Board, uci: bool)
    {
        let pool = self.threadpool();
        pool.state = position.clone();

        for handle in pool.threads.iter_mut()
        {
            let thread : & mut Searcher = unsafe { & mut (** (* handle).get()) };
            
            thread.clear();
            thread.initialize(position);
        }

        pool.launch(position);

        if uci 
        {
            log::info!("Sent '= 0 {}'.", self.best_move().notate());
            println!("= 0 {}\n", self.best_move().notate());
        }
    }

    ///
    /// Searches and blocks until the move is found.
    ///
    pub fn search_return (& mut self, position: & Board) -> Tetromino
    { 
        self.search(position, false);
        self.threadpool.wait_for(SearcherEvent::Finish);
        self.best_move()
    }

    ///
    /// Stops an ongoing search early.
    ///
    pub fn stop_early (& mut self)
    {
        self.threadpool().set_stop_requirement(true);
    }

    ///
    /// Returns a non-exclusive-mut reference to the threadpool for use 
    ///
    pub fn threadpool (& mut self) -> & mut ThreadPool
    {
        & mut self.threadpool
    }

    ///
    /// Trains the root model and passes it to each thread.
    ///
    pub fn train (& mut self) 
    {
        self.policy.train();

        self.threadpool.threads.iter_mut()
            .map(|handle| unsafe { & mut (** handle.get()) })
            .for_each(
                |thread|
                {
                    thread.network = self.policy.copy();
                }
            );
    }
}
