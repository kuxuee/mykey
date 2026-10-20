//! Password derivation shared by the GPUI application and compatibility tests.
pub mod text_offsets;
use aes::{
    Aes256,
    cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray},
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroize;

const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`~!@#$%^&*()-_=+[{]}\\|;:\'\",<.>/?";
const SPECIAL: &[u8] = b"!@#$%^&*()-_=+?";

/// Matches the existing Dart implementation: UTF-8 inputs, no trimming or normalization,
/// PBKDF2-HMAC-SHA256 (4096 rounds), AES-256-CTR with a big-endian zero counter,
/// and gokey rejection sampling. The old 65536-byte buffer starts at byte zero.
pub fn derive_password(master: &str, service: &str) -> Result<String, &'static str> {
    if master.is_empty() {
        return Err("主密码不能为空");
    }
    if service.is_empty() {
        return Err("服务标识不能为空");
    }
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(
        master.as_bytes(),
        format!("{service}-pass").as_bytes(),
        4096,
        &mut key,
    );
    let cipher = Aes256::new(GenericArray::from_slice(&key));
    key.zeroize();
    let mut stream = ByteStream {
        cipher,
        counter: 0,
        block: [0; 16],
        offset: 16,
        consumed: 0,
    };
    loop {
        let mut candidate = [0u8; 16];
        for character in &mut candidate {
            // Preserve gokey's division into buckets, rather than using modulo.
            loop {
                let value = stream.next()? as usize;
                if value < 255 - 255 % CHARS.len() {
                    *character = CHARS[value / (255 / CHARS.len())];
                    break;
                }
            }
        }
        let valid = candidate.iter().any(u8::is_ascii_uppercase)
            && candidate.iter().any(u8::is_ascii_lowercase)
            && candidate.iter().any(u8::is_ascii_digit)
            && candidate.iter().any(|c| SPECIAL.contains(c))
            && candidate
                .iter()
                .all(|c| c.is_ascii_alphanumeric() || SPECIAL.contains(c));
        if valid {
            return Ok(String::from_utf8(candidate.to_vec()).expect("ASCII alphabet"));
        }
        candidate.zeroize();
    }
}

struct ByteStream {
    cipher: Aes256,
    counter: u128,
    block: [u8; 16],
    offset: usize,
    consumed: usize,
}

impl ByteStream {
    fn next(&mut self) -> Result<u8, &'static str> {
        if self.consumed >= 65536 {
            return Err("随机数据不足，无法生成密码");
        }
        if self.offset == 16 {
            self.block = self.counter.to_be_bytes();
            self.cipher
                .encrypt_block(GenericArray::from_mut_slice(&mut self.block));
            self.counter += 1;
            self.offset = 0;
        }
        let result = self.block[self.offset];
        self.offset += 1;
        self.consumed += 1;
        Ok(result)
    }
}

impl Drop for ByteStream {
    fn drop(&mut self) {
        self.block.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_existing_gokey_vectors() {
        for (master, service, expected) in [
            (
                "correct horse battery staple",
                "github.com",
                "jjbHr0a6*K_^-KsK",
            ),
            (
                "correct horse battery staple",
                "gmail.com",
                "_Cjw**lIR5lrB^1X",
            ),
            ("secret", "example.com", "pR2!h3Ap3z)hMOL!"),
        ] {
            assert_eq!(derive_password(master, service), Ok(expected.to_owned()));
        }
    }

    #[test]
    fn rejects_empty_inputs() {
        assert_eq!(derive_password("", "github.com"), Err("主密码不能为空"));
        assert_eq!(derive_password("secret", ""), Err("服务标识不能为空"));
    }

    #[test]
    fn preserves_unicode_and_whitespace_and_password_rules() {
        let first = derive_password("主密码🔑", " 服务.example ").unwrap();
        assert_eq!(
            first,
            derive_password("主密码🔑", " 服务.example ").unwrap()
        );
        assert_ne!(first, derive_password("主密码🔑", "服务.example").unwrap());
        assert_eq!(first.len(), 16);
        assert!(first.bytes().any(|c| c.is_ascii_uppercase()));
        assert!(first.bytes().any(|c| c.is_ascii_lowercase()));
        assert!(first.bytes().any(|c| c.is_ascii_digit()));
        assert!(first.bytes().any(|c| SPECIAL.contains(&c)));
        assert!(
            first
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || SPECIAL.contains(&c))
        );
    }
}
