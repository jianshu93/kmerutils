//! This file implements KmerAA representing Kmer for Amino Acid.
//! We use implement compression og base on 5 bits stored in a u128.
//! So KmerAA can store up to 25 AA. For less 12 AA a u64 would be sufficient.
//! The structures found in module base such as KmerSeqIterator and KmerGenerationPattern
//! applies to KmerAA 

#![allow(unused)]


use std::mem;

use std::io;
use std::io::{ErrorKind};

use std::str::FromStr;


use std::cmp::Ordering;
use std::ops::{Range};

use indexmap::{IndexMap};
use fnv::FnvBuildHasher;
type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;


#[allow(unused)]
use log::{debug,info,error};


use crate::base::kmertraits::*;

/// alphabet of RNA. 
pub struct Alphabet {
    pub bases: String,
}

/*
 We keep the 0 bit field

A = 00001
C = 00010
D = 00011
E = 00100
F = 00101
G = 00110
H = 00111
I = 01000
K = 01001
L = 01010
M = 01011
N = 01100
P = 01101
Q = 01111
R = 10000
S = 10001
T = 10010
V = 10011
W = 10100
Y = 10101
*/

impl Alphabet {
    pub fn new() -> Alphabet {
        Alphabet { bases : String::from("ACDEFGHIKLMNPQRSTVWY")}
    }
    //
    pub fn len(&self) -> u8 {
        return self.bases.len() as u8;
    }

    #[inline(always)]
    fn is_valid_base(&self, c: u8) -> bool {
        self.bases.find(c as char).is_some() 
    } // end is_valid_base

    fn get_nb_bits(&self) -> u8 { 
        5
    }

    // encode a base into its bit pattern and returns it in a u8
    fn encode(c : u8) -> u8 {
        match c {
            b'A' => 0b00001,
            b'C' => 0b00010,
            b'D' => 0b00011,
            b'E' => 0b00100,
            b'F' => 0b00101,
            b'G' => 0b00110,
            b'H' => 0b00111,
            b'I' => 0b01000,
            b'K' => 0b01001,
            b'L' => 0b01010,
            b'M' => 0b01011,
            b'N' => 0b01100,
            b'P' => 0b01101,
            b'Q' => 0b01111,
            b'R' => 0b10000,
            b'S' => 0b10001,
            b'T' => 0b10010,
            b'V' => 0b10011,
            b'W' => 0b10100,
            b'Y' => 0b10101,
            _    => panic!("pattern not a code in alpahabet for amino acid"),
        } // end of match
    }   // end of encode


    fn decode(&self, c:u8) -> u8 {
        match c {
            0b00001 => b'A',
            0b00010 => b'C',
            0b00011 => b'D',
            0b00100 => b'E',
            0b00101 => b'F',
            0b00110 => b'G',
            0b00111 => b'H',
            0b01000 => b'I',
            0b01001 => b'K',
            0b01010 => b'L',
            0b01011 => b'M',
            0b01100 => b'N',
            0b01101 => b'P',
            0b01111 => b'Q',
            0b10000 => b'R',
            0b10001 => b'S',
            0b10010 => b'T',
            0b10011 => b'V',
            0b10100 => b'W',
            0b10101 => b'Y',
            _    => panic!("pattern not a code in alpahabet for Amino Acid"),
        }
   }  // end of decode
}  // end of impl Alphabet



#[derive(Copy,Clone,Hash)]
/// We implement Amino Acif Kmer as packed in a u128 using 5bits by base. So we can go up to 25 bases.
pub struct KmerAA {
    aa      : u128,
    nb_base : u8,
 
} // end of struct KmerAA

impl KmerAA {

    pub fn new(nb_base : u8) -> Self {
        if (nb_base >= 25) {
            panic!("For KmerAA nb_base must be less or equal to 25")
        }
        KmerAA{aa:0, nb_base}
    }
}  // end of impl KmerAA



impl KmerT for KmerAA {

    fn get_nb_base(&self) -> u8 {
        self.nb_base
    } // end of get_nb_base

    // 
    fn push(&self, c : u8) -> Self {
        // shift left 5 bits, insert new base and enforce 0 at upper bits
        let value_mask :u128 = (0b1 << (2*self.get_nb_base())) - 1;
        let new_kmer = ((self.aa << 5) & value_mask) | (c as u128 & 0b11111);
        KmerAA{aa:new_kmer, nb_base:self.nb_base}
    }  // end of push

    // TODO
    fn reverse_complement(&self) -> Self {
        panic!("KmerAA reverse_complement not yet implemented");
    } // end of reverse_complement

    fn dump(&self, bufw: &mut dyn io::Write) -> io::Result<usize> {
        bufw.write(unsafe { &mem::transmute::<u8, [u8;1]>(self.nb_base) }).unwrap();
        bufw.write(unsafe { &mem::transmute::<u128, [u8;16]>(self.aa) } )
    } 
     
} // end of impl KmerT block for KmerAA


impl PartialEq for KmerAA {
    // we must check equality of field
    fn eq(&self, other: &KmerAA) -> bool {
        if (self.aa == other.aa) & (self.nb_base ==other.nb_base) { true } else {false}
    }
}  // end of impl PartialEq for KmerAA

impl Eq for KmerAA {}



/// We define ordering as a kind of "lexicographic" order by taking into account first number of base.
/// The more the number of base the greater. Then we have integer comparison between aa parts
/// 
/// 

impl CompressedKmerT for KmerAA {
    type Val = u128;

    fn get_nb_base_max() -> usize { 25}

    /// a decompressing function mainly for test and debugging purpose
    fn get_uncompressed_kmer(&self) -> Vec<u8> {
        let nb_bases = self.nb_base;
        let alphabet = Alphabet::new();
        // we treat each block of 2 bis as u8 end call decoder of Alphabet2b
        let mut decompressed_kmer = Vec::<u8>::with_capacity(nb_bases as usize);
        let mut base:u8;
        //
        let mut buf = self.aa;
        // get the base coding part at left end of u32
        buf = buf.rotate_left((128 - 5 * nb_bases) as u32);
        for _ in 0..nb_bases {
            buf = buf.rotate_left(5);
            base = (buf & 0b11111) as u8; 
            decompressed_kmer.push(alphabet.decode(base));
        }
        return decompressed_kmer;
    }

        /// return the pure value with part coding number of bases reset to 0.
    #[inline(always)]    
    fn get_compressed_value(&self) -> u128 {
        return self.aa;
    }

    #[inline(always)]    
    fn get_bitsize(&self) -> usize { 128 }
}  // end of impl CompressedKmerT for KmerAA


//===================================================================



impl  Ord for KmerAA {

    fn cmp(&self, other: &KmerAA) -> Ordering {
        if self.nb_base != other.nb_base {
            return (self.nb_base).cmp(&(other.nb_base));
        }
        else {
            return (self.aa).cmp(&(other.aa));
        }
    } // end cmp
} // end impl Ord for KmerAA 



impl PartialOrd for KmerAA {
    fn partial_cmp(&self, other: &KmerAA) -> Option<Ordering> {
        Some(self.cmp(other))
    } // end partial_cmp
} // end impl Ord for KmerAA



//=======================================================================

/// our sequence of Amino Acid is encoded on a byte (even if 5 bits are enough but we do not store sequences yet)
// type SequenceAA = Vec<u8>;

pub struct SequenceAA {
    seq: Vec<u8>
}


impl SequenceAA {

    /// allocates and check for compatibility with alphabet
    pub fn new(str: &[u8]) -> Self {
        let alphabet = Alphabet::new();
        str.iter().map(|c| if !alphabet.is_valid_base(*c) {
            panic!("character not in alphabet {}", c); }
        );
        SequenceAA{seq : str.to_vec()}
    } // end of new

    pub fn len(&self) -> usize {
        self.seq.len()
    }

    pub fn get_base(&self, pos : usize) -> u8 {
        if pos >= self.seq.len() {
            panic!("base position after end of sequence");
        }
        else {
            return self.seq[pos];
        }
    } // end of get_base

}  // end of SequenceAA


impl FromStr for SequenceAA {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //
        let sbytes = s.as_bytes();
        let alphabet = Alphabet::new();
        //
        sbytes.iter().map(|c| if !alphabet.is_valid_base(*c) {
                panic!("character not in alphabet {}", c);
            }
        );
        Ok(SequenceAA{seq:sbytes.to_vec()})
    }

}  // end of FromStr

//=========================================================================




pub struct KmerSeqIterator<'a, T> where T : CompressedKmerT {
    /// size of kmer
    nb_base: usize,
    /// an iterator for base calling
    sequence: &'a SequenceAA,
    /// last position of last kmer returned. At the beginning its None
    previous: Option<T>,
    ///
    range : Range<usize>,
    /// at present time, sequence for Amino Acid are not compressed, only Kmer so we do not need IterSequence as in mode base
    base_position : usize,
} // end of KmerSeqIterator


impl<'a, T> KmerSeqIterator<'a, T> where T : CompressedKmerT {

    pub fn new(kmer_size : usize, seq : &'a SequenceAA) -> Self {
        let range = std::ops::Range{start : 0, end : seq.len() -1};
        let base_position = 0;
        KmerSeqIterator{nb_base : kmer_size, sequence : seq, previous : None, range, base_position}
    }

    /// iterates...
    pub fn next(&mut self) -> Option<T> {
        // check for end of iterator
        if self.base_position >= self.sequence.len() {
            return None;
        }
        // now we know we are not at end of iterator
        // if we do not have a previous we have to contruct first kmer
        // we have to push a base.
        //
        if let Some(kmer) = self.previous {
            // in fact we have the base to push
            self.previous = Some(kmer.push(self.sequence.get_base(self.base_position)));
            self.base_position += 1;
            return self.previous;
        }
        else {
            // we are at beginning of kmer construction sequence, we must push kmer_size bases
            let kmer_size = self.nb_base as usize;
            let pos = 5*(kmer_size -1);
            let mut new_kmer = 0u128;

        }
        None
    } // end of next



    /// defines the range of kmer generation.  
    /// All bases in kmer generated must be between in first..last last excluded!
    fn set_range(&mut self, first: usize, last:usize) -> std::result::Result<(),()> { 
        if last <= first || last > self.sequence.len() {
            return Err(());
        }
        else {
            self.range = Range{start:first, end:last};
            self.base_position = first;
            return Ok(());
        }
    } // end of set_range

} // end of impl block for KmerSeqIterator

//============================================================================


pub trait KmerGenerationPattern<T:KmerT> {
    /// generate all kmers included in 0..
    fn generate_kmer_pattern(&self, seq : & SequenceAA) -> Vec<T>;
    /// generate all kmers inclused in begin..end with end excluded as in rust conventions.
    fn generate_kmer_pattern_in_range(&self, seq : & SequenceAA, begin:usize, end:usize) -> Vec<T>;   
    /// generate kmers with their multiplicities
    fn generate_kmer_distribution(&self, seq : & SequenceAA) -> Vec<(T,usize)>;
}



//==========================================================================================


use std::marker::PhantomData;

pub struct KmerGenerator<T:KmerT> {
    /// size of kmer we generate
    pub kmer_size : u8,
    t_marker: PhantomData<T>,
}


impl  <T:KmerT> KmerGenerator<T> {
    pub fn new(ksize:u8) -> Self {
        KmerGenerator{kmer_size: ksize, t_marker : PhantomData}
    }
    /// generic driver for kmer generation
    pub fn generate_kmer (&self, seq : &SequenceAA) -> Vec<T> where Self: KmerGenerationPattern<T> {
        self.generate_kmer_pattern(seq)
    }
    /// generic driver for kmer generation
    pub fn generate_kmer_in_range(&self, seq : & SequenceAA, begin:usize, end:usize) -> Vec<T>
    where Self: KmerGenerationPattern<T> {
        self.generate_kmer_pattern_in_range(seq, begin, end)
    }
    /// generic driver for kmer distribution pattern
    pub fn generate_weighted_kmer(&self, seq : &SequenceAA) -> Vec<(T,usize)>  where Self : KmerGenerationPattern<T> {
        self.generate_kmer_distribution(seq)
    }
    ///
    pub fn get_kmer_size(&self) -> usize { self.kmer_size as usize}
}  // end of impl KmerGenerator



/*
    Now we have the basics of Kmer Traits we implement KmerSeqIterator and KmerGenerationPattern
 */




/// implementation of kmer generation pattern for KmerAA<N>
impl KmerGenerationPattern<KmerAA> for KmerGenerator<KmerAA> {
    fn generate_kmer_pattern(&self, seq : &SequenceAA) -> Vec<KmerAA> {
        if self.kmer_size > 25 {
            panic!("KmerAA cannot have size greater than 25!!");   // cannot happen !
        }
        let kmer_size = self.kmer_size as usize; 
        // For a sequence of size the number of kmer is seq.size - kmer.size + 1  !!!
        // But it happens that "long reads" are really short 
        let nb_kmer = if seq.len() >= kmer_size { seq.len()-kmer_size+1} else {0};
        let mut kmer_vect = Vec::<KmerAA>::with_capacity(nb_kmer);
        let mut kmeriter  = KmerSeqIterator::new(kmer_size, seq);
        loop {
            match kmeriter.next() {
                Some(kmer) => kmer_vect.push(kmer),
                None => break,
            }
        }
        //
        return kmer_vect;
    }  // end of generate_kmer_pattern


    /// generate all kmers associated to their multiplicity
    /// This is useful in the context of Jaccard Probability Index estimated with ProbminHash 
    fn generate_kmer_distribution(&self, seq : &SequenceAA) -> Vec<(KmerAA,usize)> {
        if self.kmer_size as usize > 25 {
            panic!("KmerAA cannot be greater than 25!!");  // cannot happen
        }
        // For a sequence of size the number of kmer is seq.size - kmer.size + 1  !!!
        // But it happens that "long reads" are really short 
        let kmer_size = self.kmer_size as usize; 
        //
        let nb_kmer = if seq.len() >= kmer_size { seq.len()- kmer_size + 1} else {0};
        let mut kmer_distribution : FnvIndexMap::<KmerAA,usize> = FnvIndexMap::with_capacity_and_hasher(nb_kmer, FnvBuildHasher::default());
        let mut kmeriter = KmerSeqIterator::new(kmer_size, seq);
        loop {
            match kmeriter.next(){
                Some(kmer) => {
                    // do we store the kmer in the FnvIndexMap or a already hashed value aka nthash?
                    *kmer_distribution.entry(kmer).or_insert(0) += 1;
                },
                None => break,
            }
        }
        // convert to a Vec
        let mut hashed_kmers = kmer_distribution.keys();
        let mut weighted_kmer = Vec::<(KmerAA,usize)>::with_capacity(kmer_distribution.len());
        loop {
            match hashed_kmers.next() {
                Some(key) => {
                    if let Some(weight) = kmer_distribution.get(key) {
                        // get back to Kmer16b32bit from 
                        weighted_kmer.push((*key,*weight));
                    };
                },
                None => break,
            }
        }
        //
        return weighted_kmer;
    }  // end of generate_kmer_pattern



    fn generate_kmer_pattern_in_range(&self, seq : &SequenceAA, begin:usize, end:usize) -> Vec<KmerAA> {
        if self.kmer_size as usize > 25 {
            panic!("KmerAA cannot have size greater than 25");   // cannot happen
        }
        if begin >= end {
            panic!("KmerGenerationPattern<'a, KmerAA>  bad range for kmer iteration");
        }
        // For a sequence of size the number of kmer is seq.size - kmer.size + 1  !!!
        // But it happens that "long reads" are really short 
        let kmer_size = self.kmer_size as usize; 
        let nb_kmer = if seq.len() >= kmer_size { seq.len() - kmer_size + 1} else {0};
        let mut kmer_vect = Vec::<KmerAA>::with_capacity(nb_kmer);
        let mut kmeriter = KmerSeqIterator::new(kmer_size, seq);
        kmeriter.set_range(begin, end).unwrap();
        loop {
            match kmeriter.next() {
                Some(kmer) => kmer_vect.push(kmer),
                None => break,
            }
        }
        //
        return kmer_vect;
    }  // end of generate_kmer_pattern

}  // end of impl KmerGenerationPattern<'a, KmerAA<N>>




//===========================================================






#[cfg(test)]
mod tests {

// to run with  cargo test -- --nocapture kmeraa
    use super::*;

fn log_init_test() {
    let mut builder = env_logger::Builder::from_default_env();
    //    builder.filter_level(LevelFilter::Trace);
    let _ = builder.is_test(true).try_init();
}

    // test iterator
#[test]
    fn test_seqaa_iterator_range() {
        log_init_test();
        //
        log::info!("in test_seqaa_iterator_range");
        //
        let str = "MTEQIELIKLYSTRILALAAQMPHVGSLDNPDASAMKRSPLCGSKVTVDVIMQNGKITEF
        AQNVKACALGQAAASVAAQNIIGRTAEEVVRARDELAAMLKSGGPPPGPPFDGFEVLAPA
        SEYKNRHASILLSLDATAEACASIAAQNSA";

        let seqaa = SequenceAA::from_str(str).unwrap();
        // ask for Kmer of size 4
        let mut seq_iterator = KmerSeqIterator::<KmerAA>::new(4, &seqaa);
        // set a range 
        seq_iterator.set_range(3,8);
        // So we must havr from "QIEL" 
        let mut kmer_num = 0;
        let kmer_res = [ "QIEL" ,"IELI", "ELIK",  "LIKL"];
        while let Some(kmer) =  seq_iterator.next() {
            let k_uncompressed = kmer.get_uncompressed_kmer();
            let kmer_str=  std::str::from_utf8(&k_uncompressed).unwrap();
            log::info!(" kmer {} = {:?}", kmer_num, kmer_str);
            if kmer_str != kmer_res[kmer_num] {
                log::error!(" kmer {} = {:?}", kmer_num, kmer_str);
                panic!("error in kmeraa test::test_seq_aa_iterator at kmer num {}, got {:?}", kmer_num, kmer_res[kmer_num]);
            }
        }
        // check iterator sees the end
        match seq_iterator.next() {
            Some(kmer) => {
                panic!("iterator do not see end");
            },
            None => (),
        } // end match
    } // end of test_iterator_range

    // test we arrive at end correctly
#[test]
    fn test_seqaa_iterator_end() {

        let str = "MTEQIELIKLYSTRILALAAQMPHVGSLDNPDASAMKRSPLCGSKVTVDVIMQNGKITEF
        AQNVKACALGQAAASVAAQNIIGRTAEEVVRARDELAAMLKSGGPPPGPPFDGFEVLAPA
        SEYKNRHASILLSLDATAEACASIAAQNSA";

        let seqaa = SequenceAA::from_str(str).unwrap();
        // ask for Kmer of size 4
        let mut last_kmer : KmerAA;
        let mut seq_iterator = KmerSeqIterator::<KmerAA>::new(4, &seqaa);

    }

}  // end of mod tests