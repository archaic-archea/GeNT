# Cached Threads
Cached threads can often be used by an OS when a handler thread is spun up exteremely often. When caching a thread, it's `ThreadData` structure should be saved, and a pointer to it should be stored in the supervisor-level event handler table (more information in `Userspace/Events.md` at the `Supervisor Event Handler Table` section).
It's also valid to immediately cache a thread when a handler for it is made, but that should depend on available RAM, and swap space.  
A cached thread's structure should just contain a copy of the the thread's registers at the start of the event.
When spinning up based off of this, the thread pointer still needs set, and the stack needs set, the rest should be safe to assume already initialized.