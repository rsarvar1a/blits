
use std::sync::{Condvar, Mutex};

///
/// Represents a waitable boolean latch on which you can 
/// wait for a particular boolean state.
///
pub struct Guard
{
    mutex: Mutex<bool>,
    latch: Condvar
}

impl Guard
{
    ///
    /// Creates a new latch.
    ///
    pub fn new (value: bool) -> Guard 
    {
        Guard { mutex: Mutex::new(value), latch: Condvar::new() }
    }

    ///
    /// Sets the latch to be true.
    ///
    pub fn set (& self, value: bool)
    {
        let mut guard = self.mutex.lock().unwrap();
        (* guard) = value;
        self.latch.notify_all();
    }

    ///
    /// Waits for the latch to be set.
    ///
    pub fn wait (& self, value: bool)
    {
        let mut guard = self.mutex.lock().unwrap();
        while (* guard) != value
        {
            guard = self.latch.wait(guard).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Latch 
{
    mutex: Mutex<bool>,
    latch: Condvar
}

impl Latch 
{
    ///
    /// Locks this latch, setting it to false.
    ///
    pub fn lock (& self)
    {
        let mut guard = self.mutex.lock().unwrap();
        (* guard) = false;
    }

    ///
    /// Creates a new latch.
    ///
    pub fn new () -> Latch
    {
        Latch { mutex: Mutex::new(false), latch: Condvar::new() }
    }

    ///
    /// Sets the latch to be true.
    ///
    pub fn set (& self)
    {
        let mut guard = self.mutex.lock().unwrap();
        (* guard) = true;
        self.latch.notify_all();
    }

    ///
    /// Waits for the latch to be set.
    ///
    pub fn wait (& self)
    {
        let mut guard = self.mutex.lock().unwrap();
        while ! (* guard) 
        {
            guard = self.latch.wait(guard).unwrap();
        }
    }

}
