pub trait Digest {
    fn update(&mut self, data: &[u8]);
    fn finalize_hex(self: Box<Self>) -> String;
}

#[derive(Clone)]
pub struct Md5 {
    state: [u32; 4],
    count: u64,
    buffer: [u8; 64],
}

impl Default for Md5 {
    fn default() -> Self {
        Self::new()
    }
}

impl Md5 {
    pub fn new() -> Self {
        Self {
            state: [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476],
            count: 0,
            buffer: [0u8; 64],
        }
    }

    pub fn update(&mut self, input: &[u8]) {
        let idx = (self.count & 0x3f) as usize;
        self.count = self.count.wrapping_add(input.len() as u64);
        let mut input_idx = 0;

        if idx > 0 {
            let space = 64 - idx;
            if input.len() >= space {
                self.buffer[idx..64].copy_from_slice(&input[..space]);
                Self::process_block(&mut self.state, &self.buffer);
                input_idx = space;
            } else {
                self.buffer[idx..idx + input.len()].copy_from_slice(input);
                return;
            }
        }

        while input_idx + 64 <= input.len() {
            let block: &[u8; 64] = input[input_idx..input_idx + 64].try_into().unwrap();
            Self::process_block(&mut self.state, block);
            input_idx += 64;
        }

        if input_idx < input.len() {
            let rem = &input[input_idx..];
            self.buffer[..rem.len()].copy_from_slice(rem);
        }
    }

    pub fn finalize(mut self) -> [u8; 16] {
        let bit_len = self.count.wrapping_mul(8);
        let idx = (self.count & 0x3f) as usize;

        let pad_len = if idx < 56 { 56 - idx } else { 120 - idx };
        let mut pad = [0u8; 128];
        pad[0] = 0x80;
        self.update(&pad[..pad_len]);

        let len_bytes = bit_len.to_le_bytes();
        self.update(&len_bytes);

        let mut out = [0u8; 16];
        for (i, val) in self.state.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_le_bytes());
        }
        out
    }

    pub fn finalize_hex(&self) -> String {
        hex_encode(&self.clone().finalize())
    }

    fn process_block(state: &mut [u32; 4], block: &[u8; 64]) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes(block[i * 4..(i + 1) * 4].try_into().unwrap());
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];

        #[inline(always)]
        fn f(x: u32, y: u32, z: u32) -> u32 {
            (x & y) | (!x & z)
        }
        #[inline(always)]
        fn g(x: u32, y: u32, z: u32) -> u32 {
            (x & z) | (y & !z)
        }
        #[inline(always)]
        fn h(x: u32, y: u32, z: u32) -> u32 {
            x ^ y ^ z
        }
        #[inline(always)]
        fn i(x: u32, y: u32, z: u32) -> u32 {
            y ^ (x | !z)
        }

        macro_rules! step {
            ($func:ident, $a:expr, $b:expr, $c:expr, $d:expr, $k:expr, $s:expr, $t:expr) => {
                $a = $b.wrapping_add(
                    $a.wrapping_add($func($b, $c, $d))
                        .wrapping_add($k)
                        .wrapping_add($t)
                        .rotate_left($s),
                );
            };
        }

        step!(f, a, b, c, d, m[0], 7, 0xd76aa478);
        step!(f, d, a, b, c, m[1], 12, 0xe8c7b756);
        step!(f, c, d, a, b, m[2], 17, 0x242070db);
        step!(f, b, c, d, a, m[3], 22, 0xc1bdceee);
        step!(f, a, b, c, d, m[4], 7, 0xf57c0faf);
        step!(f, d, a, b, c, m[5], 12, 0x4787c62a);
        step!(f, c, d, a, b, m[6], 17, 0xa8304613);
        step!(f, b, c, d, a, m[7], 22, 0xfd469501);
        step!(f, a, b, c, d, m[8], 7, 0x698098d8);
        step!(f, d, a, b, c, m[9], 12, 0x8b44f7af);
        step!(f, c, d, a, b, m[10], 17, 0xffff5bb1);
        step!(f, b, c, d, a, m[11], 22, 0x895cd7be);
        step!(f, a, b, c, d, m[12], 7, 0x6b901122);
        step!(f, d, a, b, c, m[13], 12, 0xfd987193);
        step!(f, c, d, a, b, m[14], 17, 0xa679438e);
        step!(f, b, c, d, a, m[15], 22, 0x49b40821);

        step!(g, a, b, c, d, m[1], 5, 0xf61e2562);
        step!(g, d, a, b, c, m[6], 9, 0xc040b340);
        step!(g, c, d, a, b, m[11], 14, 0x265e5a51);
        step!(g, b, c, d, a, m[0], 20, 0xe9b6c7aa);
        step!(g, a, b, c, d, m[5], 5, 0xd62f105d);
        step!(g, d, a, b, c, m[10], 9, 0x02441453);
        step!(g, c, d, a, b, m[15], 14, 0xd8a1e681);
        step!(g, b, c, d, a, m[4], 20, 0xe7d3fbc8);
        step!(g, a, b, c, d, m[9], 5, 0x21e1cde6);
        step!(g, d, a, b, c, m[14], 9, 0xc33707d6);
        step!(g, c, d, a, b, m[3], 14, 0xf4d50d87);
        step!(g, b, c, d, a, m[8], 20, 0x455a14ed);
        step!(g, a, b, c, d, m[13], 5, 0xa9e3e905);
        step!(g, d, a, b, c, m[2], 9, 0xfcefa3f8);
        step!(g, c, d, a, b, m[7], 14, 0x676f02d9);
        step!(g, b, c, d, a, m[12], 20, 0x8d2a4c8a);

        step!(h, a, b, c, d, m[5], 4, 0xfffa3942);
        step!(h, d, a, b, c, m[8], 11, 0x8771f681);
        step!(h, c, d, a, b, m[11], 16, 0x6d9d6122);
        step!(h, b, c, d, a, m[14], 23, 0xfde5380c);
        step!(h, a, b, c, d, m[1], 4, 0xa4beea44);
        step!(h, d, a, b, c, m[4], 11, 0x4bdecfa9);
        step!(h, c, d, a, b, m[7], 16, 0xf6bb4b60);
        step!(h, b, c, d, a, m[10], 23, 0xbebfbc70);
        step!(h, a, b, c, d, m[13], 4, 0x289b7ec6);
        step!(h, d, a, b, c, m[0], 11, 0xeaa127fa);
        step!(h, c, d, a, b, m[3], 16, 0xd4ef3085);
        step!(h, b, c, d, a, m[6], 23, 0x04881d05);
        step!(h, a, b, c, d, m[9], 4, 0xd9d4d039);
        step!(h, d, a, b, c, m[12], 11, 0xe6db99e5);
        step!(h, c, d, a, b, m[15], 16, 0x1fa27cf8);
        step!(h, b, c, d, a, m[2], 23, 0xc4ac5665);

        step!(i, a, b, c, d, m[0], 6, 0xf4292244);
        step!(i, d, a, b, c, m[7], 10, 0x432aff97);
        step!(i, c, d, a, b, m[14], 15, 0xab9423a7);
        step!(i, b, c, d, a, m[5], 21, 0xfc93a039);
        step!(i, a, b, c, d, m[12], 6, 0x655b59c3);
        step!(i, d, a, b, c, m[3], 10, 0x8f0ccc92);
        step!(i, c, d, a, b, m[10], 15, 0xffeff47d);
        step!(i, b, c, d, a, m[1], 21, 0x85845dd1);
        step!(i, a, b, c, d, m[8], 6, 0x6fa87e4f);
        step!(i, d, a, b, c, m[15], 10, 0xfe2ce6e0);
        step!(i, c, d, a, b, m[6], 15, 0xa3014314);
        step!(i, b, c, d, a, m[13], 21, 0x4e0811a1);
        step!(i, a, b, c, d, m[4], 6, 0xf7537e82);
        step!(i, d, a, b, c, m[11], 10, 0xbd3af235);
        step!(i, c, d, a, b, m[2], 15, 0x2ad7d2bb);
        step!(i, b, c, d, a, m[9], 21, 0xeb86d391);

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }
}

impl Digest for Md5 {
    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize_hex(self: Box<Self>) -> String {
        hex_encode(&self.finalize())
    }
}

#[derive(Clone)]
pub struct Sha1 {
    state: [u32; 5],
    count: u64,
    buffer: [u8; 64],
}

impl Default for Sha1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha1 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6745_2301,
                0xefcd_ab89,
                0x98ba_dcfe,
                0x1032_5476,
                0xc3d2_e1f0,
            ],
            count: 0,
            buffer: [0u8; 64],
        }
    }

    pub fn update(&mut self, input: &[u8]) {
        let idx = (self.count & 0x3f) as usize;
        self.count = self.count.wrapping_add(input.len() as u64);
        let mut input_idx = 0;

        if idx > 0 {
            let space = 64 - idx;
            if input.len() >= space {
                self.buffer[idx..64].copy_from_slice(&input[..space]);
                Self::process_block(&mut self.state, &self.buffer);
                input_idx = space;
            } else {
                self.buffer[idx..idx + input.len()].copy_from_slice(input);
                return;
            }
        }

        while input_idx + 64 <= input.len() {
            let block: &[u8; 64] = input[input_idx..input_idx + 64].try_into().unwrap();
            Self::process_block(&mut self.state, block);
            input_idx += 64;
        }

        if input_idx < input.len() {
            let rem = &input[input_idx..];
            self.buffer[..rem.len()].copy_from_slice(rem);
        }
    }

    pub fn finalize(mut self) -> [u8; 20] {
        let bit_len = self.count.wrapping_mul(8);
        let idx = (self.count & 0x3f) as usize;

        let pad_len = if idx < 56 { 56 - idx } else { 120 - idx };
        let mut pad = [0u8; 128];
        pad[0] = 0x80;
        self.update(&pad[..pad_len]);

        let len_bytes = bit_len.to_be_bytes();
        self.update(&len_bytes);

        let mut out = [0u8; 20];
        for (i, val) in self.state.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
        }
        out
    }

    pub fn finalize_hex(&self) -> String {
        hex_encode(&self.clone().finalize())
    }

    fn process_block(state: &mut [u32; 5], block: &[u8; 64]) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(block[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];

        for (i, &w_val) in w.iter().enumerate().take(80) {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5a82_7999),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w_val);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
    }
}

impl Digest for Sha1 {
    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize_hex(self: Box<Self>) -> String {
        hex_encode(&self.finalize())
    }
}

#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    count: u64,
    buffer: [u8; 64],
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09_e667,
                0xbb67_ae85,
                0x3c6e_f372,
                0xa54f_f53a,
                0x510e_527f,
                0x9b05_688c,
                0x1f83_d9ab,
                0x5be0_cd19,
            ],
            count: 0,
            buffer: [0u8; 64],
        }
    }

    pub fn update(&mut self, input: &[u8]) {
        let idx = (self.count & 0x3f) as usize;
        self.count = self.count.wrapping_add(input.len() as u64);
        let mut input_idx = 0;

        if idx > 0 {
            let space = 64 - idx;
            if input.len() >= space {
                self.buffer[idx..64].copy_from_slice(&input[..space]);
                Self::process_block(&mut self.state, &self.buffer);
                input_idx = space;
            } else {
                self.buffer[idx..idx + input.len()].copy_from_slice(input);
                return;
            }
        }

        while input_idx + 64 <= input.len() {
            let block: &[u8; 64] = input[input_idx..input_idx + 64].try_into().unwrap();
            Self::process_block(&mut self.state, block);
            input_idx += 64;
        }

        if input_idx < input.len() {
            let rem = &input[input_idx..];
            self.buffer[..rem.len()].copy_from_slice(rem);
        }
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.count.wrapping_mul(8);
        let idx = (self.count & 0x3f) as usize;

        let pad_len = if idx < 56 { 56 - idx } else { 120 - idx };
        let mut pad = [0u8; 128];
        pad[0] = 0x80;
        self.update(&pad[..pad_len]);

        let len_bytes = bit_len.to_be_bytes();
        self.update(&len_bytes);

        let mut out = [0u8; 32];
        for (i, val) in self.state.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
        }
        out
    }

    pub fn finalize_hex(&self) -> String {
        hex_encode(&self.clone().finalize())
    }

    fn process_block(state: &mut [u32; 8], block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(block[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
}

impl Digest for Sha256 {
    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize_hex(self: Box<Self>) -> String {
        hex_encode(&self.finalize())
    }
}

#[derive(Clone)]
pub struct Sha512 {
    state: [u64; 8],
    count: u128,
    buffer: [u8; 128],
}

impl Default for Sha512 {
    fn default() -> Self {
        Self::new()
    }
}

const K512: [u64; 80] = [
    0x428a2f98d728ae22,
    0x7137449123ef65cd,
    0xb5c0fbcfec4d3b2f,
    0xe9b5dba58189dbbc,
    0x3956c25bf348b538,
    0x59f111f1b605d019,
    0x923f82a4af194f9b,
    0xab1c5ed5da6d8118,
    0xd807aa98a3030242,
    0x12835b0145706fbe,
    0x243185be4ee4b28c,
    0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,
    0x80deb1fe3b1696b1,
    0x9bdc06a725c71235,
    0xc19bf174cf692694,
    0xe49b69c19ef14ad2,
    0xefbe4786384f25e3,
    0x0fc19dc68b8cd5b5,
    0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,
    0x4a7484aa6ea6e483,
    0x5cb0a9dcbd41fbd4,
    0x76f988da831153b5,
    0x983e5152ee66dfab,
    0xa831c66d2db43210,
    0xb00327c898fb213f,
    0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,
    0xd5a79147930aa725,
    0x06ca6351e003826f,
    0x142929670a0e6e70,
    0x27b70a8546d22ffc,
    0x2e1b21385c26c926,
    0x4d2c6dfc5ac42aed,
    0x53380d139d95b3df,
    0x650a73548baf63de,
    0x766a0abb3c77b2a8,
    0x81c2c92e47edaee6,
    0x92722c851482353b,
    0xa2bfe8a14cf10364,
    0xa81a664bbc423001,
    0xc24b8b70d0f89791,
    0xc76c51a30654be30,
    0xd192e819d6ef5218,
    0xd69906245565a910,
    0xf40e35855771202a,
    0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,
    0x1e376c085141ab53,
    0x2748774cdf8eeb99,
    0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,
    0x4ed8aa4ae3418acb,
    0x5b9cca4f7763e373,
    0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,
    0x78a5636f43172f60,
    0x84c87814a1f0ab72,
    0x8cc702081a6439ec,
    0x90befffa23631e28,
    0xa4506cebde82bde9,
    0xbef9a3f7b2c67915,
    0xc67178f2e372532b,
    0xca273eceea26619c,
    0xd186b8c721c0c207,
    0xeada7dd6cde0eb1e,
    0xf57d4f7fee6ed178,
    0x06f067aa72176fba,
    0x0a637dc5a2c898a6,
    0x113f9804bef90dae,
    0x1b710b35131c471b,
    0x28db77f523047d84,
    0x32caab7b40c72493,
    0x3c9ebe0a15c9bebc,
    0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,
    0x597f299cfc657e2a,
    0x5fcb6fab3ad6faec,
    0x6c44198c4a475817,
];

impl Sha512 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667f3bcc908,
                0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b,
                0xa54ff53a5f1d36f1,
                0x510e527fade682d1,
                0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b,
                0x5be0cd19137e2179,
            ],
            count: 0,
            buffer: [0u8; 128],
        }
    }

    pub fn update(&mut self, input: &[u8]) {
        let idx = (self.count & 0x7f) as usize;
        self.count = self.count.wrapping_add(input.len() as u128);
        let mut input_idx = 0;

        if idx > 0 {
            let space = 128 - idx;
            if input.len() >= space {
                self.buffer[idx..128].copy_from_slice(&input[..space]);
                Self::process_block(&mut self.state, &self.buffer);
                input_idx = space;
            } else {
                self.buffer[idx..idx + input.len()].copy_from_slice(input);
                return;
            }
        }

        while input_idx + 128 <= input.len() {
            let block: &[u8; 128] = input[input_idx..input_idx + 128].try_into().unwrap();
            Self::process_block(&mut self.state, block);
            input_idx += 128;
        }

        if input_idx < input.len() {
            let rem = &input[input_idx..];
            self.buffer[..rem.len()].copy_from_slice(rem);
        }
    }

    pub fn finalize(mut self) -> [u8; 64] {
        let bit_len = self.count.wrapping_mul(8);
        let idx = (self.count & 0x7f) as usize;

        let pad_len = if idx < 112 { 112 - idx } else { 240 - idx };
        let mut pad = [0u8; 256];
        pad[0] = 0x80;
        self.update(&pad[..pad_len]);

        let len_bytes = bit_len.to_be_bytes();
        self.update(&len_bytes);

        let mut out = [0u8; 64];
        for (i, val) in self.state.iter().enumerate() {
            out[i * 8..(i + 1) * 8].copy_from_slice(&val.to_be_bytes());
        }
        out
    }

    pub fn finalize_hex(&self) -> String {
        hex_encode(&self.clone().finalize())
    }

    fn process_block(state: &mut [u64; 8], block: &[u8; 128]) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes(block[i * 8..(i + 1) * 8].try_into().unwrap());
        }
        for i in 16..80 {
            let s0 = w[i - 15].rotate_right(1) ^ w[i - 15].rotate_right(8) ^ (w[i - 15] >> 7);
            let s1 = w[i - 2].rotate_right(19) ^ w[i - 2].rotate_right(61) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K512[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }
}

impl Digest for Sha512 {
    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize_hex(self: Box<Self>) -> String {
        hex_encode(&self.finalize())
    }
}

#[derive(Clone, Default)]
pub struct BsdSum {
    checksum: u32,
    total_bytes: u64,
}

impl BsdSum {
    pub fn new() -> Self {
        Self {
            checksum: 0,
            total_bytes: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.total_bytes += data.len() as u64;
        let mut s = self.checksum;
        for &byte in data {
            s = (s >> 1) + ((s & 1) << 15) + (byte as u32);
            s &= 0xffff;
        }
        self.checksum = s;
    }

    pub fn finalize(&self) -> (u32, u64) {
        let blocks = self.total_bytes.div_ceil(1024);
        (self.checksum, blocks)
    }
}

#[derive(Clone, Default)]
pub struct SysVSum {
    sum: u32,
    total_bytes: u64,
}

impl SysVSum {
    pub fn new() -> Self {
        Self {
            sum: 0,
            total_bytes: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.total_bytes += data.len() as u64;
        let mut s = self.sum;
        for &byte in data {
            s = s.wrapping_add(byte as u32);
        }
        self.sum = s;
    }

    pub fn finalize(&self) -> (u32, u64) {
        let r = (self.sum & 0xffff) + (self.sum >> 16);
        let s = (r & 0xffff) + (r >> 16);
        let blocks = self.total_bytes.div_ceil(512);
        (s, blocks)
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let mut md5 = Md5::new();
        assert_eq!(
            md5.clone().finalize_hex(),
            "d41d8cd98f00b204e9800998ecf8427e"
        );

        md5.update(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(md5.finalize_hex(), "9e107d9d372bb6826bd81d3542a419d6");
    }

    #[test]
    fn test_sha1() {
        let mut sha1 = Sha1::new();
        assert_eq!(
            sha1.clone().finalize_hex(),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );

        sha1.update(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            sha1.finalize_hex(),
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12"
        );
    }

    #[test]
    fn test_sha256() {
        let mut sha256 = Sha256::new();
        assert_eq!(
            sha256.clone().finalize_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        sha256.update(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            sha256.finalize_hex(),
            "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        );
    }

    #[test]
    fn test_sha512() {
        let mut sha512 = Sha512::new();
        assert_eq!(
            sha512.clone().finalize_hex(),
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
        );

        sha512.update(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            sha512.finalize_hex(),
            "07e547d9586f6a73f73fbac0435ed76951218fb7d0c8d788a309d785436bbb642e93a252a954f23912547d1e8a3b5ed6e1bfd7097821233fa0538f3db854fee6"
        );
    }

    #[test]
    fn test_bsd_and_sysv() {
        let mut bsd = BsdSum::new();
        bsd.update(b"test");
        let (s_bsd, b_bsd) = bsd.finalize();
        assert_eq!(s_bsd, 16597);
        assert_eq!(b_bsd, 1);

        let mut sysv = SysVSum::new();
        sysv.update(b"test");
        let (s_sysv, b_sysv) = sysv.finalize();
        assert_eq!(s_sysv, 448);
        assert_eq!(b_sysv, 1);
    }
}
