//! provide minimal tool to sketch RNA sequences by probminhash3a


#![allow(unused)]

use std::io::{BufReader, BufWriter };


use std::fs::OpenOptions;
use std::fmt::{Debug};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{to_writer};

use indexmap::{IndexMap};
use fnv::{FnvBuildHasher};

use num;

use crate::nohasher::*;

use crate::base::{kmertraits::*};
use crate::rnautils::{kmeraa::*};

use rayon::prelude::*;

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

use probminhash::probminhasher::*;



// TODO this should be factorized with DNA case.

#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct SeqSketcher {
    kmer_size : usize,
    sketch_size : usize
}  // end of SeqSketcher


impl SeqSketcher {
    /// 
    pub fn new(kmer_size: usize, sketch_size : usize) -> Self {
        SeqSketcher{kmer_size, sketch_size}
    }

    /// returns kmer size
    pub fn get_kmer_size(&self) -> usize {
        self.kmer_size
    }

    /// return sketch size
    pub fn get_sketch_size(&self) -> usize {
        self.sketch_size
    }  
    
    /// serialized dump
    pub fn dump_json(&self, filename : &String) -> Result<(), String> {
        //
        let filepath = PathBuf::from(filename.clone());
        //
        log::info!("dumping sketching parameters in json file : {}", filename);
        //
        let fileres = OpenOptions::new().write(true).create(true).truncate(true).open(&filepath);
        if fileres.is_err() {
            log::error!("SeqSketcher dump : dump could not open file {:?}", filepath.as_os_str());
            println!("SeqSketcher dump: could not open file {:?}", filepath.as_os_str());
            return Err("SeqSketcher dump failed".to_string());
        }
        // 
        let mut writer = BufWriter::new(fileres.unwrap());
        let _ = to_writer(&mut writer, &self).unwrap();
        //
        Ok(())
    } // end of dump


    /// reload from a json dump
    pub fn reload_json(dirpath : &Path) -> Result<SeqSketcher, String> {
        log::info!("in reload_json");
        //
        let filepath = dirpath.join("sketchparams_dump.json");
        let fileres = OpenOptions::new().read(true).open(&filepath);
        if fileres.is_err() {
            log::error!("Sketcher reload_json : reload could not open file {:?}", filepath.as_os_str());
            println!("Sketcher reload_json: could not open file {:?}", filepath.as_os_str());
            return Err("Sketcher reload_json could not open file".to_string());            
        }
        //
        let loadfile = fileres.unwrap();
        let reader = BufReader::new(loadfile);
        let sketch_params:SeqSketcher = serde_json::from_reader(reader).unwrap();
        //
        log::info!("SeqSketcher reload, kmer_size : {}, sketch_size : {}", 
            sketch_params.get_kmer_size(), sketch_params.get_sketch_size());     
        //
        Ok(sketch_params)
    } // end of reload_json




    /// a generic implementation of probminhash3a  against our stndard compressed Kmer types
    /// Kmer::Val is the base type u128 on which compressed kmer representations relies. 
    /// We could lso implement a version on 64bit , we have KmerAA64bit.
    pub fn sketch_probminhash3a_compressed_kmeraa128bit<'b, KmerAA128bit : CompressedKmerT, F>(&self, vseq : &'b Vec<&SequenceAA>, fhash : F) -> Vec<Vec<KmerAA128bit::Val> >
        where F : Fn(&KmerAA128bit) -> KmerAA128bit::Val + Send + Sync,
              KmerAA128bit::Val : num::PrimInt + Send + Sync + Debug,
              KmerGenerator<KmerAA128bit> :  KmerGenerationPattern<KmerAA128bit> {
        //
        let comput_closure = | seqb : &'b SequenceAA, i:usize | -> (usize,Vec<KmerAA128bit::Val>) {
            let kmers : Vec<(KmerAA128bit, usize)>= KmerGenerator::new(self.kmer_size as u8).generate_kmer_distribution(&seqb);
            // now we have weights but in usize and we want them in float!! and in another FnvIndexMap, ....
            // TODO we pass twice in a FnvIndexMap! generate_kmer_distribution should return a FnvIndexMap
            let mut wb : FnvIndexMap::<KmerAA128bit::Val,f64> = FnvIndexMap::with_capacity_and_hasher(seqb.size(), FnvBuildHasher::default());
            for kmer in kmers {
                let hashval = fhash(&kmer.0);
                let res = wb.insert(hashval, kmer.1 as f64);
                match res {
                    Some(_) => {
                        panic!("key already existed");
                    }
                    _ => { }
                }
            }
            // We cannot use NohashHasher beccause Hash::finish is declared to return a f64 in trait std::hash::Hasher
            let mut pminhashb = ProbMinHash3a::<KmerAA128bit::Val,fnv::FnvHasher>::new(self.sketch_size, num::zero::<KmerAA128bit::Val>());
            pminhashb.hash_weigthed_idxmap(&wb);
            let sigb = pminhashb.get_signature();
            // get back from usize to Kmer32bit ?. If fhash is inversible possible, else NO.
            return (i,sigb.clone());
        };
        //
        let sig_with_rank : Vec::<(usize,Vec<KmerAA128bit::Val>)> = (0..vseq.len()).into_par_iter().map(|i| comput_closure(vseq[i],i)).collect();
        // re-order from jac_with_rank to jaccard_vec as the order of return can be random!!
        let mut jaccard_vec = Vec::<Vec<KmerAA128bit::Val>>::with_capacity(vseq.len());
        for _ in 0..vseq.len() {
            jaccard_vec.push(Vec::new());
        }
        // CAVEAT , boxing would avoid the clone?
        for i in 0..sig_with_rank.len() {
            let slot = sig_with_rank[i].0;
            jaccard_vec[slot] = sig_with_rank[i].1.clone();
        }
        jaccard_vec
    }  // end of sketch_probminhash3a_compressedkmer

} // end of SeqSketcher (RNA case)



//=========================================================


#[cfg(test)]
mod tests {

use super::*;
use std::str::FromStr;

    fn log_init_test() {
        let mut builder = env_logger::Builder::from_default_env();
        //    builder.filter_level(LevelFilter::Trace);
        let _ = builder.is_test(true).try_init();
    }

    #[test]
    fn test_seqaa_probminhash_128bit() {
        log_init_test();
        //
        log::debug!("test_seqaa_probminhash");
        //
        let str1 = "MTEQIELIKLYSTRILALAAQMPHVGSLDNPDASAMKRSPLCGSKVTVDVIMQNGKITFDGFEVLAPASEYKNRHASILLSLDATAEACASIAAQNSA";
        // The second string is the first half of the first repeated
        let str2 = "MTEQIELIKLYSTRILALAAQMPHVGSLDNPDASAMKRSPLCGSKVMTEQIELIKLYSTRILALAAQMPHVGSLDNPDASAMKRSPLCGSKV";

        let seq1 = SequenceAA::from_str(str1).unwrap();
        let seq2 = SequenceAA::from_str(str2).unwrap();
        let vseq = vec![&seq1, &seq2];
        let kmer_size = 5;
        let sketch_size = 100;
        let sketcher = SeqSketcher::new(kmer_size, sketch_size);
        let nb_alphabet_bits = Alphabet::new().get_nb_bits();
        // we need a hash function from u128 to f64
        let kmer_hash_fn = | kmer : &KmerAA128bit | -> <KmerAA128bit as CompressedKmerT>::Val {
            let mask : <KmerAA128bit as CompressedKmerT>::Val = num::NumCast::from::<u128>((0b1 << nb_alphabet_bits*kmer.get_nb_base()) - 1).unwrap();
            let hashval = kmer.get_compressed_value() & mask;
            hashval
        };
        let mask : u128 = num::NumCast::from::<u128>((0b1 << nb_alphabet_bits*kmer_size as u8) - 1).unwrap();
        log::info!("mask = {:b}", mask);
        //
        log::info!("calling sketch_probminhash3a_compressedKmerAA128bit");
        let signatures = sketcher.sketch_probminhash3a_compressed_kmeraa128bit(&vseq, kmer_hash_fn); 
        // get distance between the 2 strings  
        // compute Jp as in 
        let mut jp = 0.;
        let sig1 = &signatures[0];
        let sig2 = &signatures[1];
        //
        let inter : u128 = sig1.iter().zip(sig2.iter()).map(|(a,b)| if a==b {1} else {0}).sum();
        let dist = inter as f64/sig1.len() as f64;
        log::info!("inter : {:?} length {:?} jaccard distance {:?}", inter, sig1.len(), dist );
        assert!( (dist-0.5).abs() < 1./10.);
    } // end of test_seqaa_probminhash_128bit


}  // end of mod tests in rnautils::seqsketchjaccard