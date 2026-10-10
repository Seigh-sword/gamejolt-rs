use crate::error::Result;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HashAlgorithm {
    Md5,
    Sha1,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Sha1
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashAlgorithm::Md5 => write!(f, "md5"),
            HashAlgorithm::Sha1 => write!(f, "sha1"),
        }
    }
}

#[derive(Clone)]
pub struct Signer {
    private_key: String,
    algorithm: HashAlgorithm,
}

impl fmt::Debug for Signer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signer")
            .field("algorithm", &self.algorithm)
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

impl Signer {
    pub fn new(private_key: impl Into<String>, algorithm: HashAlgorithm) -> Self {
        Self {
            private_key: private_key.into(),
            algorithm,
        }
    }

    pub fn sign(&self, params: &[(&str, &str)]) -> Result<String> {
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in params {
            query.append_pair(key, value);
        }
        self.digest(&(query.finish() + &self.private_key))
    }

    pub fn sign_url(&self, url: &str) -> Result<String> {
        self.digest(&(url.to_owned() + &self.private_key))
    }

    fn digest(&self, data: &str) -> Result<String> {
        match self.algorithm {
            HashAlgorithm::Md5 => self.sign_md5(data),
            HashAlgorithm::Sha1 => self.sign_sha1(data),
        }
    }

    #[cfg(feature = "md5")]
    fn sign_md5(&self, data: &str) -> Result<String> {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    #[cfg(not(feature = "md5"))]
    fn sign_md5(&self, _data: &str) -> Result<String> {
        Err(crate::error::GameJoltError::SignatureError(
            "MD5 support not enabled. Enable 'md5' feature.".to_string(),
        ))
    }

    #[cfg(feature = "sha1")]
    fn sign_sha1(&self, data: &str) -> Result<String> {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    #[cfg(not(feature = "sha1"))]
    fn sign_sha1(&self, _data: &str) -> Result<String> {
        Err(crate::error::GameJoltError::SignatureError(
            "SHA1 support not enabled. Enable 'sha1' feature.".to_string(),
        ))
    }

    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_signing() {
        let signer = Signer::new("private_key", HashAlgorithm::Sha1);
        let params = vec![("user", "test"), ("score", "100")];
        let signature = signer.sign(&params).unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 40);
    }

    #[test]
    fn test_md5_signing() {
        #[cfg(feature = "md5")]
        {
            let signer = Signer::new("private_key", HashAlgorithm::Md5);
            let params = vec![("user", "test"), ("score", "100")];
            let signature = signer.sign(&params).unwrap();
            assert!(!signature.is_empty());
            assert_eq!(signature.len(), 32);
        }
        #[cfg(not(feature = "md5"))]
        {
            let signer = Signer::new("private_key", HashAlgorithm::Md5);
            assert!(signer.sign(&[]).is_err());
        }
    }

    #[test]
    fn test_signature_deterministic() {
        let signer = Signer::new("private_key", HashAlgorithm::Sha1);
        let params = vec![("user", "test"), ("score", "100")];
        let sig1 = signer.sign(&params).unwrap();
        let sig2 = signer.sign(&params).unwrap();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sha1_known_vector() {
        let signer = Signer::new("", HashAlgorithm::Sha1);
        assert_eq!(
            signer.sign_url("abc").unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[cfg(feature = "md5")]
    #[test]
    fn test_md5_known_vector() {
        let signer = Signer::new("", HashAlgorithm::Md5);
        assert_eq!(
            signer.sign_url("abc").unwrap(),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn test_different_params_different_sig() {
        let signer = Signer::new("private_key", HashAlgorithm::Sha1);
        let sig1 = signer.sign(&[("user", "test")]).unwrap();
        let sig2 = signer.sign(&[("user", "other")]).unwrap();
        assert_ne!(sig1, sig2);
    }
}
