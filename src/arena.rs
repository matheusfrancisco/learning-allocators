// An arena is  essentially a way to group up allocations that are expected to have the same
// lifetime. Sometimes you need to allocate a bunch of objects for the lifetime of an event,
// after which they can all be thrown away wholesale. It's inefficient to call into the system
// allocator each time, and far more preferable to preallocate a bunch of memory for your objects,
// clenaning it all up at once you are done with them.
//
// Use arena for reduce the allocation pressure
// for example in a game application there may be large mishmash of per-frame-tick
// objects that need to get allocated each frame, and then thrown away. This is extremely
// common in game development in particular, and allocator pressure is somehting gamedevs tend
// to care about.
// This has additional benefits of cache locality: you can ensure that most of the per-frame
// obejcts (which are likely use more ofthen than other objects) are usually in cache during
// the frame, since they have been allocated adjacently
//
//Another goal might be that you want to write self referential data, like a complex graph with cycles,
//that can get cleaned up all at once. For example, when writting compilers, type information will
//likely need to reference other types and other such data, leading to a complex, potentially
//cyclic graph ot types.

use std::cell::RefCell;

// areana social network
pub struct Arena<'arena> {
    people: Vec<Box<Person<'arena>>>,
}

struct Person<'arena> {
    pub follows: RefCell<Vec<PersonRef<'arena>>>, // now RefCell
    pub reverse_follows: RefCell<Vec<PersonRef<'arena>>>,
    pub name: &'static str,
}

type PersonRef<'arena> = &'arena Person<'arena>;

impl<'arena> Arena<'arena> {
    pub fn new() -> Arena<'arena> {
        Arena { people: Vec::new() }
    }

    pub fn get(&'arena self, idx: usize) -> PersonRef<'arena> {
        &self.people[idx]
    }

    pub fn link_reverse_follows(&'arena self) {
        for person in &self.people {
            let me: PersonRef<'arena> = &**person;
            for friend in me.follows.borrow().iter() {
                friend.reverse_follows.borrow_mut().push(me);
            }
        }
    }

    // Returns an index instead of a reference, sidestepping the lifetime conflict
    pub fn add_person(&mut self, name: &'static str, follows: Vec<PersonRef<'arena>>) -> usize {
        let idx = self.people.len();
        self.people.push(Box::new(Person {
            name,
            follows: follows.into(),
            reverse_follows: Default::default(),
        }));
        idx
    }

    pub fn dump(&'arena self) {
        for thing in &self.people {
            println!("{} network:", thing.name);
            println!("\tfollowing:");
            for friend in thing.follows.borrow().iter() {
                println!("\t\t{}", friend.name);
            }
            println!("\tfollowers:");
            for friend in thing.reverse_follows.borrow().iter() {
                println!("\t\t{}", friend.name);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_allocations() {
        let mut arena = Arena::new();
        // Phase 1: insert all nodes, collect indices
        let alice_idx = arena.add_person("Alice", vec![]);
        let bob_idx = arena.add_person("Bob", vec![]);
        let carol_idx = arena.add_person("Carol", vec![]);

        // Phase 2: now arena is done growing — safe to take &'arena refs
        let alice = arena.get(alice_idx);
        let bob = arena.get(bob_idx);
        let carol = arena.get(carol_idx);

        // Phase 3: wire up follows (needs &'arena refs, but no more mutation)
        bob.follows.borrow_mut().push(alice);
        carol.follows.borrow_mut().push(alice);
        carol.follows.borrow_mut().push(bob);

        // Phase 4: build reverse edges
        arena.link_reverse_follows();
        arena.dump();
    }
}
