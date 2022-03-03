
#![allow(mutable_transmutes)]

use crate::config::*;
use crate::neural::network::Network;

use lits::*;

use std::mem;
use std::sync::atomic::Ordering;

use super::node::MoveID;
use super::searcher::*;
use super::threadpool::*;

use utils::error::*;
use utils::notate::Notate;

///
/// The manager for an MCTS search.
///
pub struct MCTS 
{
    threadpool: ThreadPool,
    policy: Network,
    config: MCTSConfig,
    pub best_move: MoveID
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
        self.best_move.into()
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
        let threadpool = ThreadPool::new();

        let mcts = MCTS { config: mctsconfig, policy, threadpool, best_move: 0 };

        mcts.threadpool().set_parent(& mcts);
        mcts.threadpool().set_num_threads(mctsconfig.num_threads);

        Ok(mcts)
    }

    ///
    /// Returns the policy handle, but highly unsafely.
    ///
    pub fn policy (& self) -> & mut Network 
    {
        unsafe 
        { 
            mem::transmute::<& Network, & mut Network>(
                & self.policy
            ) 
        }
    }

    ///
    /// Remembers a state-result pair.
    ///
    pub fn remember (& self, board: & Board, outcome: & Outcome)
    {
        self.policy().remember(board, outcome);
    }

    ///
    /// Starts a search on this threadpool, with the given starting position,
    /// optimizing for the given player.
    ///
    pub fn search (& self, position: & Board, uci: bool)
    {
        let pool = self.threadpool();
        pool.state = position.clone();

        pool.wait_for(SearcherEvent::Finish);
        pool.stop.store(false, Ordering::Relaxed);

        for handle in pool.threads.iter_mut()
        {
            let thread : & mut Searcher = unsafe { & mut (** (* handle).get()) };
            
            thread.clear();
            thread.initialize(position);
        }

        pool.launch(position);

        if uci 
        {
            println!("= {}", self.best_move().notate());
        }
    }

    ///
    /// Searches and blocks until the move is found.
    ///
    pub fn search_return (& self, position: & Board) -> Tetromino
    {
        let pool = self.threadpool();
        
        self.search(position, false);
        pool.wait_for(SearcherEvent::Finish);
        self.best_move()
    }

    ///
    /// Stops an ongoing search early.
    ///
    pub fn stop_early (& self)
    {
        self.threadpool().set_stop_requirement(true);
    }

    ///
    /// Returns a non-exclusive-mut reference to the threadpool for use 
    ///
    pub fn threadpool (& self) -> & mut ThreadPool
    {
        unsafe 
        {
            mem::transmute::<& ThreadPool, & mut ThreadPool>(
                & self.threadpool
            )
        }
    }
}
