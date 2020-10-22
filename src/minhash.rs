//! This module implementes original minhash algorithm and is highly inspired by the finch module.
//! The implementation is more generic as it was designed to hash various type of compressed Kmers or 
//! in fact any type T that satisfies Hash+Clone+Copy
//! Moreover it can just computes Jaccard estimate or keep track of objects hashed.
//! 
//! See also module invhash to keep track of objects hashed with inversible hash

//


// with the inspiration of the  finch module for NoHashHasher.
// perhaps use a bloomfilter instead of Hashmap. Especially if counts are not used in jaccard estimate.


// random generation numbers by decreasing speed XorShiftRng, ChaChaRng Isaac64Rng
// The individual items to store in the BinaryHeap



#[allow(unused_imports)]
use probminhash::invhash;

#[allow(unused_imports)]
use log::{debug, trace};

use std::cmp::Ordering;

use std::collections::{BinaryHeap, HashMap};

use std::hash::{BuildHasher, BuildHasherDefault, Hasher, Hash};

use std::fmt::Debug;



/// We use maximum size to store hash value but with invertible 32 hash
/// the value stored is in fact a u32.
/// We would like to template over item hash but Hasher has u64 as arrival type
pub type ItemHash = u64;



// If we use an inversible hash we do not need to keep item (the kmer)
// for other hash  we need copying and storing of Kmer... whence the Option<T> field

/// A HashedItem is a hashed item and possibly the associated object (of type T) if
/// we want to keep track of objects contributiong to minhash signature.
/// This can be useful in genomics. Note that using invertible hash if objects hashes 
/// are stored in a u32 or a u64 (as in some Kmer representation) we can retrive objects
/// from hashed value. (See module invhash)
#[derive(Debug,Clone,Copy)]
pub struct HashedItem<T:Clone+Hash> {
    hash: ItemHash,
    item: Option<T>,
}

impl<T:Hash+Clone+Copy> PartialEq for HashedItem<T> {
    fn eq(&self, other: &HashedItem<T>) -> bool {
        other.hash.eq(&self.hash)
    }
}

impl<T:Clone+Copy+Hash> Eq for HashedItem<T> {}

impl<T:Clone+Hash+Copy> Ord for HashedItem<T> {
    fn cmp(&self, other: &HashedItem<T>) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl<T:Clone+Hash+Copy> PartialOrd for HashedItem<T> {
    fn partial_cmp(&self, other: &HashedItem<T>) -> Option<Ordering> {
        Some(self.hash.cmp(&other.hash))
    }
}



// size is 2*8+2 bytes !!
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashCount<T:Clone+Copy+Hash> {
    pub hashed: HashedItem<T>,
    pub count: u16,
}

/// result of minhash distance computations a tuple for containment, jaccard, common, total
pub struct MinHashDist(pub f64, pub f64, pub u64, pub u64);

pub struct MinHashCount<T: Hash+Clone+Debug, H: Hasher+Default> {
    keep_item:bool,
    hashes: BinaryHeap<HashedItem<T>>,
    b_hasher: BuildHasherDefault<H>,
    counts: HashMap<ItemHash, u16, BuildHasherDefault<H>>,
    total_count: u64,
    size: usize,
    // heap_lock: Mutex<()>,
    // instead of map_lock, look into using https://docs.rs/chashmap/2.2
    // map_lock: Mutex<()>,
}



impl <T:Hash + Clone + Copy + Debug ,  H : Hasher+Default> MinHashCount<T, H> {
    /// an allocator , size is capacity measured as  max number of hashed item
    /// keep_item is to ask( or not) to keep the objects (kmers) hashed.
    /// if using an invertible hasher for compressed kmers we do not need to keep track of kmers
    /// as they can be recovered from hashed values.
    pub fn new(size: usize, keep_item: bool) -> Self {
        MinHashCount {
            keep_item: keep_item,
            b_hasher: BuildHasherDefault::<H>::default(),
            hashes: BinaryHeap::with_capacity(size + 1),
            counts: HashMap::with_capacity_and_hasher(size, BuildHasherDefault::<H>::default()),
            total_count: 0,
            size: size,
            // heap_lock: Mutex::new(()),
            // map_lock: Mutex::new(()),
        }
    }  // end of new


    /// push an item in the sketching
    pub fn push(&mut self, item : &T) {
        //
        // hash
        let mut hasher = self.b_hasher.build_hasher();
        item.hash(&mut hasher);
        let new_hash : u64 = hasher.finish();
        //
        // trace!(" pushing item {:?}, hash {}", item, new_hash);
        // do we insert
        let add_hash = match self.hashes.peek() {
            None => true,
            Some(old_max_hash) => (new_hash <= (*old_max_hash).hash) || (self.hashes.len() < self.size),
        };
        // if add_hash is true we must insert in hashes, 
        if add_hash {
            self.total_count += 1;
            if self.counts.contains_key(&new_hash) {
                // the item was already seen once.
                // let _lock = self.map_lock.lock().unwrap();
                let count = self.counts.entry(new_hash).or_insert(0u16);
                (*count) += 1;
                // drop(_lock);
            } else {
                // newhash is encountered for the first time
                // let _ = self.heap_lock.lock().unwrap();
                self.hashes.push(HashedItem {
                    hash: new_hash,
                    item: if self.keep_item {
                        Some(item.clone())
                    }
                    else {
                        None
                    }
                });
                // 
                self.counts.insert(new_hash, 1u16);
                if self.hashes.len() > self.size {
                    let hashitem = self.hashes.pop().unwrap();
                    let _old_count = self.counts.remove(&hashitem.hash).unwrap();
                }
                // drop(_lock);
                // drop(_map_lock);
            }
        } // end if add_hash        
    } // end push

    /// push a slice in the sketching
    pub fn sketch_slice(&mut self, to_sketch : &[T]) {
        trace!("sketching slice");
        to_sketch.into_iter().for_each(|x| self.push(x));
    } // end of sketch_slice


    /// returns a sorted vecotr of the sketch
    pub fn get_sketchcount(&self) -> Vec<HashCount<T> > {
        trace!("get_sketchcount  got nb hashes : {} ",self.hashes.len());
        let mut results = Vec::with_capacity(self.hashes.len());
        for item in self.hashes.iter() {
            trace!(" got hash : {:?}", item.hash);
            let counts = *self.counts.get(&item.hash).unwrap();
            let counted_item = HashCount {
                hashed: *item,
                count: counts,
            };
            results.push(counted_item);
        }
        results
    }  // end of get_sketchcount

    /// returns 
    pub fn get_signature(&self) -> Option<&BinaryHeap<HashedItem<T>> > {
        if self.keep_item == true {
            return None;
        }
        else {
            return Some(&self.hashes);
        }
    } // end of get_signature



}  // end of impl MinHashCount



/// compute different distances from sketch. What do we do of counts?
pub fn minhash_distance<T:Hash+Clone+Copy>(sketch1: &Vec<HashCount<T> >, sketch2: &Vec<HashCount<T> >) ->  MinHashDist {
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut common: u64 = 0;
    let mut total: u64 = 0;
    let sketch_size = sketch1.len();
    //
    trace!("sketch1 len : {}, sketch2 len : {}", sketch1.len(), sketch2.len());
    //
    let mut items1 : Vec<HashedItem<T>> = sketch1.iter().map(|x| x.hashed).collect();
    items1.sort_unstable();
    let mut items2 : Vec<HashedItem<T>> = sketch2.iter().map(|x| x.hashed).collect();
    items2.sort_unstable();
    //    
    while i < items1.len() && j < items2.len() {
        if items1[i] < items2[j] {
            i += 1;
        } else if items2[j] < items1[i] {
            j += 1;
        } else {
            i += 1;
            j += 1;
            common += 1;
        }
        total += 1;
        if total >= sketch1.len() as u64 {
            break;
        }
    } // end while
    //
    // try to increase total up to asked sketch size
    //
    if total < items1.len() as u64 {
        // try to increase total.
        if i < items1.len() {
            total += (items1.len() - i) as u64;
        }
        if j < items1.len() {
            total += (items1.len() - j) as u64;
        }
        // now if ever total increase too much we truncate it
        if total > sketch_size as u64 {
            total = sketch_size as u64;
        }            
    }        
    //
    let containment: f64 = common as f64 / i as f64;
    let jaccard: f64 = common as f64 / total as f64;
    MinHashDist(containment, jaccard, common, total)
}  // end of minhash_distance


////////////////////////////////////////////////////////////////////////////////////////:


#[cfg(test)]
mod tests {
    use super::*;
    extern crate fnv;
    #[allow(unused_imports)]
    use self::fnv::FnvHasher; // extern fnv declared in test so we use self::fnv , if declared above we use super::fnv
    #[allow(unused_imports)]
    use crate::nohasher::NoHashHasher;

    #[allow(dead_code)]
    fn log_init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }



    #[test]
    fn test_minhash_count_range_intersection_fnv() {
        log_init();
        // we construct 2 ranges [a..b] [c..d], with a<b, b < d, c<d sketch them and compute jaccard.
        // we should get something like max(b,c) - min(b,c)/ (b-a+d-c)
        //
        let va : Vec<usize> = (0..100).collect();
        let vb : Vec<usize> = (80..160).collect();
        let _bh = BuildHasherDefault::<FnvHasher>::default();
        let mut minhash_a : MinHashCount<usize, FnvHasher>= MinHashCount::new(50, true);
        let mut minhash_b : MinHashCount<usize, FnvHasher>= MinHashCount::new(50, true);
        // now compute sketches
        println!("sketching a ");
        minhash_a.sketch_slice(&va);
        println!("\n \n sketching b ");
        minhash_b.sketch_slice(&vb);
        let sketch_a = minhash_a.get_sketchcount();
        let sketch_b = minhash_b.get_sketchcount();
        // 
        let resdist = minhash_distance(&sketch_a, &sketch_b);
        debug!("distance minhash (contain, dist, common, total):  {}  {}   {}  {} ",
               resdist.0, resdist.1, resdist.2, resdist.3);
        if let Some(opthashes) = minhash_a.get_signature() {
            trace!(" nb objects {} ", opthashes.len());
        }
        else {
            println!("minhash_a.get_signature() returned None");
        }
        // 
        assert!(resdist.2 > 0);
        //
    } // end of test_range_intersection


}  // end of mod test
