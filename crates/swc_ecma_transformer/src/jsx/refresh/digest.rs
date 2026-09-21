pub(super) fn sha1_base64(input: &str) -> String {
    let digest = sha1(input.as_bytes());
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(28);
    for chunk in digest.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[((first & 0x03) << 4 | second >> 4) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((second & 0x0f) << 2 | third >> 6) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(third & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

fn sha1(input: &[u8]) -> [u8; 20] {
    let mut state = [
        0x6745_2301u32,
        0xefcd_ab89,
        0x98ba_dcfe,
        0x1032_5476,
        0xc3d2_e1f0,
    ];
    let (blocks, remainder) = input.as_chunks::<64>();
    for block in blocks {
        compress(&mut state, block);
    }
    let mut final_block = [0u8; 64];
    final_block[..remainder.len()].copy_from_slice(remainder);
    final_block[remainder.len()] = 0x80;
    if remainder.len() >= 56 {
        compress(&mut state, &final_block);
        final_block = [0; 64];
    }
    let bit_len = (input.len() as u64).wrapping_mul(8);
    final_block[56..].copy_from_slice(&bit_len.to_be_bytes());
    compress(&mut state, &final_block);
    let mut digest = [0u8; 20];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn compress(state: &mut [u32; 5], block: &[u8; 64]) {
    let mut words = [0u32; 80];
    for (index, word) in words[..16].iter_mut().enumerate() {
        let offset = index * 4;
        *word = u32::from_be_bytes([
            block[offset],
            block[offset + 1],
            block[offset + 2],
            block[offset + 3],
        ]);
    }
    for index in 16..80 {
        words[index] =
            (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                .rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *state;
    for (index, word) in words.iter().enumerate() {
        let (function, constant) = match index {
            0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999),
            20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
            _ => (b ^ c ^ d, 0xca62_c1d6),
        };
        let next = a
            .rotate_left(5)
            .wrapping_add(function)
            .wrapping_add(e)
            .wrapping_add(constant)
            .wrapping_add(*word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = next;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

#[cfg(test)]
mod tests {
    use super::sha1_base64;

    #[test]
    fn refresh_signature_hash() {
        assert_eq!(sha1_base64("useContext{}"), "gDsCjeeItUuvgOWf1v4qoK9RF6k=");

        assert_eq!(sha1_base64(""), "2jmj7l5rSw0yVb/vlWAYkK/YBwk=");

        assert_eq!(sha1_base64("abc"), "qZk+NkcGgWq6PiVxeFDCbJzQ2J0=");

        let output = sha1_base64(&"a".repeat(55));
        assert_eq!(output, "wci73CJ5bijA4VFj0giZtlYh1lo=");

        let output = sha1_base64(&"a".repeat(56));
        assert_eq!(output, "wtszD2CDhUyZ1LW/tujynyAb5pk=");

        let output = sha1_base64(&"a".repeat(64));
        assert_eq!(output, "AJi6gktcFkJ716ESKlpEKiXsZE0=");

        let output = sha1_base64(&"a".repeat(120));
        assert_eq!(output, "80wUiDhTRqVXCboFbd0IKA3UxtY=");

        let output = sha1_base64(&"a".repeat(128));
        assert_eq!(output, "rVs/28tSZ3jCg50vFR6nU5leJqA=");
    }
}
