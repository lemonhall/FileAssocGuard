pub const USER_EXPERIENCE: &str =
    "user choice set via windows user experience {d18b6dd5-6124-4341-9318-804003bafa0b}";

pub fn compute_user_choice_hash(
    extension: &str,
    sid: &str,
    prog_id: &str,
    regdate_hex: &str,
) -> String {
    use base64::Engine;

    let input = format!(
        "{}{}{}{}{}\0",
        extension, sid, prog_id, regdate_hex, USER_EXPERIENCE
    );
    let input = input.to_ascii_lowercase();

    let bytes = utf16le_bytes(&input);
    let digest = md5::compute(&bytes);
    let md5 = digest.0;

    let h1 = sub_1(&bytes, md5);
    let h2 = sub_2(&bytes, md5);

    let mut finalraw = [0u8; 8];
    for i in 0..8 {
        finalraw[i] = h1[i] ^ h2[i];
    }

    base64::engine::general_purpose::STANDARD.encode(finalraw)
}

fn utf16le_bytes(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 2);
    for u in s.encode_utf16() {
        out.extend_from_slice(&u.to_le_bytes());
    }
    out
}

fn length_in_dwords(data_len_bytes: usize) -> usize {
    let n = data_len_bytes / 4;
    n & !1
}

fn dword_data(data: &[u8]) -> Vec<u32> {
    let length = length_in_dwords(data.len());
    let mut out = Vec::with_capacity(length);
    for i in 0..length {
        let off = i * 4;
        out.push(u32::from_le_bytes(data[off..off + 4].try_into().unwrap()));
    }
    out
}

fn dword_md5(md5: [u8; 16]) -> [u32; 4] {
    [
        u32::from_le_bytes(md5[0..4].try_into().unwrap()),
        u32::from_le_bytes(md5[4..8].try_into().unwrap()),
        u32::from_le_bytes(md5[8..12].try_into().unwrap()),
        u32::from_le_bytes(md5[12..16].try_into().unwrap()),
    ]
}

fn sub_1(data: &[u8], md5: [u8; 16]) -> [u8; 8] {
    let length = length_in_dwords(data.len());
    let mut retval = [0u8; 8];
    if length <= 1 || (length & 1) == 1 {
        return retval;
    }

    let dword_data = dword_data(data);
    let dword_md5 = dword_md5(md5);

    let mut v5: u32 = 0;
    let mut v6: usize = 0;
    let mut v7: u32 = ((length as u32).wrapping_sub(2)) >> 1;
    let v18: u32 = v7;
    v7 = v7.wrapping_add(1);
    let mut v8: u32 = v7;
    let v19: u32 = v7;
    let mut result: u32 = 0;
    let v9: u32 = (dword_md5[1] | 1).wrapping_add(0x13DB0000);
    let v10: u32 = (dword_md5[0] | 1).wrapping_add(0x69FB0000);

    while v8 != 0 {
        let v11 = dword_data[v6].wrapping_add(result);
        v6 += 2;
        let t1 = v10.wrapping_mul(v11)
            .wrapping_sub(0x10FA9605u32.wrapping_mul(v11 >> 16));
        let v12 = 0x79F8A395u32
            .wrapping_mul(t1)
            .wrapping_add(0x689B6B9Fu32.wrapping_mul(t1 >> 16));
        let v13 = 0xEA970001u32
            .wrapping_mul(v12)
            .wrapping_sub(0x3C101569u32.wrapping_mul(v12 >> 16));
        let v14 = v13.wrapping_add(v5);
        let v15_input = dword_data[v6 - 1].wrapping_add(v13);
        let v15 = v9
            .wrapping_mul(v15_input)
            .wrapping_sub(0x3CE8EC25u32.wrapping_mul(v15_input >> 16));
        let t2 = 0x59C3AF2Du32
            .wrapping_mul(v15)
            .wrapping_sub(0x2232E0F1u32.wrapping_mul(v15 >> 16));
        result = 0x1EC90001u32
            .wrapping_mul(t2)
            .wrapping_add(0x35BD1EC9u32.wrapping_mul(t2 >> 16));
        v5 = result.wrapping_add(v14);
        v8 = v8.wrapping_sub(1);
    }

    if (length as u32)
        .wrapping_sub(2)
        .wrapping_sub(2u32.wrapping_mul(v18))
        == 1
    {
        let idx = 2usize.wrapping_mul(v19 as usize);
        let v16_input = dword_data[idx].wrapping_add(result);
        let v16 = v16_input
            .wrapping_mul(v10)
            .wrapping_sub(0x10FA9605u32.wrapping_mul(v16_input >> 16));

        let v17 = 0x39646B9Fu32
            .wrapping_mul(v16 >> 16)
            .wrapping_add(0x28DBA395u32.wrapping_mul(v16))
            .wrapping_sub(
                0x3C101569u32.wrapping_mul(
                    (0x689B6B9Fu32.wrapping_mul(v16 >> 16)
                        .wrapping_add(0x79F8A395u32.wrapping_mul(v16)))
                        >> 16,
                ),
            );

        let v17v9 = v17
            .wrapping_mul(v9)
            .wrapping_sub(0x3CE8EC25u32.wrapping_mul(v17 >> 16));

        let t3 = 0x59C3AF2Du32
            .wrapping_mul(v17v9)
            .wrapping_sub(0x2232E0F1u32.wrapping_mul(v17v9 >> 16));

        result = 0x35BD1EC9u32
            .wrapping_mul(t3 >> 16)
            .wrapping_add(0x2A18AF2Du32.wrapping_mul(v17v9))
            .wrapping_sub(0xFD6BE0F1u32.wrapping_mul(v17v9 >> 16));

        v5 = v5.wrapping_add(result).wrapping_add(v17);
    }

    retval[0..4].copy_from_slice(&result.to_le_bytes());
    retval[4..8].copy_from_slice(&v5.to_le_bytes());
    retval
}

fn sub_2(data: &[u8], md5: [u8; 16]) -> [u8; 8] {
    let length = length_in_dwords(data.len());
    let mut retval = [0u8; 8];
    if length <= 1 || (length & 1) == 1 {
        return retval;
    }

    let dword_data = dword_data(data);
    let dword_md5 = dword_md5(md5);

    let mut v5: u32 = 0;
    let mut v6: usize = 0;
    let mut v7: u32 = 0;
    let v25: u32 = ((length as u32).wrapping_sub(2)) >> 1;
    let v21: u32 = dword_md5[0] | 1;
    let v22: u32 = dword_md5[1] | 1;
    let v23: u32 = 0xB1110000u32.wrapping_mul(v21);
    let v24: u32 = 0x16F50000u32.wrapping_mul(v22);
    let mut v8: u32 = v25.wrapping_add(1);

    while v8 != 0 {
        v6 += 2;
        let left = dword_data[v6 - 2].wrapping_add(v5);
        let v9 = left
            .wrapping_mul(v23)
            .wrapping_sub(0x30674EEFu32.wrapping_mul(v21.wrapping_mul(left) >> 16));
        let v10 = v9 >> 16;
        let v11 = 0xE9B30000u32
            .wrapping_mul(v10)
            .wrapping_add(
                0x12CEB96Du32.wrapping_mul(
                    (0x5B9F0000u32.wrapping_mul(v9)
                        .wrapping_sub(0x78F7A461u32.wrapping_mul(v10)))
                        >> 16,
                ),
            );
        let v12 = 0x1D830000u32
            .wrapping_mul(v11)
            .wrapping_add(0x257E1D83u32.wrapping_mul(v11 >> 16));

        let right = v12.wrapping_add(dword_data[v6 - 1]);
        let t1 = right
            .wrapping_mul(v24)
            .wrapping_sub(0x5D8BE90Bu32.wrapping_mul(v22.wrapping_mul(right) >> 16));
        let v13 = t1 >> 16;
        let v14 = (0x96FF0000u32
            .wrapping_mul(t1)
            .wrapping_sub(0x2C7C6901u32.wrapping_mul(v13)))
            >> 16;
        v5 = 0xF2310000u32
            .wrapping_mul(v14)
            .wrapping_sub(
                0x405B6097u32.wrapping_mul(
                    (0x7C932B89u32.wrapping_mul(v14)
                        .wrapping_sub(0x5C890000u32.wrapping_mul(v13)))
                        >> 16,
                ),
            );
        v7 = v7.wrapping_add(v5).wrapping_add(v12);
        v8 = v8.wrapping_sub(1);
    }

    if (length as u32)
        .wrapping_sub(2)
        .wrapping_sub(2u32.wrapping_mul(v25))
        == 1
    {
        let idx = 2usize.wrapping_mul(v25.wrapping_add(1) as usize);
        let left = dword_data[idx].wrapping_add(v5);

        let v15 = 0xB1110000u32
            .wrapping_mul(v21)
            .wrapping_mul(left)
            .wrapping_sub(0x30674EEFu32.wrapping_mul(v21.wrapping_mul(left) >> 16));
        let v16 = v15 >> 16;
        let v17 = (0x5B9F0000u32
            .wrapping_mul(v15)
            .wrapping_sub(0x78F7A461u32.wrapping_mul(v15 >> 16)))
            >> 16;
        let v18 = 0x257E1D83u32
            .wrapping_mul((0xE9B30000u32.wrapping_mul(v16).wrapping_add(0x12CEB96Du32.wrapping_mul(v17))) >> 16)
            .wrapping_add(0x3BC70000u32.wrapping_mul(v17));

        let t1 = 0x16F50000u32
            .wrapping_mul(v18)
            .wrapping_mul(v22)
            .wrapping_sub(0x5D8BE90Bu32.wrapping_mul(v18.wrapping_mul(v22) >> 16));
        let v19 = t1 >> 16;
        let v20 = (0x96FF0000u32
            .wrapping_mul(t1)
            .wrapping_sub(0x2C7C6901u32.wrapping_mul(v19)))
            >> 16;
        v5 = 0xF2310000u32
            .wrapping_mul(v20)
            .wrapping_sub(
                0x405B6097u32.wrapping_mul(
                    (0x7C932B89u32.wrapping_mul(v20)
                        .wrapping_sub(0x5C890000u32.wrapping_mul(v19)))
                        >> 16,
                ),
            );
        v7 = v7.wrapping_add(v5).wrapping_add(v18);
    }

    retval[0..4].copy_from_slice(&v5.to_le_bytes());
    retval[4..8].copy_from_slice(&v7.to_le_bytes());
    retval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_vector_3g2() {
        let hash = compute_user_choice_hash(
            ".3g2",
            "S-1-5-21-819709642-920330688-1657285119-500",
            "WMP11.AssocFile.3G2",
            "01d4d98267246000",
        );
        assert_eq!(hash, "PCCqEmkvW2Y=");
    }

    #[test]
    fn hash_vector_txtfile() {
        let hash = compute_user_choice_hash(
            ".txt",
            "S-1-5-21-463486358-3398762107-1964875780-1001",
            "txtfile",
            "01d3442a29887400",
        );
        assert_eq!(hash, "PGINlytwZJo=");
    }

    #[test]
    fn hash_vector_mp4_potplayer() {
        let hash = compute_user_choice_hash(
            ".mp4",
            "S-1-5-21-463486358-3398762107-1964875780-1001",
            "PotPlayer.mp4",
            "01d4d98267246000",
        );
        assert_eq!(hash, "bqwC5h8a7rY=");
    }
}
