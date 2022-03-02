
#![allow(mutable_transmutes)]

use crate::config::*;
use crate::neural::network::Network;

use lits::*;

use std::mem;
use std::ptr;
use std::sync::atomic::Ordering;
use std::thread;

use super::searcher::*;
use super::threadpool::*;

use utils::error::*;
use utils::notate::Notate;

///
/// The manager for an MCTS search.
///
pub struct MCTS 
{
    threadpool: [u8; THREADPOOL_SIZE],
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
        unsafe { & ** self.threadpool().threads[0].get() }.best_move.clone().into()
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
            true  => Network::from_best(& config.neural),
            false => Network::from_template(& config.neural)
        }?;
        let poolarray = [0; THREADPOOL_SIZE];

        let mcts = MCTS { config: mctsconfig, policy, threadpool: poolarray };

        mcts.threadpool_initialize();
        mcts.threadpool().set_num_threads(mctsconfig.num_threads);

        Ok(mcts)
    }

    ///
    /// Returns the policy handle, but highly unsafely.
    ///
    pub fn policy (& self) -> & mut Network 
    {
        unsafe { mem::transmute::<& Network, & mut Network>(& self.policy) }
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

        pool.wait_for(SearcherFilter::All, SearcherEvent::Finish);
        pool.stop.store(false, Ordering::Relaxed);

        for handle in pool.threads.iter_mut()
        {
            let thread : & mut Searcher = unsafe { & mut (** (* handle).get()) };
            
            thread.clear();
            thread.initialize(position);
        }

        pool.main_cond.set();
        pool.wait_for(SearcherFilter::Main, SearcherEvent::Start);
        pool.main_cond.lock();

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
        pool.wait_for(SearcherFilter::All, SearcherEvent::Finish);
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
            mem::transmute::<& [u8; THREADPOOL_SIZE], & mut ThreadPool>(
                & self.threadpool
            )
        }
    }

    ///
    /// Initializes the threadpool.
    ///
    pub fn threadpool_initialize (& self)
    {
        unsafe 
        {
            let builder = thread::Builder::new()
                .name("ThreadPool creator".to_owned());

            let handle = builder.spawn_unchecked(
                move ||
                {
                    let pool: * mut ThreadPool = mem::transmute::<& [u8; THREADPOOL_SIZE], & mut ThreadPool>(& self.threadpool);
                    ptr::write(pool, ThreadPool::new(self));
                }
            );

            handle.unwrap().join().unwrap();
        }
    }
}
