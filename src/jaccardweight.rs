#[allow(unused_imports)]
use log::*;

use std::fmt::{Debug};

#[allow(unused_imports)]
use std::hash::{BuildHasher, BuildHasherDefault, Hasher, Hash};
use indexmap::{IndexMap};
use fnv::{FnvBuildHasher};

use crate::nohasher::*;

use crate::kmergenerator::*;


type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

use probminhash::probminhash::*;



pub fn compute_probminhash3a_jaccard<D,H, Hidx>(idxa : &IndexMap<D, f64, Hidx>, idxb: &IndexMap<D, f64, Hidx>, 
                                                        sketch_size : usize, return_object: bool)  -> (f64, Option<Vec<D>>)
            where   D : Copy+Eq+Hash+Debug+Default,
                    H:  Hasher+Default ,
                    Hidx : std::hash::BuildHasher {
        //
        let mut pminhasha = ProbMinHash3a::<D,H>::new(sketch_size, D::default());
        pminhasha.hash_weigthed_idxmap(idxa);
        let mut pminhashb = ProbMinHash3a::<D,H>::new(sketch_size, D::default());
        pminhashb.hash_weigthed_idxmap(idxb);
        let siga = pminhasha.get_signature();
        let sigb = pminhashb.get_signature();
        let jac : f64;
        if !return_object {
            jac = compute_probminhash_jaccard(&siga, &sigb);
            return (jac,None);
        }
        else {
            return probminhash_get_jaccard_objects(&siga, &sigb);
        }
}  // end of compute_probminhash3a_jaccard



pub fn probminhash_get_jaccard_objects<D:Eq+Copy>(siga : &Vec<D>, sigb : &Vec<D>) -> (f64, Option<Vec<D>>) {
    let sig_size = siga.len();
    assert_eq!(sig_size, sigb.len());
    //
    let mut common_objects =  Vec::<D>::new();
    let mut inter = 0;
    for i in 0..siga.len() {
        if siga[i] == sigb[i] {
            inter += 1;
            common_objects.push(siga[i]);
        }
    }
    let jp = inter as f64/siga.len() as f64;
    //
    if jp > 0. {
        return (jp,Some(common_objects));
    }
    else {
        return (0., None);
    }
}  // end of compute_probminhash_jaccard





/// compute jaccard probability index between a sequence and a vecor of sequences sequences for Kmer16b32bit
/// and returns a vector of Jaccard probability index
pub fn sketch_seqrange_probminhash3a_kmer16b32bit<F>(seqa: &Sequence, vseqb : &Vec<&Sequence>, sketch_size: usize, fhash : F) -> Vec<f64> 
    where F : Fn(&Kmer16b32bit) -> u32 {
    //
    info!("seqsketcher : entering sketch_seqrange_probminhash3a_kmer16b32bit");
    // a vector to return results
    let mut jaccard_vec = Vec::<f64>::with_capacity(vseqb.len());
    // default is invertible hash and then superminhash without any hashing
    let mut pminhasha = ProbMinHash3a::<usize,NoHashHasher>::new(sketch_size, 0);
    let mut wa : FnvIndexMap::<usize,f64> = FnvIndexMap::with_capacity_and_hasher(seqa.size(), FnvBuildHasher::default());
    //
    // generate all kmers include in range arg. dependance upon kmer_size 
    // seqa
    let mut kmergen = KmerSeqIterator::<Kmer16b32bit>::new(16, &seqa);
    kmergen.set_range(0, seqa.size()).unwrap();
    loop {
        match kmergen.next() {
            Some(kmer) => {
                let hashval = fhash(&kmer);
                *wa.entry(hashval as usize).or_insert(0.) += 1.;
            },
            None => break,
        }
    }  // end loop
    pminhasha.hash_weigthed_idxmap(&wa);
    let siga = pminhasha.get_signature();
    // loop on vseqb to // with rayon
    for seqb in vseqb {
        let mut wb : FnvIndexMap::<usize,f64> = FnvIndexMap::with_capacity_and_hasher(seqb.size(), FnvBuildHasher::default());
        let mut kmergen = KmerSeqIterator::<Kmer16b32bit>::new(16, &seqb);
        kmergen.set_range(0, seqb.size()).unwrap();
        loop {
            match kmergen.next() {
                Some(kmer) => {
                    let hashval = fhash(&kmer);
                    *wb.entry(hashval as usize).or_insert(0.) += 1.;
                },
                None => break,
            }
        }  // end loop 
        let mut pminhashb = ProbMinHash3a::<usize,NoHashHasher>::new(sketch_size, 0);
        pminhashb.hash_weigthed_idxmap(&wb);
        let sigb = pminhashb.get_signature();
        let jac64 = compute_probminhash_jaccard(siga, sigb);
        jaccard_vec.push(jac64);
    }
    return jaccard_vec;
    //
} // end of sketch_seqrange_probminhash3a_kmer16b32bit




/// Compute jaccard probability index between a sequence and a vecor of sequences sequences for Kmer16b32bit
/// and returns a vector of Jaccard probability index
/// the fhash function is a hash function 
pub fn sketch_seqrange_probminhash3a_kmer32bit<F>(seqa: &Sequence, vseqb : &Vec<&Sequence>, sketch_size: usize, 
                    kmer_size : u8, fhash : F) -> Vec<f64> 
                    where F : Fn(&Kmer32bit) -> u32 {
    //
    info!("seqsketcher : entering sketch_seqrange_probminhash3a");
    // a vector to return results
    let mut jaccard_vec = Vec::<f64>::with_capacity(vseqb.len());
    // default is invertible hash and then superminhash without any hashing
    let mut pminhasha = ProbMinHash3a::<usize,NoHashHasher>::new(sketch_size, 0);
    let mut wa : FnvIndexMap::<usize,f64> = FnvIndexMap::with_capacity_and_hasher(seqa.size(), FnvBuildHasher::default());
    //
    // generate all kmers include in range arg. dependance upon kmer_size 
    // seqa
    let mut kmergen = KmerSeqIterator::<Kmer32bit>::new(kmer_size, &seqa);
    kmergen.set_range(0, seqa.size()).unwrap();
    loop {
        match kmergen.next() {
            Some(kmer) => {
                let hashval = fhash(&kmer);
                trace!(" kmer in seqa {:?}, hvalval  {:?} ", kmer.get_uncompressed_kmer(), hashval);
                *wa.entry(hashval as usize).or_insert(0.) += 1.;
            },
            None => break,
        }
    }  // end loop
    pminhasha.hash_weigthed_idxmap(&wa);
    let siga = pminhasha.get_signature();
    trace!("siga = {:?}", siga);
    // loop on vseqb to // with rayon
    for seqb in vseqb {
        let mut wb : FnvIndexMap::<usize,f64> = FnvIndexMap::with_capacity_and_hasher(seqb.size(), FnvBuildHasher::default());
        let mut kmergen = KmerSeqIterator::<Kmer32bit>::new(kmer_size, &seqb);
        kmergen.set_range(0, seqb.size()).unwrap();
        loop {
            match kmergen.next() {
                Some(kmer) => {
                    let hashval = fhash(&kmer);
                    trace!(" kmer in seqb {:?}  hashval {:?} ", kmer.get_uncompressed_kmer(), hashval);
                    *wb.entry(hashval as usize).or_insert(0.) += 1.;
                },
                None => break,
            }
        }  // end loop 
        let mut pminhashb = ProbMinHash3a::<usize,NoHashHasher>::new(sketch_size, 0);
        pminhashb.hash_weigthed_idxmap(&wb);
        let sigb = pminhashb.get_signature();
        trace!("sigb = {:?}", sigb);
        let jac64 = compute_probminhash_jaccard(siga, sigb);
        jaccard_vec.push(jac64);
    }
    return jaccard_vec;
    //
} // end of sketch_seqrange_probminhash3a_kmer32bit





//===========================================================================================================

#[cfg(test)]
mod tests {
    use super::*;


    #[allow(dead_code)]
    fn log_init_test() {
        let mut builder = env_logger::Builder::from_default_env();
    //    builder.filter_level(LevelFilter::Trace);
        let _ = builder.is_test(true).try_init();
    }

    #[test]
    // This function tests probability jaccard estimates on less than 16 bases 
    fn test_probminhash_kmer_smallb() {
        // initialize test logging
        log_init_test();
        let kmer_size = 5;
        let sketch_size = 50;
        // 80 bases
        let seqstr = String::from("TCAAAGGGAAACATTCAAAATCAGTATGCGCCCGTTCAGTTACGTATTGCTCTCGCTAATGAGATGGGCTGGGTACAGAG");
        let seqabytes = seqstr.as_bytes();
        let seqa = Sequence::new(seqstr.as_bytes(),2);
        //
        let mut vecseqb = Vec::<&Sequence>::new();
        // seqb1 has 40-kmer_size in common with seqstr. Jaccard index should be (40-kmer)/(80-kmer_size)
        let seqb1 = Sequence::new(&seqabytes[0..40],2);   // half the length of seqa
        vecseqb.push(&seqb1);
        //
        let seqarevcomp = seqa.get_reverse_complement();
        vecseqb.push(&seqarevcomp);
        let reverse_str = String::from_utf8(seqarevcomp.decompress()).unwrap();
        println!("\n reverse string : {}", reverse_str);
        // for this hash function seqarevcomp sjaccard signature  should be : [ (40-kmer_size)/(80-kmer_size) , 1.]
        let jac_theo_0 = (40-kmer_size) as f64 / (80-kmer_size) as f64;
        let kmer_revcomp_hash_fn = | kmer : &Kmer32bit | -> u32 {
            let canonical =  kmer.reverse_complement().min(*kmer);
            let hashval = probminhash::invhash::int32_hash(canonical.0);
            hashval
        };  
        // for this hash function (40-kmer)/(80-kmer_size) [ (40-kmer)/(80-kmer_size) , epsil]
        // epsil being the proba there is a common small kmer between seqstr and revers comp which can occur
        let kmer_identity = | kmer : &Kmer32bit | -> u32  {
            let hashval = kmer.0;
            hashval
        };
        let vecsig = sketch_seqrange_probminhash3a_kmer32bit(&seqa, &vecseqb, sketch_size, kmer_size, kmer_revcomp_hash_fn);
        debug!("vecsig with revcomp hash  {:?}", vecsig);
        assert!(vecsig[0] >= 0.75 * jac_theo_0);
        assert!(vecsig[1] >= 1.);
        // now we try with identity hash
        println!("calling with identity hash");
        let vecsig = sketch_seqrange_probminhash3a_kmer32bit(&seqa, &vecseqb, sketch_size, kmer_size, kmer_identity);
        debug!("vecsig with identity  {:?}", vecsig);
        assert!(vecsig[0] >= 0.75 * jac_theo_0);
        // get the kmer in intersection if any between seqa and its reverse complement.
        if vecsig[1] > 0. {
            // means we have a kmer in common in seqa and reverse complement of seqa. We check it
            println!("got intersection with reverse complement seq");
            let mut wa : FnvIndexMap::<usize,f64> = FnvIndexMap::with_capacity_and_hasher(seqa.size(), FnvBuildHasher::default());
            let mut pminhasha = ProbMinHash3a::<usize,NoHashHasher>::new(sketch_size, 0);
            // generate all kmers include in range arg. dependance upon kmer_size in seqa 
            let mut kmergen = KmerSeqIterator::<Kmer32bit>::new(kmer_size, &seqa);
            kmergen.set_range(0, seqa.size()).unwrap();
            loop {
                match kmergen.next() {
                    Some(kmer) => {
                        let hashval = kmer_identity(&kmer);
                        debug!(" kmer in seqa {:?}, hvalval  {:?} ", kmer.get_uncompressed_kmer(), hashval);
                        *wa.entry(hashval as usize).or_insert(0.) += 1.;
                    },
                    None => break,
                }
            }  // end loop
            pminhasha.hash_weigthed_idxmap(&wa);
            //
            let mut wb : FnvIndexMap::<usize,f64> = FnvIndexMap::with_capacity_and_hasher(seqarevcomp.size(), FnvBuildHasher::default());
            let mut pminhashb = ProbMinHash3a::<usize,NoHashHasher>::new(sketch_size, 0);
            // generate all kmers include in range arg. dependance upon kmer_size 
            let mut kmergen = KmerSeqIterator::<Kmer32bit>::new(kmer_size, &seqarevcomp);
            kmergen.set_range(0, seqarevcomp.size()).unwrap();
            loop {
                match kmergen.next() {
                    Some(kmer) => {
                        let hashval = kmer_identity(&kmer);
                        trace!(" kmer in seqrevcomp {:?}, hvalval  {:?} ", kmer.get_uncompressed_kmer(), hashval);
                        *wb.entry(hashval as usize).or_insert(0.) += 1.;
                    },
                    None => break,
                }
            }  // end loop    
            pminhashb.hash_weigthed_idxmap(&wb);
            let (jac, common) = probminhash_get_jaccard_objects(pminhasha.get_signature(), pminhashb.get_signature());
            debug!("jac for common objects = {}", jac);
            if jac > 0. {
                // with kmer size = 5 we have ACGTA and TACGT that are common!
                trace!("common kemrs {:?}", common.unwrap());
            }
        }  // end search of intersecting kmers
        //
        assert!(vecsig[1] <= 0.1);

    }  // end of test_probminhash_kmer_smallb


    #[test]
    fn test_probminhash_kmer_16b32bit() {
        // initialize test logging
        log_init_test();
        // 80 bases
        let kmer_size = 16;
        let seqstr = String::from("TCAAAGGGAAACATTCAAAATCAGTATGCGCCCGTTCAGTTACGTATTGCTCTCGCTAATGAGATGGGCTGGGTACAGAG");
        let seqabytes = seqstr.as_bytes();
        let seqa = Sequence::new(seqstr.as_bytes(),2);
        //
        let mut vecseqb = Vec::<&Sequence>::new();
        // seqb1 has 40-kmer_size in common with seqstr. Jaccard index should be (40-kmer)/(80-kmer_size)
        let seqb1 = Sequence::new(&seqabytes[0..40],2);   // half the length of seqa
        vecseqb.push(&seqb1);
        //
        let seqarevcomp = seqa.get_reverse_complement();
        vecseqb.push(&seqarevcomp);
        let reverse_str = String::from_utf8(seqarevcomp.decompress()).unwrap();
        println!("\n reverse string : {}", reverse_str);
        // for this hash function seqarevcomp sjaccard signature  should be : [ (40-kmer_size)/(80-kmer_size) , 1.]
        let jac_theo_0 = (40-kmer_size) as f64 / (80-kmer_size) as f64;
        let kmer_revcomp_hash_fn = | kmer : &Kmer16b32bit | -> u32 {
            let canonical =  kmer.reverse_complement().min(*kmer);
            let hashval = probminhash::invhash::int32_hash(canonical.0);
            hashval
        };  
        // for this hash function (40-kmer)/(80-kmer_size) [ (40-kmer)/(80-kmer_size) , epsil]
        // epsil being the proba there is a common small kmer between seqstr and revers comp which can occur
        let kmer_identity = | kmer : &Kmer16b32bit | -> u32  {
            let hashval = kmer.0;
            hashval
        };
        let vecsig = sketch_seqrange_probminhash3a_kmer16b32bit(&seqa, &vecseqb, 50, kmer_revcomp_hash_fn);
        debug!("vecsig with revcomp hash  {:?}", vecsig);
        assert!(vecsig[0] >= 0.75 * jac_theo_0);
        assert!(vecsig[1] >= 1.);
        //
        println!("calling with identity hash");
        let vecsig = sketch_seqrange_probminhash3a_kmer16b32bit(&seqa, &vecseqb, 50, kmer_identity);
        debug!("vecsig with identity  {:?}", vecsig);
        assert!(vecsig[0] >= 0.75 * jac_theo_0);
        assert!(vecsig[1] <= 0.1);
    }  // end of test_probminhash_kmer_16b32bit


}  // end of mod test

