use std::io::{Error, ErrorKind};

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bitbuf: u32,
    bitcnt: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bitbuf: 0,
            bitcnt: 0,
        }
    }

    fn fill(&mut self, need: u32) -> std::io::Result<()> {
        while self.bitcnt < need {
            if self.pos >= self.data.len() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "truncated deflate stream",
                ));
            }
            self.bitbuf |= (self.data[self.pos] as u32) << self.bitcnt;
            self.pos += 1;
            self.bitcnt += 8;
        }
        Ok(())
    }

    fn bits(&mut self, n: u32) -> std::io::Result<u32> {
        if n == 0 {
            return Ok(0);
        }
        self.fill(n)?;
        let v = self.bitbuf & (if n == 32 { u32::MAX } else { (1u32 << n) - 1 });
        self.bitbuf >>= n;
        self.bitcnt -= n;
        Ok(v)
    }

    fn align_byte(&mut self) {
        let consumed_bits = self.pos * 8 - self.bitcnt as usize;
        self.pos = consumed_bits.div_ceil(8);
        self.bitbuf = 0;
        self.bitcnt = 0;
    }

    fn read_bytes(&mut self, n: usize) -> std::io::Result<&'a [u8]> {
        self.align_byte();

        if self.pos + n > self.data.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "truncated stored block",
            ));
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
}

struct Huffman {
    table: Vec<Vec<i32>>,
    max_len: usize,
}

fn reverse_bits(mut v: u32, n: usize) -> u32 {
    let mut r = 0u32;
    for _ in 0..n {
        r = (r << 1) | (v & 1);
        v >>= 1;
    }
    r
}

impl Huffman {
    fn from_lengths(lengths: &[u8]) -> std::io::Result<Self> {
        let max_len = lengths.iter().copied().max().unwrap_or(0) as usize;
        if max_len > 15 {
            return Err(Error::new(ErrorKind::InvalidData, "bad huffman length"));
        }
        let mut bl_count = [0u32; 16];
        for &l in lengths {
            if l as usize > 15 {
                return Err(Error::new(ErrorKind::InvalidData, "bad code length"));
            }
            if l != 0 {
                bl_count[l as usize] += 1;
            }
        }
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for b in 1..16 {
            code = (code + bl_count[b - 1]) << 1;
            next_code[b] = code;
        }
        let mut table: Vec<Vec<i32>> = vec![Vec::new(); 16];
        for (b, slot) in table.iter_mut().enumerate().skip(1) {
            *slot = vec![-1; 1 << b];
        }
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                let c = next_code[len as usize];
                next_code[len as usize] += 1;

                let rev = reverse_bits(c, len as usize);
                if rev as usize >= table[len as usize].len() {
                    return Err(Error::new(ErrorKind::InvalidData, "bad huffman code"));
                }
                table[len as usize][rev as usize] = sym as i32;
            }
        }
        Ok(Self { table, max_len })
    }

    fn decode(&self, br: &mut BitReader<'_>) -> std::io::Result<usize> {
        let mut code = 0u32;
        for len in 1..=self.max_len {
            br.fill(len as u32)?;
            code |= (br.bitbuf & 1) << (len - 1);
            br.bitbuf >>= 1;
            br.bitcnt -= 1;
            if len < self.table.len() && (code as usize) < self.table[len].len() {
                let s = self.table[len][code as usize];
                if s >= 0 {
                    return Ok(s as usize);
                }
            }
        }
        Err(Error::new(ErrorKind::InvalidData, "invalid huffman code"))
    }
}

fn fixed_tables() -> (Huffman, Huffman) {
    let mut ll = vec![0u8; 288];
    ll[..144].fill(8);
    ll[144..256].fill(9);
    ll[256..280].fill(7);
    ll[280..].fill(8);
    let dl = vec![5u8; 32];
    (
        Huffman::from_lengths(&ll).expect("fixed litlen"),
        Huffman::from_lengths(&dl).expect("fixed dist"),
    )
}

const LEN_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn decode_dynamic(br: &mut BitReader<'_>) -> std::io::Result<(Huffman, Huffman)> {
    let hlit = br.bits(5)? as usize + 257;
    let hdist = br.bits(5)? as usize + 1;
    let hclen = br.bits(4)? as usize + 4;
    if hlit > 288 || hdist > 32 {
        return Err(Error::new(ErrorKind::InvalidData, "bad HLIT/HDIST"));
    }
    let mut cl_len = [0u8; 19];
    for i in 0..hclen {
        cl_len[CL_ORDER[i]] = br.bits(3)? as u8;
    }
    let cl = Huffman::from_lengths(&cl_len)?;
    let total = hlit + hdist;
    let mut lens: Vec<u8> = Vec::with_capacity(total);
    while lens.len() < total {
        let s = cl.decode(br)?;
        match s {
            0..=15 => lens.push(s as u8),
            16 => {
                let rep = br.bits(2)? as usize + 3;
                let last = *lens.last().ok_or_else(|| {
                    Error::new(ErrorKind::InvalidData, "repeat with no previous length")
                })?;
                lens.extend(std::iter::repeat_n(last, rep));
            }
            17 => {
                let rep = br.bits(3)? as usize + 3;
                lens.extend(std::iter::repeat_n(0, rep));
            }
            18 => {
                let rep = br.bits(7)? as usize + 11;
                lens.extend(std::iter::repeat_n(0, rep));
            }
            _ => return Err(Error::new(ErrorKind::InvalidData, "bad code-length symbol")),
        }
        if lens.len() > total {
            return Err(Error::new(ErrorKind::InvalidData, "too many code lengths"));
        }
    }
    let ll = Huffman::from_lengths(&lens[..hlit])?;
    let dl = Huffman::from_lengths(&lens[hlit..])?;
    Ok((ll, dl))
}

fn decode_body(
    br: &mut BitReader<'_>,
    litlen: &Huffman,
    dist: &Huffman,
    out: &mut Vec<u8>,
) -> std::io::Result<bool> {
    loop {
        let sym = litlen.decode(br)?;
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(false),
            257..=285 => {
                let li = sym - 257;
                let len = LEN_BASE[li] + br.bits(LEN_EXTRA[li])?;
                let dsym = dist.decode(br)?;
                if dsym >= 30 {
                    return Err(Error::new(ErrorKind::InvalidData, "bad distance symbol"));
                }
                let d = DIST_BASE[dsym] + br.bits(DIST_EXTRA[dsym])?;
                if d as usize > out.len() || d == 0 {
                    return Err(Error::new(ErrorKind::InvalidData, "distance too far back"));
                }
                let start = out.len() - d as usize;
                for i in 0..len as usize {
                    let b = out[start + i];
                    out.push(b);
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "bad literal/length symbol",
                ))
            }
        }
    }
}

pub(crate) fn inflate_all(input: &[u8]) -> std::io::Result<Vec<u8>> {
    inflate_common(input).map(|(o, _)| o)
}

pub(crate) fn inflate_all_consumed(input: &[u8]) -> std::io::Result<(Vec<u8>, usize)> {
    inflate_common(input)
}

fn inflate_common(input: &[u8]) -> std::io::Result<(Vec<u8>, usize)> {
    let mut br = BitReader::new(input);
    let mut out: Vec<u8> = Vec::new();
    loop {
        let bfinal = br.bits(1)?;
        let btype = br.bits(2)?;
        let done = bfinal != 0;
        match btype {
            0 => {
                br.align_byte();
                let len_bytes = br.read_bytes(4)?;
                let len = u16::from_le_bytes([len_bytes[0], len_bytes[1]]) as usize;
                let nlen = u16::from_le_bytes([len_bytes[2], len_bytes[3]]) as usize;
                if len ^ 0xFFFF != nlen {
                    return Err(Error::new(ErrorKind::InvalidData, "bad stored LEN/NLEN"));
                }
                if len > 0 {
                    let chunk = br.read_bytes(len)?;
                    out.extend_from_slice(chunk);
                }
                if done {
                    break;
                }
            }
            1 => {
                let (ll, dd) = fixed_tables();
                if decode_body(&mut br, &ll, &dd, &mut out)? {
                    break;
                }
                if done {
                    break;
                }
            }
            2 => {
                let (ll, dd) = decode_dynamic(&mut br)?;
                if decode_body(&mut br, &ll, &dd, &mut out)? {
                    break;
                }
                if done {
                    break;
                }
            }
            _ => return Err(Error::new(ErrorKind::InvalidData, "reserved block type")),
        }
    }
    let consumed = br.pos - (br.bitcnt as usize / 8);
    Ok((out, consumed))
}
