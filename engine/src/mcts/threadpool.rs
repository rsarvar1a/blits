
#![allow(mutable_transmutes)]

use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::cell::UnsafeCell;
use std::mem;
use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use super::mcts::*;
use super::searcher::*;
use super::sync::*;

///
/// The memory size of the thread pool structure.
///
pub const THREADPOOL_SIZE : usize = mem::size_of::<ThreadPool>();

///
/// The resource manager for the threads that make up an MCTS search pool.
///
pub struct ThreadPool
{
    pub mcts: * const MCTS,

    pub threads: Vec<UnsafeCell<* mut Searcher>>,
    pub handles: Vec<JoinHandle<()>>,
    
    pub main_cond: Arc<Latch>,
    pub work_cond: Arc<Latch>,

    pub stop: AtomicBool
}

impl ThreadPool 
{
    ///
    /// Attaches a new thread to the thread pool and starts it.
    ///
    pub fn attach_one (& mut self)
    {
        unsafe 
        {
            let mem_layout = Layout::new::<Searcher>();
            let result = alloc_zeroed(mem_layout);
            let searcher_ptr : * mut Searcher = result.cast() as * mut Searcher;

            let searcher_id = self.threads.len();
            let cond_variable = match searcher_id { 0 => & self.main_cond, _ => & self.work_cond }.clone();
            let pool : * mut ThreadPool = (* self.mcts).threadpool();
            let config = (* self.mcts).config();

            ptr::write(searcher_ptr, Searcher::new(pool, config, searcher_id, cond_variable));

            self.threads.push(UnsafeCell::new(searcher_ptr));
            let searcher_handle = SearcherHandle { ptr: UnsafeCell::new(searcher_ptr) };

            let builder = thread::Builder::new()
                .name("SearcherHandle creator".to_owned());

            let handle = builder.spawn_unchecked(
                move || 
                {
                    let s_handle = searcher_handle;
                    let thread = & mut ** s_handle.ptr.get();
                    thread.cond_variable.lock();
                    thread.idle();
                }
            ).unwrap();

            self.handles.push(handle);
        }
    }

    ///
    /// Clears all threads.
    ///
    pub fn clear (& mut self)
    {
        self.threads.iter_mut()
            .map(|handle| unsafe { & mut ** (* handle).get() })
            .for_each(|thread| thread.clear());
    }

    ///
    /// Kills all threads.
    ///
    pub fn kill (& mut self) 
    {
        self.stop.store(true, Ordering::Relaxed);
        self.wait_for(SearcherFilter::All, SearcherEvent::Finish);

        let mut handles = Vec::with_capacity(self.threads.len());

        unsafe 
        {
            self.threads.iter()
                .map(|handle| & (** handle.get()))
                .for_each(|thread| { thread.kill.store(true, Ordering::SeqCst); });

            self.threads.iter()
                .map(|handle| & (** handle.get()))
                .for_each(|thread| { thread.cond_variable.set(); });

            while let Some(handle) = self.handles.pop()
            {
                handles.push(handle.join());
            }

            while let Some(handle) = self.threads.pop()
            {
                let thread : * mut Searcher = * handle.get();
                let pointer : NonNull<u8> = mem::transmute(NonNull::new_unchecked(thread));
                let layout = Layout::new::<Searcher>();
                dealloc(pointer.as_ptr(), layout);
            }
        }
    }

    ///
    /// Creates a new thread pool and attaches the main thread.
    ///
    pub fn new (mcts: & MCTS) -> ThreadPool
    {
        let mut pool = ThreadPool 
        {
            mcts,

            threads: Vec::new(),
            handles: Vec::new(),

            main_cond: Arc::new(Latch::new()),
            work_cond: Arc::new(Latch::new()),

            stop: AtomicBool::new(true)
        };

        // Lock all conditions.

        pool.main_cond.lock();
        pool.work_cond.lock();

        // Create the main thread.

        pool.attach_one();
    
        pool
    }

    ///
    /// Returns the MCTS instance behind this pool.
    ///
    pub fn parent (& mut self) -> & mut MCTS 
    {
        unsafe { mem::transmute(& self.mcts) }
    }

    ///
    /// Unsafely sets the number of threads.
    ///
    pub fn set_num_threads (& mut self, num: usize)
    {
        if num > 0 
        {
            self.wait_for(SearcherFilter::All, SearcherEvent::Finish);
            self.kill();
            for _ in 0 .. num 
            {
                self.attach_one();
            }
        }
    }

    ///
    /// Sets the stop state on the thread pool.
    ///
    pub fn set_stop_requirement (& mut self, to: bool)
    {
        self.stop.store(to, Ordering::Relaxed);
    }

    ///
    /// Waits for the conditions of every thread matching the filter 
    /// to evaluate to the same value.
    ///
    pub fn wait_for (& mut self, filter: SearcherFilter, event: SearcherEvent)
    {
        self.threads.iter()
            .map(|handle| unsafe { & (** handle.get()) })
            .filter(|thread| filter.matches(thread.id))
            .for_each(|thread| { thread.search_status.wait(event.clone().into()); });
    }
}
