//! FPGA Toolkit — TLMM ternary encoding, COE memory files, Hilbert curve tile mapping
//! Bridges frozen-intelligence weights to FPGA BRAM initialization.

use std::collections::HashMap;

/// Ternary Level Matrix Module (TLMM) encoding
/// Converts INT8 weights to ternary {-1, 0, +1} for lookup-table-based inference.

#[derive(Debug, Clone, PartialEq)]
pub enum Ternary {
    NegOne,   // -1
    Zero,     // 0
    One,      // +1
}

impl Ternary {
    pub fn from_i8(w: i8) -> Self {
        if w > 0 { Ternary::One }
        else if w < 0 { Ternary::NegOne }
        else { Ternary::Zero }
    }
    
    pub fn to_i8(&self) -> i8 {
        match self {
            Ternary::NegOne => -1,
            Ternary::Zero => 0,
            Ternary::One => 1,
        }
    }
    
    pub fn to_bits(&self) -> u8 {
        match self {
            Ternary::Zero => 0b00,
            Ternary::One => 0b01,
            Ternary::NegOne => 0b10,
        }
    }
    
    /// Pack 4 ternary values into 1 byte (2 bits each)
    pub fn pack4(values: &[Ternary]) -> u8 {
        assert!(values.len() >= 4, "Need 4 ternary values");
        values[0].to_bits() | (values[1].to_bits() << 2) 
        | (values[2].to_bits() << 4) | (values[3].to_bits() << 6)
    }
    
    /// Unpack 1 byte into 4 ternary values
    pub fn unpack4(byte: u8) -> [Ternary; 4] {
        [
            Self::from_bits(byte & 0x03),
            Self::from_bits((byte >> 2) & 0x03),
            Self::from_bits((byte >> 4) & 0x03),
            Self::from_bits((byte >> 6) & 0x03),
        ]
    }
    
    fn from_bits(bits: u8) -> Self {
        match bits {
            0b01 => Ternary::One,
            0b10 => Ternary::NegOne,
            _ => Ternary::Zero,
        }
    }
}

/// Quantization statistics for a weight matrix
#[derive(Debug, Clone)]
pub struct QuantStats {
    pub total_weights: usize,
    pub ternary_ones: usize,
    pub ternary_negones: usize,
    pub ternary_zeros: usize,
    pub compression_ratio: f64,
    pub original_bits: usize,
    pub ternary_bits: usize,
}

impl QuantStats {
    pub fn sparsity(&self) -> f64 {
        if self.total_weights == 0 { return 0.0; }
        self.ternary_zeros as f64 / self.total_weights as f64
    }
}

/// TLMM encoder — converts weight matrices to ternary
pub struct TlmmEncoder {
    threshold: i8,
}

impl TlmmEncoder {
    pub fn new() -> Self { Self { threshold: 0 } }
    
    pub fn with_threshold(mut self, t: i8) -> Self { self.threshold = t; self }
    
    /// Encode a flat INT8 weight buffer to ternary
    pub fn encode(&self, weights: &[i8]) -> (Vec<Ternary>, QuantStats) {
        let mut ternary = Vec::with_capacity(weights.len());
        let mut ones = 0usize;
        let mut negs = 0usize;
        let mut zeros = 0usize;
        
        for &w in weights {
            let t = if w > self.threshold { Ternary::One } 
                    else if w < -self.threshold { Ternary::NegOne }
                    else { Ternary::Zero };
            match t {
                Ternary::One => ones += 1,
                Ternary::NegOne => negs += 1,
                Ternary::Zero => zeros += 1,
            }
            ternary.push(t);
        }
        
        let total = weights.len();
        let stats = QuantStats {
            total_weights: total,
            ternary_ones: ones,
            ternary_negones: negs,
            ternary_zeros: zeros,
            compression_ratio: (total * 8) as f64 / ((total + 3) / 4 * 8) as f64,
            original_bits: total * 8,
            ternary_bits: (total + 3) / 4 * 8,
        };
        (ternary, stats)
    }
    
    /// Pack ternary values into dense byte buffer
    pub fn pack(&self, ternary: &[Ternary]) -> Vec<u8> {
        let mut packed = vec![];
        for chunk in ternary.chunks(4) {
            let mut vals = [Ternary::Zero; 4];
            for (i, &t) in chunk.iter().enumerate() { vals[i] = t; }
            packed.push(Ternary::pack4(&vals));
        }
        packed
    }
    
    /// Generate TLMM lookup table (3 outputs x 3 inputs = 9 entries)
    pub fn lookup_table() -> [[i8; 3]; 3] {
        let neg = Ternary::NegOne.to_i8();
        let zero = Ternary::Zero.to_i8();
        let one = Ternary::One.to_i8();
        [
            [neg * neg, neg * zero, neg * one],   // -1 * {-1, 0, 1}
            [zero * neg, zero * zero, zero * one], // 0 * {-1, 0, 1}
            [one * neg, one * zero, one * one],    // 1 * {-1, 0, 1}
        ]
    }
}

/// COE file generator — creates Xilinx .coe memory initialization files
pub struct CoeGenerator {
    pub radix: u32,
    pub data_width: u32,
}

impl CoeGenerator {
    pub fn new(data_width: u32) -> Self { Self { radix: 16, data_width } }
    
    /// Generate .coe file content from packed bytes
    pub fn generate(&self, packed: &[u8], comment: &str) -> String {
        let mut lines = vec![];
        lines.push(format!("; {}", comment));
        lines.push(format!("memory_initialization_radix={};", self.radix));
        lines.push(format!("memory_initialization_vector=",));
        
        for (i, chunk) in packed.chunks(4).enumerate() {
            let word = if chunk.len() == 4 {
                u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            } else {
                let mut buf = [0u8; 4];
                for (j, &b) in chunk.iter().enumerate() { buf[j] = b; }
                u32::from_le_bytes(buf)
            };
            lines.push(format!("{:08X},", word));
        }
        
        lines.join("\n")
    }
    
    /// Generate .mif file (Intel/Altera format)
    pub fn generate_mif(&self, packed: &[u8], depth: usize, width: usize) -> String {
        let mut lines = vec![];
        lines.push("-- Generated by cuda-fpga-toolkit".to_string());
        lines.push(format!("WIDTH={};", width));
        lines.push(format!("DEPTH={};", depth));
        lines.push("".to_string());
        lines.push("ADDRESS_RADIX=UNS;".to_string());
        lines.push("DATA_RADIX=HEX;".to_string());
        lines.push("CONTENT BEGIN".to_string());
        
        for (i, chunk) in packed.chunks(4).enumerate() {
            let word = if chunk.len() == 4 {
                u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
            } else {
                let mut buf = [0u8; 4];
                for (j, &b) in chunk.iter().enumerate() { buf[j] = b; }
                u32::from_le_bytes(buf)
            };
            lines.push(format!("    {:04X} : {:08X};", i, word));
        }
        
        lines.push("END;".to_string());
        lines.join("\n")
    }
}

/// Hilbert curve tile mapper — maps 2D tile coordinates to 1D addresses
/// for cache-friendly weight access on FPGA.

#[derive(Debug, Clone)]
pub struct HilbertMapper {
    pub order: u32, // Hilbert curve order (2^order x 2^order grid)
    pub grid_size: u32,
}

impl HilbertMapper {
    pub fn new(order: u32) -> Self {
        let grid_size = 1u32 << order;
        Self { order, grid_size }
    }
    
    /// Convert 2D tile coordinate to 1D Hilbert address
    pub fn encode(&self, x: u32, y: u32) -> u32 {
        let mut d = 0u32;
        for s in (0..self.order).rev() {
            let rx = ((x >> s) & 1) != 0;
            let ry = ((y >> s) & 1) != 0;
            d = self.hilbert_rotate(d, s, rx, ry);
            d += self.xy_to_d(rx, ry, s);
        }
        d
    }
    
    /// Convert 1D Hilbert address to 2D tile coordinate
    pub fn decode(&self, d: u32) -> (u32, u32) {
        let mut x = 0u32;
        let mut y = 0u32;
        let mut dd = d;
        for s in (0..self.order).rev() {
            let rx = self.bit(dd, 2 * s + 1);
            let ry = self.bit(dd, 2 * s);
            let rot = self.hilbert_rot(s, rx, ry);
            x |= self.d_to_xy(rot.0, rx, ry, s);
            y |= self.d_to_xy(rot.1, rx, ry, s);
        }
        (x, y)
    }
    
    /// Generate access order for all tiles
    pub fn access_order(&self) -> Vec<(u32, u32)> {
        let total = self.grid_size * self.grid_size;
        (0..total).map(|d| self.decode(d)).collect()
    }
    
    fn bit(&self, val: u32, pos: u32) -> u32 { (val >> pos) & 1 }
    
    fn xy_to_d(&self, rx: bool, ry: bool, s: u32) -> u32 {
        let rx = rx as u32;
        let ry = ry as u32;
        let ss = 1u32 << (2 * s);
        rx * ss + ry * (ss / 2)
    }
    
    fn d_to_xy(&self, rot: u32, rx: bool, ry: bool, s: u32) -> u32 {
        let ss = 1u32 << s;
        let rx = rx as u32;
        let ry = ry as u32;
        let t = (rx ^ ry) << s;
        (rot * ss) ^ (ry * ss) ^ t
    }
    
    fn hilbert_rotate(&self, mut n: u32, s: u32, rx: bool, ry: bool) -> u32 {
        if !ry {
            if rx {
                n = ((1u32 << (2 * (s + 1))) - 1) ^ n;
            }
            // Swap x and y
            let msb = self.bit(n, 2 * s);
            let lsb = self.bit(n, 2 * s + 1);
            n = (n & !(3u32 << (2 * s))) | (msb << (2 * s + 1)) | (lsb << (2 * s));
        }
        n
    }
    
    fn hilbert_rot(&self, s: u32, rx: bool, ry: bool) -> (u32, u32) {
        if ry { (rx as u32, (1 ^ rx) as u32) }
        else { ((1 ^ rx) as u32, rx as u32) }
    }
}

/// FPGA resource estimator for TLMM-based inference
#[derive(Debug, Clone)]
pub struct FpgaResourceEstimate {
    pub lut_count: u64,
    pub bram_count: u64,
    pub dsp_count: u64,
    pub flip_flops: u64,
    pub est_freq_mhz: f64,
    pub weights_supported: usize,
}

impl FpgaResourceEstimate {
    /// Estimate resources for a TLMM layer
    pub fn for_layer(weights: &[i8], neurons: usize, bits_per_weight: u32) -> Self {
        let encoder = TlmmEncoder::new();
        let (_, stats) = encoder.encode(weights);
        
        let packed_size = (weights.len() + 3) / 4;
        let bram_depth = 1024; // 36Kb BRAM = 1024 x 36 bits
        let bram_count = (packed_size + bram_depth - 1) / bram_depth;
        
        // LUTs: 3 LUTs per ternary MAC (lookup table), neurons per cycle
        let lut_per_mac = 9; // 3x3 lookup table
        let luts = neurons * lut_per_mac;
        
        Self {
            lut_count: luts as u64,
            bram_count: bram_count as u64,
            dsp_count: 0, // TLMM uses LUTs, not DSPs
            flip_flops: luts as u64 * 2,
            est_freq_mhz: 250.0, // LUT-based inference is fast
            weights_supported: weights.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_encoding() {
        assert_eq!(Ternary::from_i8(5), Ternary::One);
        assert_eq!(Ternary::from_i8(-3), Ternary::NegOne);
        assert_eq!(Ternary::from_i8(0), Ternary::Zero);
    }

    #[test]
    fn test_ternary_pack_unpack() {
        let vals = [Ternary::One, Ternary::NegOne, Ternary::Zero, Ternary::One];
        let packed = Ternary::pack4(&vals);
        let unpacked = Ternary::unpack4(packed);
        assert_eq!(vals, unpacked);
    }

    #[test]
    fn test_tlmm_encode() {
        let encoder = TlmmEncoder::new();
        let weights: Vec<i8> = vec![10, -5, 0, 3, -2, 0, 7, -8];
        let (ternary, stats) = encoder.encode(&weights);
        assert_eq!(ternary.len(), 8);
        assert_eq!(stats.total_weights, 8);
        assert_eq!(stats.ternary_ones, 3); // 10, 3, 7
        assert_eq!(stats.ternary_negones, 3); // -5, -2, -8
        assert_eq!(stats.ternary_zeros, 2); // 0, 0
    }

    #[test]
    fn test_tlmm_pack() {
        let encoder = TlmmEncoder::new();
        let weights = vec![1i8, -1, 0, 1, -1, 0, 1, 1];
        let (ternary, _) = encoder.encode(&weights);
        let packed = encoder.pack(&ternary);
        assert_eq!(packed.len(), 2); // 8 ternary / 4 per byte
    }

    #[test]
    fn test_lookup_table() {
        let lut = TlmmEncoder::lookup_table();
        assert_eq!(lut[0][0], 1);   // -1 * -1 = 1
        assert_eq!(lut[0][2], -1);  // -1 * 1 = -1
        assert_eq!(lut[2][2], 1);   // 1 * 1 = 1
        assert_eq!(lut[1][0], 0);   // 0 * -1 = 0
    }

    #[test]
    fn test_coe_generation() {
        let gen = CoeGenerator::new(32);
        let packed = vec![0x01, 0x02, 0x03, 0x04, 0xFF, 0x00, 0x00, 0x00];
        let coe = gen.generate(&packed, "Test weights");
        assert!(coe.contains("memory_initialization_radix="));
        assert!(coe.contains("04030201")); // Little-endian first word
    }

    #[test]
    fn test_mif_generation() {
        let gen = CoeGenerator::new(32);
        let packed = vec![0x01, 0x02, 0x03, 0x04];
        let mif = gen.generate_mif(&packed, 1024, 32);
        assert!(mif.contains("WIDTH=32"));
        assert!(mif.contains("DEPTH=1024"));
        assert!(mif.contains("CONTENT BEGIN"));
    }

    #[test]
    fn test_hilbert_encode_decode() {
        let mapper = HilbertMapper::new(2); // 4x4 grid
        for x in 0..4 {
            for y in 0..4 {
                let d = mapper.encode(x, y);
                let (dx, dy) = mapper.decode(d);
                assert_eq!((dx, dy), (x, y), "Failed at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_hilbert_access_order() {
        let mapper = HilbertMapper::new(1); // 2x2
        let order = mapper.access_order();
        assert_eq!(order.len(), 4);
        // Hilbert order for 2x2: (0,0), (0,1), (1,1), (1,0)
        assert_eq!(order[0], (0, 0));
        assert_eq!(order[1], (0, 1));
    }

    #[test]
    fn test_fpga_resource_estimate() {
        let weights = vec![1i8; 1024];
        let est = FpgaResourceEstimate::for_layer(&weights, 64, 2);
        assert!(est.lut_count > 0);
        assert!(est.bram_count > 0);
        assert_eq!(est.dsp_count, 0); // TLMM uses LUTs
        assert!(est.est_freq_mhz > 0.0);
    }
}
